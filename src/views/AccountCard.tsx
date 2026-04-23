import { useState } from "react";
import {
  MoreHorizontal,
  CheckCircle2,
  XCircle,
  Loader2,
  ArrowRightLeft,
  Trash2,
  ShieldCheck,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
} from "@/components/ui/dropdown-menu";
import {
  useSwitchAccount,
  useRemoveAccount,
  useValidateAccount,
} from "@/lib/accounts";
import type { Account, ValidationResult } from "@/lib/tauri";

// Inline SVG icons for provider logos
function GithubIcon({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 24 24" fill="currentColor" className={className}>
      <path d="M12 2C6.477 2 2 6.477 2 12c0 4.42 2.865 8.17 6.839 9.49.5.092.682-.217.682-.482 0-.237-.008-.866-.013-1.7-2.782.604-3.369-1.34-3.369-1.34-.454-1.156-1.11-1.464-1.11-1.464-.908-.62.069-.608.069-.608 1.003.07 1.531 1.03 1.531 1.03.892 1.529 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.11-4.555-4.943 0-1.091.39-1.984 1.029-2.683-.103-.253-.446-1.27.098-2.647 0 0 .84-.269 2.75 1.025A9.578 9.578 0 0 1 12 6.836c.85.004 1.705.115 2.504.337 1.909-1.294 2.747-1.025 2.747-1.025.546 1.377.203 2.394.1 2.647.64.699 1.028 1.592 1.028 2.683 0 3.842-2.339 4.687-4.566 4.935.359.309.678.919.678 1.852 0 1.336-.012 2.415-.012 2.743 0 .267.18.578.688.48C19.138 20.167 22 16.418 22 12c0-5.523-4.477-10-10-10z" />
    </svg>
  );
}

function BitbucketIcon({ className }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="currentColor"
      className={className}
    >
      <path d="M3.27 4.08c-.2 0-.37.17-.34.38l2.66 16.14c.04.24.25.42.5.42h12.06c.18 0 .34-.13.37-.31L21.07 4.45a.34.34 0 0 0-.34-.38H3.27Zm10.55 11.54H10.2l-.95-4.97h5.57l-1 4.97Z" />
    </svg>
  );
}

export function AccountCard({
  account,
  isActive,
}: {
  account: Account;
  isActive: boolean;
}) {
  const switchMutation = useSwitchAccount();
  const removeMutation = useRemoveAccount();
  const validateMutation = useValidateAccount();
  const [validation, setValidation] = useState<ValidationResult | null>(null);

  const isGithub = account.provider === "github";
  const ProviderIcon = isGithub ? GithubIcon : BitbucketIcon;

  const handleSwitch = () => {
    switchMutation.mutate({ name: account.label, email: account.email });
  };

  const handleRemove = () => {
    removeMutation.mutate(account.id);
  };

  const handleValidate = async () => {
    setValidation(null);
    const result = await validateMutation.mutateAsync(account.id);
    setValidation(result);
  };

  return (
    <div
      className={`card row-hover titlebar-nodrag flex items-center gap-3 px-4 py-3 ${
        isActive
          ? "border-success/30 bg-success/[0.04]"
          : ""
      }`}
    >
      {/* Provider icon */}
      <div
        className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg"
        style={{
          background: isGithub
            ? "color-mix(in srgb, #e4e7ec 10%, transparent)"
            : "color-mix(in srgb, #2684FF 12%, transparent)",
        }}
      >
        <ProviderIcon
          className={`h-4.5 w-4.5 ${isGithub ? "text-fg-default" : "text-[#2684FF]"}`}
        />
      </div>

      {/* Account details */}
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <span className="truncate text-sm font-medium text-fg-default">
            {account.label}
          </span>
          <Badge
            className="shrink-0 text-[10px]"
            style={{
              borderColor: isGithub
                ? "color-mix(in srgb, #e4e7ec 20%, transparent)"
                : "color-mix(in srgb, #2684FF 25%, transparent)",
              color: isGithub ? "var(--color-fg-muted)" : "#5fa3f5",
            }}
          >
            {isGithub ? "GitHub" : "Bitbucket"}
          </Badge>
          {isActive && (
            <Badge className="shrink-0 border-success/30 bg-success/10 text-success text-[10px]">
              Active
            </Badge>
          )}
        </div>
        <div className="mt-0.5 flex min-w-0 items-center gap-2 text-xs text-fg-muted">
          <span className="shrink-0">@{account.username}</span>
          <span className="truncate text-fg-subtle">{account.email}</span>
        </div>
      </div>

      {/* Validation indicator */}
      {validateMutation.isPending && (
        <Loader2 className="h-4 w-4 shrink-0 animate-spin text-fg-subtle" />
      )}
      {validation && !validateMutation.isPending && (
        <span title={validation.error ?? validation.display_name ?? ""}>
          {validation.valid ? (
            <CheckCircle2 className="h-4 w-4 text-success" />
          ) : (
            <XCircle className="h-4 w-4 text-danger" />
          )}
        </span>
      )}

      {/* Switch button */}
      {!isActive && (
        <Button
          variant="outline"
          size="sm"
          onClick={handleSwitch}
          disabled={switchMutation.isPending}
          className="shrink-0"
        >
          <ArrowRightLeft className="mr-1 h-3 w-3" />
          Switch
        </Button>
      )}

      {/* More actions */}
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" size="icon" className="h-7 w-7 shrink-0">
            <MoreHorizontal className="h-4 w-4" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <DropdownMenuItem onClick={handleValidate}>
            <ShieldCheck className="h-3.5 w-3.5" />
            Validate credentials
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem
            onClick={handleRemove}
            className="text-danger focus:text-danger"
          >
            <Trash2 className="h-3.5 w-3.5" />
            Remove account
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
}
