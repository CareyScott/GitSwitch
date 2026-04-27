use serde::Serialize;
use std::io::Write;
use std::process::{Command, Stdio};

use crate::accounts;

#[derive(Serialize, Clone, Debug)]
pub struct GitUser {
    pub name: String,
    pub email: String,
}

fn host_for_provider(provider: &str) -> Option<&'static str> {
    match provider {
        "github" => Some("github.com"),
        "bitbucket" => Some("bitbucket.org"),
        _ => None,
    }
}

fn git_config_get(key: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["config", "--global", "--get", key])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn git_config_set(key: &str, value: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["config", "--global", key, value])
        .output()
        .map_err(|e| format!("Failed to run git config: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git config failed: {}", stderr));
    }
    Ok(())
}

// Pipe credential descriptors into `git credential <action>` on stdin.
// On Windows that routes to Git Credential Manager (which stores in Windows
// Credential Manager); on macOS the default helper is osxkeychain. The
// stored credential is what `git push` / `git clone` over HTTPS uses.
fn git_credential_op(
    action: &str,
    host: &str,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<(), String> {
    let mut child = Command::new("git")
        .args(["credential", action])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn git credential {}: {}", action, e))?;

    let mut payload = format!("protocol=https\nhost={}\n", host);
    if let Some(u) = username {
        payload.push_str(&format!("username={}\n", u));
    }
    if let Some(p) = password {
        payload.push_str(&format!("password={}\n", p));
    }
    payload.push('\n');

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(payload.as_bytes())
            .map_err(|e| format!("Failed to write to git credential stdin: {}", e))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to wait for git credential: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git credential {} failed: {}", action, stderr));
    }
    Ok(())
}

pub fn forget_credential(host: &str, username: &str) -> Result<(), String> {
    git_credential_op("reject", host, Some(username), None)
}

// Forget the default (unnamespaced) credential for a host. Used to clear
// stale entries left over from previous setups — e.g. a `PersonalAccessToken`
// credential that GCM stored before the user installed GitSwitch. Without
// this, `git push` would keep using the old credential because git asks the
// helper without a username and gets back the unnamespaced default.
fn forget_default_credential(host: &str) {
    // Try a couple of common default usernames that GCM uses when no
    // username is specified at store time.
    let _ = git_credential_op("reject", host, None, None);
    let _ = git_credential_op("reject", host, Some("PersonalAccessToken"), None);
}

/// Ensure a credential helper is configured. On macOS, defaults to
/// `osxkeychain`; on Windows, GCM is typically pre-installed. Without a
/// helper, `git credential approve/reject` operations have no effect.
fn ensure_credential_helper() {
    // Check if any credential.helper is already set (global level)
    let output = Command::new("git")
        .args(["config", "--global", "--get", "credential.helper"])
        .output();

    let already_set = output
        .as_ref()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if already_set {
        return;
    }

    // Also check system level (Xcode's git sets it there on macOS)
    let system_output = Command::new("git")
        .args(["config", "--system", "--get", "credential.helper"])
        .output();

    let system_set = system_output
        .as_ref()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if system_set {
        return;
    }

    // Set osxkeychain on macOS, manager on Windows
    let helper = if cfg!(target_os = "macos") {
        "osxkeychain"
    } else if cfg!(target_os = "windows") {
        "manager"
    } else {
        return; // Linux: too many options, skip auto-config
    };

    let _ = git_config_set("credential.helper", helper);
}

/// Add a URL rewrite so that remotes with an embedded username for this
/// host are rewritten to the plain URL. This allows `credential.<host>.username`
/// pinning to take effect even when the remote has `user@host` in it.
///
/// Sets: url."https://{host}/".insteadOf = "https://{embedded_user}@{host}/"
fn add_url_rewrite(host: &str, embedded_username: &str) {
    // git config --global --get-all to check if this rewrite already exists
    let check = Command::new("git")
        .args([
            "config",
            "--global",
            "--get-all",
            &format!("url.https://{host}/.insteadOf"),
        ])
        .output();

    let rewrite_value = format!("https://{}@{}/", embedded_username, host);

    // Check if this specific rewrite is already present
    if let Ok(output) = check {
        let existing = String::from_utf8_lossy(&output.stdout);
        if existing.lines().any(|line| line.trim() == rewrite_value.trim()) {
            return; // Already configured
        }
    }

    // Use --add to append without overwriting other insteadOf entries
    let _ = Command::new("git")
        .args([
            "config",
            "--global",
            "--add",
            &format!("url.https://{host}/.insteadOf"),
            &rewrite_value,
        ])
        .output();
}

