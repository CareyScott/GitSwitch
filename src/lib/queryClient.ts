import { QueryClient } from "@tanstack/react-query";

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30_000,
      gcTime: 5 * 60_000,
      refetchOnWindowFocus: true,
      retry: 1,
    },
  },
});

export const queryKeys = {
  accounts: ["accounts"] as const,
  activeGitUser: ["active-git-user"] as const,
  validation: (id: string) => ["validation", id] as const,
};
