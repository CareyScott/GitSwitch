mod accounts;
mod git_config;
mod shell_env;
mod validate;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // GUI apps launched from Dock/Finder don't inherit the user's shell
    // environment. Hydrate PATH so `git` commands work.
    shell_env::hydrate_from_login_shell();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            accounts::get_accounts,
            accounts::add_account,
            accounts::remove_account,
            git_config::get_active_git_user,
            git_config::switch_account,
            validate::validate_github,
            validate::validate_bitbucket,
            validate::validate_account,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