/// Clear stale entries, store the credential, and pin git to the correct
/// username for the host. When a `url_username` (e.g., Bitbucket workspace
/// handle) is provided and differs from the auth `username` (e.g., email),
/// the workspace handle is used as the primary credential identity because
/// Git URL-encodes `@` in usernames (turning `scott@beatvest.com` into
/// `scott%40beatvest.com`), which breaks credential lookups.
pub fn pin_credential(provider: &str, username: &str, token: &str, url_username: Option<&str>) {
    let Some(host) = host_for_provider(provider) else {
        return;
    };

    // Ensure the OS credential helper is configured before we try to store anything.
    ensure_credential_helper();

    forget_default_credential(host);

    // Determine the credential username. Prefer the url_username (workspace
    // handle) over the auth username (email) because Git cannot use emails
    // as credential usernames — the `@` gets URL-encoded to `%40`.
    let cred_user = url_username.unwrap_or(username);

    // Reject the old email-based credential if it exists (clean up from
    // previous versions that stored under the email).
    if cred_user != username {
        let _ = git_credential_op("reject", host, Some(username), None);
    }

    // Store and pin the credential under the workspace handle.
    let _ = git_credential_op("approve", host, Some(cred_user), Some(token));
    let url_key = format!("credential.https://{}.username", host);
    let _ = git_config_set(&url_key, cred_user);

    // Add a URL rewrite to strip the embedded username from remote URLs,
    // so the credential.username pin takes effect.
    add_url_rewrite(host, cred_user);
}

pub fn host_for(provider: &str) -> Option<&'static str> {
    host_for_provider(provider)
}

// ---- SSH config management ----

/// Update ~/.ssh/config so the given host uses the specified SSH key.
/// Creates the Host block if it doesn't exist, or updates the IdentityFile
/// line if it does. Preserves all other entries and formatting.
fn update_ssh_config(host: &str, key_path: &str) -> Result<(), String> {
    let ssh_dir = dirs::home_dir()
        .ok_or("Could not determine home directory")?
        .join(".ssh");
    let config_path = ssh_dir.join("config");

    // Ensure ~/.ssh exists with correct permissions
    if !ssh_dir.exists() {
        std::fs::create_dir_all(&ssh_dir)
            .map_err(|e| format!("Failed to create ~/.ssh: {}", e))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&ssh_dir, std::fs::Permissions::from_mode(0o700))
                .map_err(|e| format!("Failed to set ~/.ssh permissions: {}", e))?;
        }
    }

    let content = if config_path.exists() {
        std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read SSH config: {}", e))?
    } else {
        String::new()
    };

    let new_content = rewrite_ssh_host_block(&content, host, key_path);

    std::fs::write(&config_path, new_content)
        .map_err(|e| format!("Failed to write SSH config: {}", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| format!("Failed to set SSH config permissions: {}", e))?;
    }

    Ok(())
}

/// Parse SSH config, find or create the Host block for `host`, and set
/// its IdentityFile to `key_path`. Preserves all other blocks and options.
fn rewrite_ssh_host_block(content: &str, host: &str, key_path: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result: Vec<String> = Vec::new();
    let mut found_host = false;
    let mut in_target_block = false;
    let mut replaced_identity = false;
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // Detect start of a Host block
        if trimmed.starts_with("Host ") || trimmed.starts_with("Host\t") {
            if in_target_block && !replaced_identity {
                // We were in the target block but never found IdentityFile — add it
                result.push(format!("  IdentityFile {}", key_path));
                replaced_identity = true;
            }

            let host_value = trimmed.strip_prefix("Host").unwrap().trim();
            if host_value == host {
                found_host = true;
                in_target_block = true;
                replaced_identity = false;
                result.push(line.to_string());
                i += 1;
                continue;
            } else {
                in_target_block = false;
            }
        }

        if in_target_block && trimmed.starts_with("IdentityFile ") {
            // Replace the IdentityFile line
            result.push(format!("  IdentityFile {}", key_path));
            replaced_identity = true;
            i += 1;
            continue;
        }

        result.push(line.to_string());
        i += 1;
    }

    // If we were in the target block at EOF and never replaced
    if in_target_block && !replaced_identity {
        result.push(format!("  IdentityFile {}", key_path));
    }

    // If the host block didn't exist at all, append it
    if !found_host {
        if !result.is_empty() && !result.last().map_or(true, |l| l.is_empty()) {
            result.push(String::new());
        }
        result.push(format!("Host {}", host));
        result.push("  AddKeysToAgent yes".to_string());
        result.push(format!("  IdentityFile {}", key_path));
    }

    let mut output = result.join("\n");
    // Ensure file ends with newline
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

