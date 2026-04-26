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
export const switchAccount = (id: string) =>
  invoke<void>("switch_account", { id });
export const validateGithub = (username: string, token: string) =>
  invoke<ValidationResult>("validate_github", { username, token });
export const validateBitbucket = (username: string, token: string) =>
  invoke<ValidationResult>("validate_bitbucket", { username, token });
export const validateAccount = (id: string) =>
  invoke<ValidationResult>("validate_account", { id });
export const getConfigFolder = () => invoke<string>("get_config_folder");

export interface DeviceFlowStart {
  device_code: string;
  user_code: string;
  verification_uri: string;
  expires_in: number;
  interval: number;
}

export type DeviceFlowPoll =
  | { status: "pending" }
  | { status: "slow_down" }
  | { status: "expired" }
  | { status: "denied" }
  | {
      status: "success";
      access_token: string;
      username: string;
      email: string;
      display_name: string | null;
    }
  | { status: "error"; message: string };

export const githubDeviceStart = () =>
  invoke<DeviceFlowStart>("github_device_start");
export const githubDevicePoll = (device_code: string) =>
  invoke<DeviceFlowPoll>("github_device_poll", { deviceCode: device_code });
