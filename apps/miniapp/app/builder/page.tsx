import { redirect } from "next/navigation";

/** Legacy `/builder` route — the MAIRON OS keeps everything in the one-page four-tab shell. */
export default function BuilderRedirect(): never {
  redirect("/?tab=forge");
}