#[tauri::command]
pub fn list_ssh_keys() -> Result<Vec<String>, String> {
    let ssh_dir = dirs::home_dir()
        .ok_or("Could not determine home directory")?
        .join(".ssh");

    if !ssh_dir.exists() {
        return Ok(vec![]);
    }

    let mut keys = Vec::new();
    let entries = std::fs::read_dir(&ssh_dir)
        .map_err(|e| format!("Failed to read ~/.ssh: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        // Look for private keys (files that have a corresponding .pub file)
        if path.is_file() {
            let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            // Skip .pub files, config, known_hosts, authorized_keys, and hidden files
            if name.ends_with(".pub")
                || name == "config"
                || name == "known_hosts"
                || name == "known_hosts.old"
                || name == "authorized_keys"
                || name.starts_with('.')
            {
                continue;
            }
            // Check if a matching .pub file exists (confirms it's a key pair)
            let pub_file = ssh_dir.join(format!("{}.pub", name));
            if pub_file.exists() {
                keys.push(format!("~/.ssh/{}", name));
            }
        }
    }

    keys.sort();
    Ok(keys)
}

/// Read ~/.ssh/config and return the IdentityFile currently configured for
/// the given provider's host, if any. Used by the frontend to auto-detect
/// existing SSH key configuration.
#[tauri::command]
pub fn detect_ssh_key_for_host(provider: String) -> Result<Option<String>, String> {
    let Some(host) = host_for_provider(&provider) else {
        return Ok(None);
    };

    let config_path = dirs::home_dir()
        .ok_or("Could not determine home directory")?
        .join(".ssh")
        .join("config");

    if !config_path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read SSH config: {}", e))?;

    let mut in_target_block = false;
    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("Host ") || trimmed.starts_with("Host\t") {
            let host_value = trimmed.strip_prefix("Host").unwrap().trim();
            in_target_block = host_value == host;
            continue;
        }

        if in_target_block && trimmed.starts_with("IdentityFile ") {
            let path = trimmed.strip_prefix("IdentityFile").unwrap().trim();
            return Ok(Some(path.to_string()));
        }
    }

    Ok(None)
}

/// Test SSH connectivity to a provider's host. Returns true if the SSH
/// key authenticates successfully. Bitbucket and GitHub both return exit
/// code 1 with a success message on `ssh -T`, so we check stderr text.
#[tauri::command]
pub async fn test_ssh_connection(provider: String) -> Result<bool, String> {
    let Some(host) = host_for_provider(&provider) else {
        return Err(format!("Unknown provider: {}", provider));
    };

    let output = Command::new("ssh")
        .args(["-T", "-o", "StrictHostKeyChecking=accept-new", "-o", "ConnectTimeout=10", &format!("git@{}", host)])
        .output()
        .map_err(|e| format!("Failed to run ssh: {}", e))?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
    let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
    let combined = format!("{} {}", stdout, stderr);

    // Both GitHub and Bitbucket print "authenticated" or "logged in as"
    // on successful key auth (but return exit code 1 because shell access
    // is disabled). Check for known success patterns.
    Ok(combined.contains("authenticated") || combined.contains("logged in as") || combined.contains("successfully authenticated"))
}

/// Update the SSH key for an existing account and immediately apply it
/// to ~/.ssh/config for the provider's host.
#[tauri::command]
pub fn update_account_ssh_key(id: String, ssh_key_path: Option<String>) -> Result<(), String> {
    let account = accounts::get_full_account(&id)?;

    // Persist on the account record.
    accounts::update_ssh_key_path(
        &id,
        ssh_key_path.as_deref(),
    )?;

    // Apply immediately to SSH config.
    if let Some(ref key) = ssh_key_path {
        if !key.is_empty() {
            if let Some(host) = host_for_provider(&account.provider) {
                update_ssh_config(host, key)?;
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub fn get_active_git_user() -> Result<GitUser, String> {
    let name = git_config_get("user.name").unwrap_or_default();
    let email = git_config_get("user.email").unwrap_or_default();

    Ok(GitUser { name, email })
}

#[tauri::command]
pub async fn switch_account(id: String) -> Result<(), String> {
    let account = accounts::get_full_account(&id)?;

    // Update commit identity.
    git_config_set("user.name", &account.label)?;
    git_config_set("user.email", &account.email)?;

    // For Bitbucket accounts missing a url_username (pre-existing entries),
    // auto-discover the workspace handle from the API so credential pinning
    // works correctly with URL-embedded usernames.
    let url_username = match (&account.provider as &str, &account.url_username) {
        ("bitbucket", None) => {
            let discovered = crate::validate::fetch_bitbucket_workspace_username(
                &account.username,
                &account.token,
            )
            .await;

            // Persist for future switches so we don't re-fetch every time.
            if discovered.is_some() {
                let _ = accounts::update_url_username(&account.id, discovered.as_deref());
            }
            discovered
        }
        _ => account.url_username.clone(),
    };

    // Update the stored credential and pin git to it. Without the username
    // pin (`credential.<host>.username`), git asks the helper for the host
    // with no username and gets back whatever default credential GCM has —
    // often a stale one from a previous setup. Pinning forces git to ask
    // for *this* account's namespaced credential.
    pin_credential(
        &account.provider,
        &account.username,
        &account.token,
        url_username.as_deref(),
    );

    // Update SSH config if the account has an SSH key configured.
    if let Some(ref ssh_key) = account.ssh_key_path {
        if !ssh_key.is_empty() {
            if let Some(host) = host_for_provider(&account.provider) {
                if let Err(e) = update_ssh_config(host, ssh_key) {
                    eprintln!("Warning: failed to update SSH config: {}", e);
                }
            }
        }
    }

    Ok(())
}
