import { GitBranch } from "lucide-react";

export function AppShell({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex h-full flex-col">
      {/* macOS titlebar drag region */}
      <div className="titlebar-drag flex h-12 shrink-0 items-center gap-2.5 pl-[72px] pr-4">
        <span
          className="inline-flex h-5 w-5 items-center justify-center rounded-md bg-accent/15"
        >
          <GitBranch className="h-3 w-3 text-accent" />
        </span>
        <span className="text-[13px] font-semibold tracking-tight text-fg-default">
          Git Switch
        </span>
      </div>

      {/* Main content */}
      <main className="flex-1 overflow-hidden px-6 pb-6 flex flex-col">
        {children}
      </main>
    </div>
  );
}
