import { GitBranch, FolderOpen } from "lucide-react";
import { openPath } from "@tauri-apps/plugin-opener";
import { getConfigFolder } from "@/lib/tauri";

export function AppShell({ children }: { children: React.ReactNode }) {
  const handleOpenConfig = async () => {
    try {
      const folder = await getConfigFolder();
      await openPath(folder);
    } catch (e) {
      console.error("Failed to open config folder:", e);
    }
  };

  return (
    <div className="flex h-full flex-col">
      {/* macOS titlebar drag region */}
      <div className="titlebar-drag flex h-12 shrink-0 items-center gap-2.5 pl-6 pr-4">
        <span
          className="inline-flex h-5 w-5 items-center justify-center rounded-md bg-accent/15"
        >
          <GitBranch className="h-3 w-3 text-accent" />
        </span>
        <span className="text-[13px] font-semibold tracking-tight text-fg-default">
          Git Switch
        </span>
        <button
          type="button"
          onClick={handleOpenConfig}
          title="Open accounts folder"
          className="titlebar-nodrag ml-auto inline-flex h-7 w-7 items-center justify-center rounded-md text-fg-muted transition-colors hover:bg-bg-elevated hover:text-fg-default"
        >
          <FolderOpen className="h-3.5 w-3.5" />
        </button>
      </div>

      {/* Main content */}
      <main className="flex-1 overflow-hidden px-6 pb-6 flex flex-col">
        {children}
      </main>
    </div>
  );
}
