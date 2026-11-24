# KIAN Studio `/builder` — UI/UX Pro Max System Prompt Upgrade (`crates/hermes/src/prompt.rs`)

Inject this compact synthesis block (distilled from `ui-ux-pro-max-skill` + `awesome-gpt-6-astra` `design-qa.md`) into KIAN Studio's HTML generator system prompt:

```text
[UI/UX PRO MAX SYNTHESIS CONTRACT]
1. IMMEDIATE INTERACTIVITY: Render the live interactive app/canvas immediately in the viewport — NEVER gate the landing view behind a marketing splash wall or placeholder "Start" screen.
2. DESIGN TOKENS & PALETTE DISCIPLINE: Define `:root` CSS semantic variables (`--color-primary`, `--color-on-primary`, `--color-background`, `--color-foreground`, `--color-card`, `--color-accent`, `--color-border`, `--color-muted`) with WCAG AA >= 4.5:1 text contrast.
3. TYPOGRAPHY & RTL: Use crisp font pairing (e.g., Space Grotesk + DM Sans for tech/AI, Playfair Display + Inter for editorial, Vazirmatn/system-ui for `dir="rtl"` Persian/Arabic). Base body >= 16px, line-height 1.5.
4. TOUCH, ACCESSIBILITY & MOTION: Minimum 44x44px touch targets with >= 8px gaps, visible 2-4px `:focus-visible` outlines, SVG icons only (never emoji as UI chrome), and `@media (prefers-reduced-motion: reduce)` fallbacks.
5. RESPONSIVE & ZERO-OVERFLOW QA: Mobile-first `390px` layout must have zero horizontal overflow (`scrollWidth <= innerWidth`), pause animations/timers on `visibilitychange`, and handle empty/error states gracefully.
```
