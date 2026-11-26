import { redirect } from "next/navigation";

/** Legacy `/market` route — the MAIRON OS keeps everything in the one-page four-tab shell. */
export default function MarketRedirect(): never {
  redirect("/?tab=desk");
}
