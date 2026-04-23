use serde::Serialize;
use std::process::Command;

#[derive(Serialize, Clone, Debug)]
pub struct GitUser {
    pub name: String,
    pub email: String,
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

#[tauri::command]
pub fn get_active_git_user() -> Result<GitUser, String> {
    let name = git_config_get("user.name").unwrap_or_default();
    let email = git_config_get("user.email").unwrap_or_default();

    Ok(GitUser { name, email })
}

#[tauri::command]
pub fn switch_account(name: String, email: String) -> Result<(), String> {
    git_config_set("user.name", &name)?;
    git_config_set("user.email", &email)?;
    Ok(())
}
