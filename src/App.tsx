import { AppShell } from "@/components/app/AppShell";
import { ActiveAccountBanner } from "@/views/ActiveAccountBanner";
import { AccountList } from "@/views/AccountList";

export default function App() {
  return (
    <AppShell>
      <ActiveAccountBanner />
      <AccountList />
    </AppShell>
  );
}
