import { invoke } from "@tauri-apps/api/core";

export interface Account {
  id: string;
  provider: "github" | "bitbucket";
  label: string;
  username: string;
  email: string;
  token: string; // masked from backend
}

export interface GitUser {
  name: string;
  email: string;
}

export interface ValidationResult {
  valid: boolean;
  display_name: string | null;
  avatar_url: string | null;
  error: string | null;
}

export interface NewAccount {
  provider: "github" | "bitbucket";
  label: string;
  username: string;
  email: string;
  token: string;
}

export const getAccounts = () => invoke<Account[]>("get_accounts");
export const addAccount = (account: NewAccount) =>
  invoke<Account>("add_account", { account });
export const removeAccount = (id: string) =>
  invoke<void>("remove_account", { id });
export const getActiveGitUser = () => invoke<GitUser>("get_active_git_user");
export const switchAccount = (name: string, email: string) =>
  invoke<void>("switch_account", { name, email });
export const validateGithub = (username: string, token: string) =>
  invoke<ValidationResult>("validate_github", { username, token });
export const validateBitbucket = (username: string, token: string) =>
  invoke<ValidationResult>("validate_bitbucket", { username, token });
export const validateAccount = (id: string) =>
  invoke<ValidationResult>("validate_account", { id });
