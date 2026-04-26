import { useState, useEffect } from "react";
import {
  CheckCircle2,
  XCircle,
  Loader2,
} from "lucide-react";
import { DeviceFlowDialog } from "./DeviceFlowDialog";
import {
  Dialog,
  DialogContent,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useAddAccount } from "@/lib/accounts";
import {
  validateGithub,
  validateBitbucket,
  type ValidationResult,
  type NewAccount,
} from "@/lib/tauri";

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
    <svg viewBox="0 0 24 24" fill="currentColor" className={className}>
      <path d="M3.27 4.08c-.2 0-.37.17-.34.38l2.66 16.14c.04.24.25.42.5.42h12.06c.18 0 .34-.13.37-.31L21.07 4.45a.34.34 0 0 0-.34-.38H3.27Zm10.55 11.54H10.2l-.95-4.97h5.57l-1 4.97Z" />
    </svg>
  );
}

export function AddAccountDialog({
  open,
  onOpenChange,
  prefill,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  prefill?: { name?: string; email?: string };
}) {
  const addMutation = useAddAccount();

  const [provider, setProvider] = useState<"github" | "bitbucket">("github");
  const [label, setLabel] = useState("");
  const [username, setUsername] = useState("");
  const [email, setEmail] = useState("");
  const [token, setToken] = useState("");

  const [validating, setValidating] = useState(false);
  const [validation, setValidation] = useState<ValidationResult | null>(null);
  const [deviceOpen, setDeviceOpen] = useState(false);

  const isGithub = provider === "github";

  // Apply prefill values whenever the dialog opens.
  useEffect(() => {
    if (open && prefill) {
      if (prefill.name) setLabel(prefill.name);
      if (prefill.email) setEmail(prefill.email);
    }
  }, [open, prefill]);

  const resetForm = () => {
    setProvider("github");
    setLabel("");
    setUsername("");
    setEmail("");
    setToken("");
    setValidation(null);
  };

  const handleTest = async () => {
    setValidating(true);
    setValidation(null);
    try {
      const result = isGithub
        ? await validateGithub(username, token)
        : await validateBitbucket(username, token);
      setValidation(result);
      // Auto-fill label from display name if empty
      if (result.valid && result.display_name && !label) {
        setLabel(result.display_name);
      }
    } catch (err) {
      setValidation({
        valid: false,
        display_name: null,
        avatar_url: null,
        error: String(err),
      });
    } finally {
      setValidating(false);
    }
  };

  const handleSubmit = async () => {
    const account: NewAccount = {
      provider,
      label: label || username,
      username,
      email,
      token,
    };
    await addMutation.mutateAsync(account);
    resetForm();
    onOpenChange(false);
  };

  const canTest = username.trim() && token.trim();
  const canSubmit = username.trim() && email.trim() && token.trim();

  return (
    <Dialog
      open={open}
      onOpenChange={(v) => {
        if (!v) resetForm();
        onOpenChange(v);
      }}
    >
      <DialogContent className="max-w-[440px]">
        <DialogTitle className="text-section">Add Account</DialogTitle>
        <DialogDescription className="sr-only">
          Add a new git account
        </DialogDescription>

        <div className="mt-4 space-y-4">
          {/* Provider picker */}
          <div>
            <p className="text-label mb-2">Provider</p>
            <div className="flex gap-2">
              <button
                type="button"
                onClick={() => {
                  setProvider("github");
                  setValidation(null);
                }}
                className={`flex flex-1 items-center justify-center gap-2 rounded-md border px-3 py-2 text-sm font-medium transition-colors ${
                  isGithub
                    ? "border-accent/40 bg-accent/[0.08] text-fg-default"
                    : "border-border-default bg-transparent text-fg-muted hover:bg-bg-elevated"
                }`}
              >
                <GithubIcon className="h-4 w-4" />
                GitHub
              </button>
              <button
                type="button"
                onClick={() => {
                  setProvider("bitbucket");
                  setValidation(null);
                }}
                className={`flex flex-1 items-center justify-center gap-2 rounded-md border px-3 py-2 text-sm font-medium transition-colors ${
                  !isGithub
                    ? "border-[#2684FF]/40 bg-[#2684FF]/[0.08] text-fg-default"
                    : "border-border-default bg-transparent text-fg-muted hover:bg-bg-elevated"
                }`}
              >
                <BitbucketIcon className="h-4 w-4" />
                Bitbucket
              </button>
            </div>
          </div>

          {/* GitHub device-flow shortcut */}
          {isGithub && (
            <>
              <Button
                variant="outline"
                className="w-full"
                size="sm"
                onClick={() => setDeviceOpen(true)}
              >
                <GithubIcon className="mr-1.5 h-3.5 w-3.5" />
                Sign in with GitHub
              </Button>
              <div className="flex items-center gap-3 text-[11px] uppercase tracking-wider text-fg-subtle">
                <div className="h-px flex-1 bg-border-default" />
                or paste a token
                <div className="h-px flex-1 bg-border-default" />
              </div>
            </>
          )}

          {/* Display name */}
          <div>
            <label className="text-label mb-1.5 block">
              Display Name
              <span className="ml-1 normal-case text-fg-subtle">(optional)</span>
            </label>
            <Input
              value={label}
              onChange={(e) => setLabel(e.target.value)}
              placeholder={isGithub ? "e.g. Personal" : "e.g. Work"}
            />
          </div>

          {/* Username */}
          <div>
            <label className="text-label mb-1.5 block">Username</label>
            <Input
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              placeholder={
                isGithub ? "GitHub username" : "Bitbucket username"
              }
            />
          </div>

          {/* Email */}
          <div>
            <label className="text-label mb-1.5 block">Email</label>
            <Input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder="Used for git commits"
            />
          </div>

          {/* Token */}
          <div>
            <label className="text-label mb-1.5 block">
              {isGithub ? "Personal Access Token" : "App Password"}
            </label>
            <Input
              type="password"
              value={token}
              onChange={(e) => setToken(e.target.value)}
              placeholder={
                isGithub ? "ghp_xxxxxxxxxxxx" : "App password from Bitbucket settings"
              }
            />
            <p className="mt-1 text-[11px] text-fg-subtle">
              {isGithub
                ? "Generate at GitHub Settings > Developer settings > Personal access tokens"
                : "Generate at Bitbucket Settings > App passwords"}
            </p>
          </div>

          {/* Test credentials */}
          <div className="flex items-center gap-3">
            <Button
              variant="outline"
              size="sm"
              onClick={handleTest}
              disabled={!canTest || validating}
            >
              {validating && <Loader2 className="mr-1 h-3 w-3 animate-spin" />}
              Test Credentials
            </Button>

            {validation && !validating && (
              <span className="flex items-center gap-1.5 text-xs">
                {validation.valid ? (
                  <>
                    <CheckCircle2 className="h-3.5 w-3.5 text-success" />
                    <span className="text-success">
                      {validation.display_name ?? "Valid"}
                    </span>
                  </>
                ) : (
                  <>
                    <XCircle className="h-3.5 w-3.5 text-danger" />
                    <span className="text-danger">
                      {validation.error ?? "Invalid credentials"}
                    </span>
                  </>
                )}
              </span>
            )}
          </div>

          {/* Submit */}
          <div className="flex justify-end gap-2 pt-2">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => {
                resetForm();
                onOpenChange(false);
              }}
            >
              Cancel
            </Button>
            <Button
              size="sm"
              onClick={handleSubmit}
              disabled={!canSubmit || addMutation.isPending}
            >
              {addMutation.isPending && (
                <Loader2 className="mr-1 h-3 w-3 animate-spin" />
              )}
              Add Account
            </Button>
          </div>
        </div>
      </DialogContent>

      <DeviceFlowDialog
        open={deviceOpen}
        onOpenChange={setDeviceOpen}
        onSuccess={({ access_token, username: u, email: e, display_name }) => {
          setProvider("github");
          setUsername(u);
          setEmail(e);
          setToken(access_token);
          if (display_name && !label) setLabel(display_name);
          setValidation({
            valid: true,
            display_name: display_name ?? u,
            avatar_url: null,
            error: null,
          });
        }}
      />
    </Dialog>
  );
}
