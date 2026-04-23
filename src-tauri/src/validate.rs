use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::accounts;

#[derive(Serialize, Clone, Debug)]
pub struct ValidationResult {
    pub valid: bool,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
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
            error: Some(format!("HTTP {} — {}", status, body)),
        });
    }

    let user: BitbucketUser = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let avatar = user.links.and_then(|l| l.avatar).and_then(|a| a.href);

    Ok(ValidationResult {
        valid: true,
        display_name: user.display_name.or(user.username),
        avatar_url: avatar,
        error: None,
    })
}

/// Validate a stored account by its ID — reads the full token from disk,
/// never exposes it to the frontend.
#[tauri::command]
pub async fn validate_account(id: String) -> Result<ValidationResult, String> {
    let account = accounts::get_full_account(&id)?;

    match account.provider.as_str() {
        "github" => validate_github(account.username, account.token).await,
        "bitbucket" => validate_bitbucket(account.username, account.token).await,
        other => Ok(ValidationResult {
            valid: false,
            display_name: None,
            avatar_url: None,
            error: Some(format!("Unknown provider: {}", other)),
        }),
    }
}
