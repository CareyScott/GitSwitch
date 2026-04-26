// GitHub Device Flow — for desktop apps that can't safely hold an OAuth
// client secret. The user pastes a short code into github.com/login/device,
// the app polls until they approve, then receives an access token.
//
// Docs: https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps#device-flow

use serde::{Deserialize, Serialize};

// Public client identifier for the GitSwitch_OAuth app on github.com.
// Safe to commit — device flow doesn't use a client secret. Anyone forking
// this repo can either reuse this ID (subject to its rate limits) or register
// their own OAuth App with Device Flow enabled and replace it.
const GITHUB_CLIENT_ID: Option<&str> = Some("Ov23liPCroas5Zmo9GxQ");

// `repo` lets the token be used by `git push` / `git clone` against private
// repos. Without it we'd only have an identity token, which doesn't actually
// solve the multi-account problem when GCM caches a different user's token.
const SCOPE: &str = "read:user user:email repo";
const USER_AGENT: &str = "git-switch-app";

// Minimal application/x-www-form-urlencoded encoder for the chars our
// payloads actually contain (colons, spaces, dots, slashes, hyphens).
// reqwest's `.form()` would do this for us but isn't enabled in our build.
fn urlencode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for b in input.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push_str(&format!("%{:02X}", b));
            }
        }
    }
    out
}

#[derive(Serialize, Clone, Debug)]
pub struct DeviceFlowStart {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DeviceFlowPoll {
    Pending,
    SlowDown,
    Expired,
    Denied,
    Success {
        access_token: String,
        username: String,
        email: String,
        display_name: Option<String>,
    },
    Error {
        message: String,
    },
}

// ---- Step 1: kick off the flow ----

#[derive(Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
}

#[tauri::command]
pub async fn github_device_start() -> Result<DeviceFlowStart, String> {
    let client_id = GITHUB_CLIENT_ID
        .ok_or("GITHUB_CLIENT_ID not configured at build time. See src-tauri/src/oauth.rs.")?;

    let body = format!(
        "client_id={}&scope={}",
        urlencode(client_id),
        urlencode(SCOPE)
    );
    let resp = reqwest::Client::new()
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("User-Agent", USER_AGENT)
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("GitHub returned HTTP {}: {}", status, body));
    }

    let parsed: DeviceCodeResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse device-code response: {}", e))?;

    Ok(DeviceFlowStart {
        device_code: parsed.device_code,
        user_code: parsed.user_code,
        verification_uri: parsed.verification_uri,
        expires_in: parsed.expires_in,
        interval: parsed.interval,
    })
}

// ---- Step 2: poll for token (called repeatedly by the frontend) ----

#[derive(Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    error: Option<String>,
}

#[tauri::command]
pub async fn github_device_poll(device_code: String) -> Result<DeviceFlowPoll, String> {
    let client_id = GITHUB_CLIENT_ID
        .ok_or("GITHUB_CLIENT_ID not configured at build time. See src-tauri/src/oauth.rs.")?;

    let body = format!(
        "client_id={}&device_code={}&grant_type={}",
        urlencode(client_id),
        urlencode(device_code.as_str()),
        urlencode("urn:ietf:params:oauth:grant-type:device_code")
    );
    let resp = reqwest::Client::new()
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("User-Agent", USER_AGENT)
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let parsed: TokenResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    if let Some(err) = parsed.error.as_deref() {
        return Ok(match err {
            "authorization_pending" => DeviceFlowPoll::Pending,
            "slow_down" => DeviceFlowPoll::SlowDown,
            "expired_token" => DeviceFlowPoll::Expired,
            "access_denied" => DeviceFlowPoll::Denied,
            other => DeviceFlowPoll::Error {
                message: other.to_string(),
            },
        });
    }

    let token = parsed
        .access_token
        .ok_or("Missing access_token in success response")?;

    // Fetch user identity so we can pre-fill name/email in the dialog.
    let (username, display_name) = fetch_user(&token).await?;
    let email = fetch_primary_email(&token).await.unwrap_or_default();

    Ok(DeviceFlowPoll::Success {
        access_token: token,
        username,
        email,
        display_name,
    })
}

// ---- Helpers: read user identity ----

#[derive(Deserialize)]
struct GhUser {
    login: String,
    name: Option<String>,
}

async fn fetch_user(token: &str) -> Result<(String, Option<String>), String> {
    let resp = reqwest::Client::new()
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("GitHub /user returned HTTP {}", resp.status()));
    }

    let user: GhUser = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse /user response: {}", e))?;

    Ok((user.login.clone(), user.name.or(Some(user.login))))
}

#[derive(Deserialize)]
struct GhEmail {
    email: String,
    primary: bool,
    verified: bool,
}

async fn fetch_primary_email(token: &str) -> Result<String, String> {
    let resp = reqwest::Client::new()
        .get("https://api.github.com/user/emails")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("GitHub /user/emails returned HTTP {}", resp.status()));
    }

    let emails: Vec<GhEmail> = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse /user/emails response: {}", e))?;

    emails
        .into_iter()
        .find(|e| e.primary && e.verified)
        .map(|e| e.email)
        .ok_or_else(|| "No primary verified email found".to_string())
}
