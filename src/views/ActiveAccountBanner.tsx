import { User, Mail } from "lucide-react";
import { useActiveGitUser } from "@/lib/accounts";

export function ActiveAccountBanner() {
  const { data: gitUser, isLoading } = useActiveGitUser();

  if (isLoading) {
    return (
      <div className="card mb-5 px-4 py-3">
        <div className="flex items-center gap-3">
          <div className="h-4 w-32 animate-pulse rounded bg-bg-elevated" />
        </div>
      </div>
    );
  }

  const hasUser = gitUser?.name || gitUser?.email;

  return (
    <div className="hairline-top card mb-5 px-4 py-3.5">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          {/* Status dot */}
          <span
            className="inline-block h-2 w-2 shrink-0 rounded-full"
            style={{
              backgroundColor: hasUser
                ? "var(--color-success)"
                : "var(--color-fg-subtle)",
              boxShadow: hasUser
                ? "0 0 8px var(--color-success)"
                : "none",
            }}
          />

          <div className="min-w-0">
            <p className="text-label mb-0.5">Active Git Identity</p>
            {hasUser ? (
              <div className="flex items-center gap-4">
                <span className="flex items-center gap-1.5 text-sm font-medium text-fg-default">
                  <User className="h-3.5 w-3.5 text-fg-muted" />
                  {gitUser?.name || "—"}
                </span>
                <span className="flex items-center gap-1.5 text-sm text-fg-muted">
                  <Mail className="h-3.5 w-3.5 text-fg-subtle" />
                  {gitUser?.email || "—"}
                </span>
              </div>
            ) : (
              <p className="text-sm text-fg-subtle">
                No git identity configured
              </p>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
