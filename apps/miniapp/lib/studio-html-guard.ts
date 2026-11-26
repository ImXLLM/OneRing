/**
 * studio-html-guard — guard for the studio's in-flight HTML with zero-flicker preview.
 *
 * Wave 2 port from the CopilotKit/OpenIntelligentUI vault
 * (`04_.../open-generative-ui/process-partial-html.ts`) with the mission contract:
 * pure function, no DOM dependency — input text in, output text out.
 *
 * Why? During the studio's live stream, the preview iframe sees partial HTML at every moment:
 * - an unterminated `<...` tag at the end of the document → the browser interprets it and the page jumps;
 * - a `<script` block without `</script>` → a syntax error in the iframe and a red console light;
 * - an incomplete trailing `&...` → lost entity and text jump;
 * - without a layout guard → streamed content overflows horizontally.
 */

/** Responsive, chroma-free guard style — injected once, only if absent. */
const ZERO_CHROMA_FALLBACK_STYLE =
  `<style data-kian-studio-guard>` +
  `*,*::before,*::after{box-sizing:border-box!important}` +
  `html,body{max-width:100vw!important;overflow-x:hidden!important;margin:0}` +
  `img,svg,video,canvas{max-width:100%!important;height:auto}` +
  `</style>`;

/**
 * Sanitize streaming HTML for safe injection into the preview iframe.
 *
 * Pipeline (order matters):
 * 1. Trailing open tags are removed (`<...` without `>`);
 * 2. A trailing unterminated `<script` block is removed with all of its content so partial JS never
 *    throws a syntax error (complete blocks stay untouched);
 * 3. Trailing incomplete HTML entities are removed (`&...` without `;`);
 * 4. If the document lacks the responsive guard (box-sizing/overflow), it is injected.
 *
 * A healthy complete document comes out essentially untouched (at most the guard is added), so
 * keeping this guard always on for the preview carries no visual cost.
 */
export function sanitizePartialStudioHtml(rawHtml: string): string {
  let result = typeof rawHtml === "string" ? rawHtml : "";

  // 1) Trailing open tag — a `<div class=…` that is not yet closed.
  result = result.replace(/<[^>]*$/, "");

  // 2) Trailing unterminated script block — the last `<script` with no `</script>` after it.
  const lower = result.toLowerCase();
  const openScript = lower.lastIndexOf("<script");
  if (openScript !== -1 && lower.indexOf("</script>", openScript) === -1) {
    result = result.slice(0, openScript);
  }

  // 3) Trailing incomplete entity — `&amp` on its way to `&amp;`.
  result = result.replace(/&[a-zA-Z0-9#]*$/, "");

  // 4) Responsive guard — only if the document has not already locked box-sizing.
  if (!/box-sizing\s*:\s*border-box/i.test(result)) {
    const lowerAfter = result.toLowerCase();
    const headClose = lowerAfter.indexOf("</head>");
    if (headClose !== -1) {
      result =
        result.slice(0, headClose) + ZERO_CHROMA_FALLBACK_STYLE + result.slice(headClose);
    } else {
      result = ZERO_CHROMA_FALLBACK_STYLE + result;
    }
  }

  return result;
}
