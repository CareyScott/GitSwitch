import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { queryKeys } from "./queryClient";
import {
  getAccounts,
  addAccount,
  removeAccount,
  getActiveGitUser,
  switchAccount,
  validateAccount,
  type NewAccount,
} from "./tauri";

export function useAccounts() {
  return useQuery({
    queryKey: queryKeys.accounts,
    queryFn: getAccounts,
  });
}

export function useActiveGitUser() {
  return useQuery({
    queryKey: queryKeys.activeGitUser,
    queryFn: getActiveGitUser,
  });
}

export function useAddAccount() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (account: NewAccount) => addAccount(account),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.accounts });
    },
  });
}

export function useRemoveAccount() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => removeAccount(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.accounts });
    },
  });
}

export function useSwitchAccount() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => switchAccount(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.activeGitUser });
    },
  });
}

export function useValidateAccount() {
  return useMutation({
    mutationFn: (id: string) => validateAccount(id),
  });
}
