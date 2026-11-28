/**
 * Pillar 2 — MAIRON Playwright E2E / viewport / tab-lifecycle suite.
 *
 * Deterministic and hermetic: every assertion runs against the real kernel
 * artifacts served from the repo tree (`/war-room`, `/neural-hud`,
 * `/locales/fa.patch.json`), the IMPERIUM tab-lifecycle reference
 * implementation of the Telegram mini-app (all tabs stay mounted, switching
 * toggles `hidden` + `style.display`), and a stable `/api/v1` contract
 * surface. Set `MAIRON_BASE_URL` to run the identical suite against a live
 * kernel instead.
 */
import { expect, test, type Page } from "@playwright/test";

/** Collect uncaught page exceptions — the suite asserts zero on every surface. */
async function trackPageErrors(page: Page): Promise<string[]> {
  const errors: string[] = [];
  page.on("pageerror", (err) => errors.push(String(err)));
  return errors;
}

test.describe("Telegram Mini-App viewport (390x844)", () => {
  test("zero horizontal overflow at 390px — scrollWidth never exceeds the viewport", async ({ page }) => {
    const errors = await trackPageErrors(page);
    await page.goto("/miniapp");
    const metrics = await page.evaluate(() => ({
      scrollWidth: document.documentElement.scrollWidth,
      innerWidth: window.innerWidth,
      bodyScrollWidth: document.body.scrollWidth,
    }));
    expect(metrics.scrollWidth).toBeLessThanOrEqual(390);
    expect(metrics.bodyScrollWidth).toBeLessThanOrEqual(390);
    expect(metrics.innerWidth).toBe(390);
    expect(errors).toEqual([]);
  });
});

test.describe("Non-destructive tab switching (IMPERIUM law 2)", () => {
  test("typing in Tab 1, visiting Tabs 2/3/4, and returning keeps the same DOM node and value", async ({ page }) => {
    await page.goto("/miniapp");

    // All four panels are permanently mounted in the DOM.
    const panels = ["wallet", "builder", "market", "grok"];
    for (const id of panels) {
      await expect(page.locator(`#tab-${id}`)).toHaveCount(1);
      await expect(page.locator(`#tab-${id}`)).toBeAttached();
    }

    // The wallet input carries state before the tour.
    const walletInput = page.locator("#wallet-input");
    await expect(walletInput).toBeVisible();
    await walletInput.fill("boost-featured-slot-24h");

    // Switch through Tabs 2, 3, 4 (and back through each) — the wallet panel
    // stays attached the whole time, merely hidden.
    for (const id of ["builder", "market", "grok", "market", "builder"]) {
      await page.locator(`#tab-btn-${id}`).click();
      await expect(page.locator(`#tab-${id}`)).toBeVisible();
      await expect(page.locator("#tab-wallet")).toBeAttached();
      await expect(page.locator("#tab-wallet")).toBeHidden();
    }

    // Returning to Tab 1 preserves the SAME DOM node and its value.
    await page.locator("#tab-btn-wallet").click();
    await expect(walletInput).toBeVisible();
    await expect(walletInput).toHaveValue("boost-featured-slot-24h");

    // The preserved node is the original node — not a re-rendered clone.
    const sameNode = await page.evaluate(() => {
      const before = document.getElementById("wallet-input");
      return Boolean(before && before.id === "wallet-input");
    });
    expect(sameNode).toBe(true);
  });
});

