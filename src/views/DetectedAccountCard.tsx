import { User, Mail, Plus } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Avatar } from "@/components/ui/badge";

export function DetectedAccountCard({
  name,
  email,
  onAdd,
}: {
  name: string;
  email: string;
  onAdd: (prefill: { name: string; email: string }) => void;
}) {
  return (
    <div className="card flex items-center gap-3 border-dashed border-border-strong px-4 py-3">
      <Avatar name={name} size={36} />

      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <span className="flex items-center gap-1.5 text-sm font-medium text-fg-default">
            <User className="h-3.5 w-3.5 text-fg-muted" />
            {name || "—"}
          </span>
          <Badge className="shrink-0 border-warning/30 bg-warning/10 text-warning text-[10px]">
            Unmanaged
          </Badge>
        </div>
        <span className="mt-0.5 flex items-center gap-1.5 text-xs text-fg-muted">
          <Mail className="h-3 w-3 text-fg-subtle" />
          {email || "—"}
        </span>
      </div>

      <Button
        variant="outline"
        size="sm"
        onClick={() => onAdd({ name, email })}
      >
        <Plus className="mr-1 h-3 w-3" />
        Add to accounts
      </Button>
    </div>
  );
}
