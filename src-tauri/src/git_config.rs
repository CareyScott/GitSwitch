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

// Clear stale entries, store the credential under the real username, and
// pin git to that username for the host. Shared by Switch (which also moves
// commit identity) and Validate (which only ensures GCM is set up for this
// account, without touching git config user.name/user.email).
pub fn pin_credential(provider: &str, username: &str, token: &str) {
    let Some(host) = host_for_provider(provider) else {
        return;
    };
    forget_default_credential(host);
    let _ = git_credential_op("approve", host, Some(username), Some(token));
    let url_key = format!("credential.https://{}.username", host);
    let _ = git_config_set(&url_key, username);
}

pub fn host_for(provider: &str) -> Option<&'static str> {
    host_for_provider(provider)
}

#[tauri::command]
pub fn get_active_git_user() -> Result<GitUser, String> {
    let name = git_config_get("user.name").unwrap_or_default();
    let email = git_config_get("user.email").unwrap_or_default();

    Ok(GitUser { name, email })
}

#[tauri::command]
pub fn switch_account(id: String) -> Result<(), String> {
    let account = accounts::get_full_account(&id)?;

    // Update commit identity.
    git_config_set("user.name", &account.label)?;
    git_config_set("user.email", &account.email)?;

    // Update the stored credential and pin git to it. Without the username
    // pin (`credential.<host>.username`), git asks the helper for the host
    // with no username and gets back whatever default credential GCM has —
    // often a stale one from a previous setup. Pinning forces git to ask
    // for *this* account's namespaced credential.
    pin_credential(&account.provider, &account.username, &account.token);

    Ok(())
}