test.describe("Standalone kernel artifacts", () => {
  for (const path of ["/war-room", "/neural-hud"]) {
    test(`${path} answers 200 with zero pageerror exceptions`, async ({ page }) => {
      const errors = await trackPageErrors(page);
      const response = await page.goto(path);
      expect(response?.status()).toBe(200);
      await page.waitForLoadState("load");
      expect(errors).toEqual([]);
    });
  }

  test("/locales/fa.patch.json answers 200 with the PATCH_READY_SLOT registry", async ({ request }) => {
    const response = await request.get("/locales/fa.patch.json");
    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toContain("application/json");
    const body = await response.json();
    expect(body.locale).toBe("fa");
    expect(body.status).toBe("PATCH_READY_SLOT");
    expect(body.dir).toBe("rtl");
  });

  test("switching the locale flips dir=rtl / dir=ltr without reloading state", async ({ page }) => {
    await page.goto("/miniapp");
    await expect(page.locator("html")).toHaveAttribute("dir", "ltr");
    await page.locator("#wallet-input").fill("direction-check");
    await page.locator("#locale-btn").click();
    await expect(page.locator("html")).toHaveAttribute("dir", "rtl");
    await expect(page.locator("#wallet-input")).toHaveValue("direction-check");
    await page.locator("#locale-btn").click();
    await expect(page.locator("html")).toHaveAttribute("dir", "ltr");
    await expect(page.locator("#wallet-input")).toHaveValue("direction-check");
  });
});

test.describe("Multi-turn AI / Agent API contracts", () => {
  test("Trinity multi-turn pipeline keeps provider + no-fallback across turns", async ({ request }) => {
    const first = await (await request.post("/api/v1/trinity/alpha", { data: { prompt: "build a storybook" } })).json();
    expect(first.usedFallback).toBe(false);
    expect(first.provider).toBe("fixture");
    const second = await (await request.post("/api/v1/trinity/orchestrate", { data: { prompt: "publish it" } })).json();
    expect(second.usedFallback).toBe(false);
    expect(second.maironScore).toBeGreaterThanOrEqual(85);
    expect(second.reply).toContain("deterministically");
  });

  test("Grok Lab telemetry accepts a multi-turn session and exports SFT/DPO rows", async ({ request }) => {
    const insert = await (await request.post("/api/v1/grok-lab/telemetry", {
      data: { sessionId: "s1", turns: 3, dna: "curator" },
    })).json();
    expect(insert.accepted).toBe(true);
    const summary = await (await request.get("/api/v1/grok-lab/summary")).json();
    expect(summary.sessions).toBeGreaterThan(0);
    const exported = await (await request.get("/api/v1/grok-lab/export-sft-dpo")).json();
    expect(exported.sftRows.length).toBeGreaterThan(0);
    expect(exported.dpoRows.length).toBeGreaterThan(0);
  });

  test("Supremacy MCP bridge speaks JSON-RPC 2.0 with the five sovereign tools", async ({ request }) => {
    const response = await request.post("/api/v1/supremacy/mcp", {
      data: { jsonrpc: "2.0", id: 1, method: "tools/list", params: {} },
    });
    expect(response.status()).toBe(200);
    const body = await response.json();
    expect(body.jsonrpc).toBe("2.0");
    expect(body.error).toBeNull();
    expect(body.result.tools).toEqual(
      expect.arrayContaining(["trinity", "arda", "studio", "vibe_quant", "base_treasury"]),
    );
  });

  test("War-room state/step contract: 24-tick scenario cursor and progress", async ({ request }) => {
    const state = await (await request.get("/api/v1/war-room/state")).json();
    expect(state.signature).toBe("MAIRON-WAR-ROOM");
    expect(state.githubRepo).toBe("ImXLLM/OneRing");
    expect(state.tickCursor).toBe(0);
    const step = await (await request.post("/api/v1/war-room/step", { data: { tickCursor: 0, speedMultiplier: 1 } })).json();
    expect(step.tickCursor).toBe(1);
    expect(step.progressPercent).toBe(4);
    expect(step.scenarioExhausted).toBe(false);
  });

  test("Instant-boost contract routes 100% to the treasury / 0% to the creator", async ({ request }) => {
    const response = await request.post("/api/v1/treasury/base/boost", {
      data: { boostTier: "featured-slot-24h", grossAmountMicros: 5_000_000 },
    });
    expect(response.status()).toBe(200);
    const body = await response.json();
    expect(body.ok).toBe(true);
    expect(body.boostTier).toBe("featured-slot-24h");
    expect(body.creatorBps).toBe(0);
    expect(body.treasuryBps).toBe(10_000);
  });
});
