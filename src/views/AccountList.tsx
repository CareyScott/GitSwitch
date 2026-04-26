import { useState } from "react";
import { Plus, Users, Import } from "lucide-react";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/common/EmptyState";
import { useAccounts, useActiveGitUser } from "@/lib/accounts";
import { AccountCard } from "./AccountCard";
import { DetectedAccountCard } from "./DetectedAccountCard";
import { AddAccountDialog } from "./AddAccountDialog";

export function AccountList() {
  const { data: accounts, isLoading } = useAccounts();
  const { data: gitUser } = useActiveGitUser();
  const [addOpen, setAddOpen] = useState(false);
  const [prefill, setPrefill] = useState<{ name?: string; email?: string } | undefined>();

  // Check if the current git identity is already tracked as a stored account
  const currentIsTracked =
    accounts &&
    accounts.length > 0 &&
    gitUser &&
    (gitUser.name || gitUser.email) &&
    accounts.some(
      (a) => a.email === gitUser.email || a.label === gitUser.name,
    );

  const hasDetectedAccount =
    gitUser && (gitUser.name || gitUser.email) && !currentIsTracked;

  return (
    <div className="flex flex-1 flex-col min-h-0">
      {/* Header — always visible */}
      <div className="mb-4 flex shrink-0 items-center justify-between">
        <div>
          <h1 className="text-page">Accounts</h1>
          {accounts && accounts.length > 0 && (
            <p className="mt-0.5 text-sm text-fg-muted">
              {accounts.length} account{accounts.length !== 1 ? "s" : ""}{" "}
              configured
            </p>
          )}
        </div>
        <Button
          size="sm"
          onClick={() => {
            setPrefill(undefined);
            setAddOpen(true);
          }}
          className="titlebar-nodrag"
        >
          <Plus className="mr-1 h-3.5 w-3.5" />
          Add Account
        </Button>
      </div>

      {/* Scrollable body — detected banner + cards */}
      <div className="flex-1 overflow-y-auto [scrollbar-gutter:stable]">
        {/* Detected unmanaged account from git config */}
        {hasDetectedAccount && (
          <div className="mb-4">
            <p className="text-label mb-2 flex items-center gap-1.5">
              <Import className="h-3 w-3" />
              Detected from git config
            </p>
            <DetectedAccountCard
              name={gitUser.name}
              email={gitUser.email}
              onAdd={(p) => {
                setPrefill(p);
                setAddOpen(true);
              }}
            />
          </div>
        )}

        {/* Stored account list */}
        {isLoading ? (
          <div className="space-y-2">
            {[1, 2].map((i) => (
              <div
                key={i}
                className="card h-[72px] animate-pulse"
              />
            ))}
          </div>
        ) : accounts && accounts.length > 0 ? (
          <div className="space-y-2 pb-1">
            {accounts.map((account) => (
              <AccountCard
                key={account.id}
                account={account}
                isActive={
                  gitUser?.email === account.email ||
                  gitUser?.name === account.label
                }
              />
            ))}
          </div>
        ) : !hasDetectedAccount ? (
          <EmptyState
            icon={<Users className="h-6 w-6" />}
            title="No accounts configured"
            description="Add your GitHub or Bitbucket accounts to switch between them with a single click."
            action={
              <Button size="sm" onClick={() => setAddOpen(true)}>
                <Plus className="mr-1 h-3.5 w-3.5" />
                Add your first account
              </Button>
            }
          />
        ) : null}
      </div>

      {/* Add dialog */}
      <AddAccountDialog
        open={addOpen}
        onOpenChange={(v) => {
          setAddOpen(v);
          if (!v) setPrefill(undefined);
        }}
        prefill={prefill}
      />
    </div>
  );
}
