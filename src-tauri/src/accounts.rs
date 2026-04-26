use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const KEYRING_SERVICE: &str = "git-switch";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Account {
    pub id: String,
    pub provider: String, // "github" | "bitbucket"
    pub label: String,
    pub username: String,
    pub email: String,
    // Tokens live in the OS keychain, not on disk.
    // The field is kept on the struct for legacy migration: accounts.json
    // written by older versions had plaintext tokens here, which we move into
    // the keychain on first read. New writes never serialize this field.
    #[serde(default, skip_serializing)]
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

// ---- keychain helpers ----

fn keyring_set(id: &str, token: &str) -> Result<(), String> {
    keyring::Entry::new(KEYRING_SERVICE, id)
        .map_err(|e| format!("keyring entry failed: {}", e))?
        .set_password(token)
        .map_err(|e| format!("keyring set failed: {}", e))
}

fn keyring_get(id: &str) -> Option<String> {
    keyring::Entry::new(KEYRING_SERVICE, id)
        .ok()?
        .get_password()
        .ok()
}

fn keyring_delete(id: &str) {
    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, id) {
        let _ = entry.delete_credential();
    }
}

// ---- store ----

fn read_store() -> AccountStore {
    let path = store_path();
    migrate_legacy_store(&path);
    if !path.exists() {
        return AccountStore::default();
    }
    let data = fs::read_to_string(&path).unwrap_or_default();
    let mut store: AccountStore = serde_json::from_str(&data).unwrap_or_default();

    // One-time migration: move any plaintext tokens left on disk into the
    // OS keychain. Best-effort — if the keychain is unavailable, leave the
    // token on disk so the user can still validate/switch.
    let mut migrated = false;
    for account in &mut store.accounts {
        if !account.token.is_empty() && keyring_set(&account.id, &account.token).is_ok() {
            account.token.clear();
            migrated = true;
        }
    }
    if migrated {
        let _ = write_store(&store);
    }
    store
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
    let masked = match keyring_get(&account.id) {
        Some(t) => mask_token(&t),
        None => "****".to_string(),
    };
    AccountSafe {
        id: account.id.clone(),
        provider: account.provider.clone(),
        label: account.label.clone(),
        username: account.username.clone(),
        email: account.email.clone(),
        token: masked,
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
    let id = uuid::Uuid::new_v4().to_string();

    // Write secret to keyring first; if that fails we never persist a
    // half-rowed account whose secret is missing.
    keyring_set(&id, &account.token)?;

    let new_account = Account {
        id: id.clone(),
        provider: account.provider,
        label: account.label,
        username: account.username,
        email: account.email,
        token: String::new(),
    };

    let safe = to_safe(&new_account);
    store.accounts.push(new_account);

    if let Err(e) = write_store(&store) {
        // Roll back the orphaned keyring entry so we don't leak secrets.
        keyring_delete(&id);
        return Err(e);
    }

    Ok(safe)
}

#[tauri::command]
pub fn remove_account(id: String) -> Result<(), String> {
    let mut store = read_store();
    let removed = store
        .accounts
        .iter()
        .find(|a| a.id == id)
        .cloned()
        .ok_or_else(|| format!("Account {} not found", id))?;

    store.accounts.retain(|a| a.id != id);
    write_store(&store)?;
    keyring_delete(&id);

    // Best-effort: also tell the git credential helper to forget this user
    // for the provider's host, so a future push doesn't silently reuse it.
    if let Some(host) = crate::git_config::host_for(&removed.provider) {
        let _ = crate::git_config::forget_credential(host, &removed.username);
    }

    Ok(())
}

#[tauri::command]
pub fn get_config_folder() -> Result<String, String> {
    let folder = store_path()
        .parent()
        .ok_or("Could not resolve config folder")?
        .to_path_buf();
    fs::create_dir_all(&folder).map_err(|e| format!("Failed to ensure config dir: {}", e))?;
    Ok(folder.to_string_lossy().into_owned())
}

/// Internal: get the full account with token for validation purposes.
/// Not exposed to the frontend.
pub fn get_full_account(id: &str) -> Result<Account, String> {
    let store = read_store();
    let mut account = store
        .accounts
        .iter()
        .find(|a| a.id == id)
        .cloned()
        .ok_or_else(|| format!("Account {} not found", id))?;
    account.token = keyring_get(id).ok_or_else(|| {
        format!("Token for account {} not found in keychain", id)
    })?;
    Ok(account)
}
