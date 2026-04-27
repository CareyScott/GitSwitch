use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::accounts;

#[derive(Serialize, Clone, Debug)]
pub struct ValidationResult {
    pub valid: bool,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub url_username: Option<String>,
    pub error: Option<String>,
}

// ---- GitHub API response (partial) ----
#[derive(Deserialize)]
struct GitHubUser {
    login: Option<String>,
    name: Option<String>,
    avatar_url: Option<String>,
}

// ---- Bitbucket API response (partial) ----
#[derive(Deserialize)]
struct BitbucketUser {
    display_name: Option<String>,
    username: Option<String>,
    links: Option<BitbucketLinks>,
}

#[derive(Deserialize)]
struct BitbucketLinks {
    avatar: Option<BitbucketHref>,
}

#[derive(Deserialize)]
struct BitbucketHref {
    href: Option<String>,
}

#[tauri::command]
pub async fn validate_github(username: String, token: String) -> Result<ValidationResult, String> {
    let _ = username; // GitHub PATs don't require username for /user endpoint

    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "git-switch-app")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        return Ok(ValidationResult {
            valid: false,
            display_name: None,
            avatar_url: None,
            url_username: None,
            error: Some(format!("HTTP {} — {}", status, body)),
        });
    }

    let user: GitHubUser = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(ValidationResult {
        valid: true,
        display_name: user.name.or(user.login),
        avatar_url: user.avatar_url,
        url_username: None,
        error: None,
    })
}

#[tauri::command]
pub async fn validate_bitbucket(
    username: String,
    token: String,
) -> Result<ValidationResult, String> {
    let client = reqwest::Client::new();
    let credentials = format!("{}:{}", username, token);
    let encoded = base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());

    let resp = client
        .get("https://api.bitbucket.org/2.0/user")
        .header("Authorization", format!("Basic {}", encoded))
        .header("User-Agent", "git-switch-app")
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        return Ok(ValidationResult {
            valid: false,
            display_name: None,
            avatar_url: None,
            url_username: None,
            error: Some(format!("HTTP {} — {}", status, body)),
        });
    }

    let user: BitbucketUser = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let avatar = user.links.and_then(|l| l.avatar).and_then(|a| a.href);

    // The API returns the workspace handle as `username`. If it differs from
    // the auth username (email), it's the URL-embedded username that repos use.
    let url_username = user.username.clone();

    Ok(ValidationResult {
        valid: true,
        display_name: user.display_name.or(user.username),
        avatar_url: avatar,
        url_username,
        error: None,
    })
}

/// Fetch the Bitbucket workspace username (handle) for a given set of
/// credentials. Returns `None` if the API call fails. This is the username
/// that appears in clone URLs (e.g., `scott_careyy` in
/// `https://scott_careyy@bitbucket.org/...`).
pub async fn fetch_bitbucket_workspace_username(
    username: &str,
    token: &str,
) -> Option<String> {
    let client = reqwest::Client::new();
    let credentials = format!("{}:{}", username, token);
    let encoded = base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());

    let resp = client
        .get("https://api.bitbucket.org/2.0/user")
        .header("Authorization", format!("Basic {}", encoded))
        .header("User-Agent", "git-switch-app")
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let user: BitbucketUser = resp.json().await.ok()?;
    user.username
}

/// Validate a stored account by its ID. On a successful validation, also
/// clears any stale GCM entries for the host and pins git to use this
/// account's namespaced credential. This is the "fix this account if needed"
/// flow — Switch also does this, but Validate lets the user run it without
/// touching their commit identity (`user.name` / `user.email`).
#[tauri::command]
pub async fn validate_account(id: String) -> Result<ValidationResult, String> {
    let account = accounts::get_full_account(&id)?;

    let result = match account.provider.as_str() {
        "github" => validate_github(account.username.clone(), account.token.clone()).await?,
        "bitbucket" => {
            validate_bitbucket(account.username.clone(), account.token.clone()).await?
        }
        other => {
            return Ok(ValidationResult {
                valid: false,
                display_name: None,
                avatar_url: None,
                url_username: None,
                error: Some(format!("Unknown provider: {}", other)),
            });
        }
    };

    if result.valid {
        // Use the discovered URL username (Bitbucket workspace handle) if available,
        // falling back to whatever was already stored on the account.
        let effective_url_username = result.url_username.clone()
            .or(account.url_username.clone());

        crate::git_config::pin_credential(
            &account.provider,
            &account.username,
            &account.token,
            effective_url_username.as_deref(),
        );

        // Persist the discovered url_username on the account for future switches.
        if result.url_username.is_some() && result.url_username != account.url_username {
            let _ = crate::accounts::update_url_username(&account.id, result.url_username.as_deref());
        }
    }

    Ok(result)
}
