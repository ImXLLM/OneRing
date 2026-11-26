import { MaironOs } from "@/components/mairon/mairon-os";
import { MaironWorldProvider } from "@/lib/mairon/world-state";

/**
 * MAIRON OS — the sovereign Web OS home. One authoritative world state, five
 * non-destructively switchable perspectives, Ctrl+K everywhere.
 */
export default function Home() {
  return (
    <MaironWorldProvider>
      <MaironOs />
    </MaironWorldProvider>
  );
}
