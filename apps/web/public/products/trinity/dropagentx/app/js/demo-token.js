/* ────────────────────────────────────────────────────────────────────────
   MOCK-ONLY SHIM  (offline demo of this zip only)
   serve.py has a mock API that accepts any Bearer token; this line makes
   the mini-app boot up already «logged in» so the wallet/dashboard/
   library pages are populated too.
   No effect against the real backend: an invalid token is rejected with 401 and cleared.
   Remove this file and nothing else changes.
   ──────────────────────────────────────────────────────────────────────── */
try {
  if (!localStorage.getItem('dgx_token')) localStorage.setItem('dgx_token', 'mock-token');
} catch (e) {}
