import { redirect } from "next/navigation";

/** Legacy `/wallet` route — the MAIRON OS keeps everything in the one-page four-tab shell. */
export default function WalletRedirect(): never {
  redirect("/?tab=wallet");
}
