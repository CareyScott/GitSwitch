use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Account {
    pub id: String,
    pub provider: String, // "github" | "bitbucket"
    pub label: String,
    pub username: String,
    pub email: String,
    pub token: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct AccountSafe {
    pub id: String,
    pub provider: String,
    pub label: String,
    pub username: String,
    pub email: String,
    pub token: String, // masked
}

#[derive(Deserialize)]
pub struct NewAccount {
    pub provider: String,
    pub label: String,
    pub username: String,
    pub email: String,
    pub token: String,
}

#[derive(Serialize, Deserialize, Default)]
struct AccountStore {
    accounts: Vec<Account>,
}

fn store_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap().join(".config"));
    config_dir.join("git-switch").join("accounts.json")
}

/// If the new store doesn't exist but the old `git-accounts` store does,
/// migrate it automatically (one-time, silent).
fn migrate_legacy_store(new_path: &PathBuf) {
    if new_path.exists() {
        return;
    }
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap().join(".config"));
    let legacy_path = config_dir.join("git-accounts").join("accounts.json");
    if legacy_path.exists() {
        if let Some(parent) = new_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::copy(&legacy_path, new_path);
    }
}

fn read_store() -> AccountStore {
    let path = store_path();
    migrate_legacy_store(&path);
    if !path.exists() {
        return AccountStore::default();
    }
    let data = fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&data).unwrap_or_default()
}

fn write_store(store: &AccountStore) -> Result<(), String> {
    let path = store_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(store)
        .map_err(|e| format!("Failed to serialize: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write store: {}", e))?;
    Ok(())
}

fn mask_token(token: &str) -> String {
    if token.len() <= 4 {
        return "****".to_string();
    }
    let last4 = &token[token.len() - 4..];
    format!("****{}", last4)
}

fn to_safe(account: &Account) -> AccountSafe {
    AccountSafe {
        id: account.id.clone(),
        provider: account.provider.clone(),
        label: account.label.clone(),
        username: account.username.clone(),
        email: account.email.clone(),
        token: mask_token(&account.token),
    }
}

#[tauri::command]
pub fn get_accounts() -> Result<Vec<AccountSafe>, String> {
    let store = read_store();
    Ok(store.accounts.iter().map(to_safe).collect())
}

#[tauri::command]
pub fn add_account(account: NewAccount) -> Result<AccountSafe, String> {
    let mut store = read_store();

    let new_account = Account {
        id: uuid::Uuid::new_v4().to_string(),
        provider: account.provider,
        label: account.label,
        username: account.username,
        email: account.email,
        token: account.token,
    };

    let safe = to_safe(&new_account);
    store.accounts.push(new_account);
    write_store(&store)?;

    Ok(safe)
}

#[tauri::command]
pub fn remove_account(id: String) -> Result<(), String> {
    let mut store = read_store();
    let initial_len = store.accounts.len();
    store.accounts.retain(|a| a.id != id);

    if store.accounts.len() == initial_len {
        return Err(format!("Account {} not found", id));
    }

    write_store(&store)?;
    Ok(())
}

/// Internal: get the full account with token for validation purposes.
/// Not exposed to the frontend.
pub fn get_full_account(id: &str) -> Result<Account, String> {
    let store = read_store();
    store
        .accounts
        .iter()
        .find(|a| a.id == id)
        .cloned()
        .ok_or_else(|| format!("Account {} not found", id))
}
