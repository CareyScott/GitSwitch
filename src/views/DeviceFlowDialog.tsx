import { useEffect, useRef, useState } from "react";
import { CheckCircle2, XCircle, Loader2, Copy, ExternalLink } from "lucide-react";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  Dialog,
  DialogContent,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import {
  githubDeviceStart,
  githubDevicePoll,
  type DeviceFlowStart,
} from "@/lib/tauri";

type Phase =
  | { kind: "starting" }
  | { kind: "waiting"; flow: DeviceFlowStart }
  | { kind: "success" }
  | { kind: "error"; message: string };

interface SuccessPayload {
  access_token: string;
  username: string;
  email: string;
  display_name: string | null;
}

export function DeviceFlowDialog({
  open,
  onOpenChange,
  onSuccess,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess: (payload: SuccessPayload) => void;
}) {
  const [phase, setPhase] = useState<Phase>({ kind: "starting" });
  const [copied, setCopied] = useState(false);
  // Track the active polling loop so a re-open or unmount cancels the old one.
  const cancelRef = useRef<{ cancelled: boolean }>({ cancelled: false });
  // Keep stable refs to the callbacks so the effect can read the latest
  // versions without the parent's identity-changing inline arrows
  // re-triggering the entire device flow.
  const onSuccessRef = useRef(onSuccess);
  const onOpenChangeRef = useRef(onOpenChange);
  useEffect(() => {
    onSuccessRef.current = onSuccess;
    onOpenChangeRef.current = onOpenChange;
  });

  useEffect(() => {
    if (!open) return;

    const cancel = { cancelled: false };
    cancelRef.current = cancel;

    setPhase({ kind: "starting" });
    setCopied(false);

    (async () => {
      let flow: DeviceFlowStart;
      try {
        flow = await githubDeviceStart();
      } catch (e) {
        if (!cancel.cancelled) {
          setPhase({ kind: "error", message: String(e) });
        }
        return;
      }
      if (cancel.cancelled) return;
      setPhase({ kind: "waiting", flow });

      let intervalSec = flow.interval;
      const deadline = Date.now() + flow.expires_in * 1000;

      while (!cancel.cancelled && Date.now() < deadline) {
        await new Promise((r) => setTimeout(r, intervalSec * 1000));
        if (cancel.cancelled) return;

        try {
          const result = await githubDevicePoll(flow.device_code);
          if (cancel.cancelled) return;

          switch (result.status) {
            case "pending":
              continue;
            case "slow_down":
              intervalSec += 5;
              continue;
            case "expired":
              setPhase({
                kind: "error",
                message: "Code expired — close and try again.",
              });
              return;
            case "denied":
              setPhase({
                kind: "error",
                message: "Authorization denied.",
              });
              return;
            case "error":
              setPhase({ kind: "error", message: result.message });
              return;
            case "success":
              setPhase({ kind: "success" });
              onSuccessRef.current({
                access_token: result.access_token,
                username: result.username,
                email: result.email,
                display_name: result.display_name,
              });
              setTimeout(() => {
                if (!cancel.cancelled) onOpenChangeRef.current(false);
              }, 800);
              return;
          }
        } catch (e) {
          if (!cancel.cancelled) {
            setPhase({ kind: "error", message: String(e) });
          }
          return;
        }
      }

      if (!cancel.cancelled) {
        setPhase({
          kind: "error",
          message: "Code expired — close and try again.",
        });
      }
    })();

    return () => {
      cancel.cancelled = true;
    };
  }, [open]);

  const handleOpen = async () => {
    if (phase.kind !== "waiting") return;
    try {
      await navigator.clipboard.writeText(phase.flow.user_code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Clipboard may not be permitted; non-fatal.
    }
    try {
      await openUrl(phase.flow.verification_uri);
    } catch (e) {
      console.error("Failed to open URL:", e);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-[440px]">
        <DialogTitle className="text-section">Sign in with GitHub</DialogTitle>
        <DialogDescription className="sr-only">
          Authorize this device on GitHub
        </DialogDescription>

        <div className="mt-4 space-y-4">
          {phase.kind === "starting" && (
            <div className="flex items-center gap-2 text-sm text-fg-muted">
              <Loader2 className="h-4 w-4 animate-spin" />
              Starting GitHub authorization...
            </div>
          )}

          {phase.kind === "waiting" && (
            <>
              <div>
                <p className="text-label mb-2">Enter this code on GitHub</p>
                <div className="flex items-center justify-center rounded-md border border-border-default bg-bg-elevated px-4 py-3">
                  <span className="font-mono text-2xl font-semibold tracking-[0.2em] text-fg-default">
                    {phase.flow.user_code}
                  </span>
                </div>
              </div>

              <Button onClick={handleOpen} className="w-full" size="sm">
                {copied ? (
                  <>
                    <CheckCircle2 className="mr-1.5 h-3.5 w-3.5" />
                    Copied — opening GitHub...
                  </>
                ) : (
                  <>
                    <Copy className="mr-1.5 h-3.5 w-3.5" />
                    Copy code & open GitHub
                    <ExternalLink className="ml-1.5 h-3 w-3 opacity-60" />
                  </>
                )}
              </Button>

              <div className="flex items-center gap-2 text-xs text-fg-muted">
                <Loader2 className="h-3 w-3 animate-spin" />
                Waiting for you to authorize this device...
              </div>
            </>
          )}

          {phase.kind === "success" && (
            <div className="flex items-center gap-2 text-sm text-success">
              <CheckCircle2 className="h-4 w-4" />
              Authorized — fetching account details...
            </div>
          )}

          {phase.kind === "error" && (
            <div className="flex items-start gap-2 text-sm text-danger">
              <XCircle className="mt-0.5 h-4 w-4 shrink-0" />
              <span>{phase.message}</span>
            </div>
          )}

          <div className="flex justify-end pt-2">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => onOpenChange(false)}
            >
              {phase.kind === "success" ? "Close" : "Cancel"}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
