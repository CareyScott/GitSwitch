import { useState } from "react";
import { User, Mail, Plus, Loader2, CheckCircle2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Avatar } from "@/components/ui/badge";
import { useAddAccount } from "@/lib/accounts";

export function DetectedAccountCard({
  name,
  email,
}: {
  name: string;
  email: string;
}) {
  const addMutation = useAddAccount();
  const [added, setAdded] = useState(false);

  const handleAdd = async () => {
    // Guess provider from email domain
    const provider = email.includes("github") ? "github" : "bitbucket";

    await addMutation.mutateAsync({
      provider,
      label: name || email.split("@")[0],
      username: email.split("@")[0],
      email,
      token: "",
    });
    setAdded(true);
  };

  return (
    <div className="card flex items-center gap-3 border-dashed border-border-strong px-4 py-3">
      {/* Avatar */}
      <Avatar name={name} size={36} />

      {/* Details */}
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

      {/* Add to managed accounts */}
      {added ? (
        <span className="flex items-center gap-1.5 text-xs text-success">
          <CheckCircle2 className="h-3.5 w-3.5" />
          Added
        </span>
      ) : (
        <Button
          variant="outline"
          size="sm"
          onClick={handleAdd}
          disabled={addMutation.isPending}
        >
          {addMutation.isPending ? (
            <Loader2 className="mr-1 h-3 w-3 animate-spin" />
          ) : (
            <Plus className="mr-1 h-3 w-3" />
          )}
          Add to accounts
        </Button>
      )}
    </div>
  );
}
