import * as React from "react";
import { cn } from "@/lib/utils";

export function Badge({
  className,
  ...props
}: React.HTMLAttributes<HTMLSpanElement>) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 rounded-full border border-border bg-bg-muted px-2 py-0.5 text-[11px] font-medium text-fg",
        className,
      )}
      {...props}
    />
  );
}

export function Avatar({
  name,
  src,
  size = 24,
  className,
}: {
  name?: string | null;
  src?: string | null;
  size?: number;
  className?: string;
}) {
  const dim = `${size}px`;
  const letters =
    (name ?? "")
      .trim()
      .split(/\s+/)
      .filter((s) => /^[a-zA-Z]/.test(s))
      .slice(0, 2)
      .map((s) => s[0].toUpperCase())
      .join("") || "?";
  return (
    <span
      title={name ?? undefined}
      className={cn(
        "inline-flex shrink-0 items-center justify-center overflow-hidden rounded-full bg-bg-muted text-[10px] font-semibold text-fg",
        className,
      )}
      style={{ width: dim, height: dim, fontSize: Math.max(9, size * 0.4) }}
    >
      {src ? (
        <img src={src} alt="" className="h-full w-full object-cover" />
      ) : (
        letters
      )}
    </span>
  );
}
