import { cn } from "@/lib/utils";

export function EmptyState({
  icon,
  title,
  description,
  action,
  className,
}: {
  icon?: React.ReactNode;
  title: string;
  description?: string;
  action?: React.ReactNode;
  className?: string;
}) {
  return (
    <div
      className={cn(
        "flex flex-col items-center justify-center gap-3 py-16 text-center",
        className,
      )}
    >
      {icon && (
        <div className="flex h-12 w-12 items-center justify-center rounded-full bg-bg-elevated text-fg-subtle">
          {icon}
        </div>
      )}
      <div className="space-y-1">
        <p className="text-section">{title}</p>
        {description && (
          <p className="text-sm text-fg-muted">{description}</p>
        )}
      </div>
      {action}
    </div>
  );
}
