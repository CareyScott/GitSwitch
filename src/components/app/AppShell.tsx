import { GitBranch, FolderOpen } from "lucide-react";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { getConfigFolder } from "@/lib/tauri";

export function AppShell({ children }: { children: React.ReactNode }) {
  const handleOpenConfig = async () => {
    try {
      const folder = await getConfigFolder();
      // revealItemInDir opens Finder and highlights the item — more
      // reliable than openPath for directories on macOS.
      await revealItemInDir(folder + "/accounts.json");
    } catch (e) {
      console.error("Failed to open config folder:", e);
    }
  };

  return (
    <div className="flex h-full flex-col">
      {/* macOS titlebar — drag region and button are siblings so
          click events on the button aren't swallowed by WKWebView's
          -webkit-app-region:drag handling. */}
      <div className="relative flex h-12 shrink-0 items-center gap-2.5 pl-6 pr-4">
        {/* Drag region covers the full bar behind everything */}
        <div className="titlebar-drag absolute inset-0" />

        <span className="relative inline-flex h-5 w-5 items-center justify-center rounded-md bg-accent/15">
          <GitBranch className="h-3 w-3 text-accent" />
        </span>
        <span className="relative text-[13px] font-semibold tracking-tight text-fg-default">
          Git Switch
        </span>
        <button
          type="button"
          onClick={handleOpenConfig}
          title="Open accounts folder"
          className="relative ml-auto inline-flex h-7 w-7 items-center justify-center rounded-md text-fg-muted transition-colors hover:bg-bg-elevated hover:text-fg-default"
        >
          <FolderOpen className="h-3.5 w-3.5 pointer-events-none" />
        </button>
      </div>

      {/* Main content */}
      <main className="flex-1 overflow-hidden px-6 pb-6 flex flex-col">
        {children}
      </main>
    </div>
  );
}
