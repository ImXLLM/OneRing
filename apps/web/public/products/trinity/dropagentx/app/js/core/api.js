/* DropAgentX — HTTP client (Bearer auth + JSON + errors)
   Strategy: browse WITHOUT auth, prompt login only on actions */

DGX.token = '';
DGX.user = null;
DGX.isLoggedIn = false;

DGX.api = async (path, opt = {}) => {
  const headers = { 'X-Requested-With': 'fetch' };
  if (DGX.token) headers['Authorization'] = 'Bearer ' + DGX.token;
  let body = opt.body;
  if (body !== undefined && !(body instanceof FormData)) {
    headers['Content-Type'] = 'application/json';
    body = JSON.stringify(body);
  }
  const ac = new AbortController();
  const tm = setTimeout(() => ac.abort(), 20000);
  let r;
  try {
    r = await fetch(path, { method: opt.method || (body ? 'POST' : 'GET'),
                            headers, body, signal: ac.signal });
  } catch (_) {
    throw { msg: '\u0627\u0631\u062a\u0628\u0627\u0637 \u0628\u0631\u0642\u0631\u0627\u0631 \u0646\u0634\u062f 📡' };
  } finally { clearTimeout(tm); }

  if (r.status === 401) {
    // one retry with fresh token inside Telegram
    if (!DGX._authTried && window.Telegram?.WebApp?.initData) {
      DGX._authTried = true;
      const ok = await DGX.tgAuth();
      if (ok && DGX.token) return DGX.api(path, opt);
    }
    // show clear toast instead of silent failure
    DGX.toast('🔒 \u0628\u0631\u0627\u06cc \u0627\u06cc\u0646 \u06a9\u0627\u0631 \u0628\u0627\u06cc\u062f \u0627\u0632 \u062f\u0627\u062e\u0644 \u062a\u0644\u06af\u0631\u0627\u0645 \u0648\u0627\u0631\u062f \u0634\u06cc', true);
    throw { msg: '\u0628\u0631\u0627\u06cc \u0627\u06cc\u0646 \u06a9\u0627\u0631 \u0628\u0627\u06cc\u062f \u0648\u0627\u0631\u062f \u0634\u06cc', code: 401 };
  }

  const d = await r.json().catch(() => ({}));
  if (!r.ok) {
    let det = d.detail;
    if (Array.isArray(det)) det = det.map(z => z.msg || '').join(' · ');
    if (typeof det !== 'string' || !det) det = '\u062e\u0637\u0627 (' + r.status + ')';
    throw { msg: det };
  }
  return d;
};

/* Auto-login via Telegram initData. Returns true on success. */
DGX.tgAuth = async () => {
  const t = window.Telegram?.WebApp;
  const init = t?.initData || '';
  if (!init) return false;
  try {
    const r = await fetch('/api/app/auth', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'X-Requested-With': 'fetch' },
      body: JSON.stringify({ initData: init })
    });
    if (!r.ok) {
      console.error('[DGX tgAuth] server rejected initData:', r.status);
      return false;
    }
    const d = await r.json();
    DGX.token = d.token;
    localStorage.setItem('dgx_token', DGX.token);
    DGX.user = d.user;
    DGX.isLoggedIn = true;
    console.log('[DGX tgAuth] OK — user:', d.user?.name, '| credits:', d.user?.credits);
    return true;
  } catch (e) {
    console.error('[DGX tgAuth] error:', e);
    return false;
  }
};

DGX.refreshMe = async () => {
  if (!DGX.isLoggedIn) return;
  try { DGX.user = await DGX.api('/api/app/me'); } catch (_) {}
};

/* ════════ Login gate for actions (NOT for browsing) ════════ */

DGX.requireAuth = (actionName) => {
  if (DGX.isLoggedIn) return true;
  DGX.toast(`🔒 \u0628\u0631\u0627\u06cc ${actionName} \u0627\u0648\u0644 \u0648\u0627\u0631\u062f \u0634\u0648`, true);
  DGX.showLogin();
  return false;
};

/* Browser login flow (code via bot chat) */

DGX.showLogin = () => {
  const view = document.getElementById('view');
  if (!view) return;
  view.innerHTML = `
    <div style="max-width:340px;margin:40px auto;padding:0 16px;text-align:center">
      <img src="/app/assets/logo.jpg" style="width:64px;height:64px;border-radius:16px;margin-bottom:14px">
      <h2 style="font-size:18px;font-weight:800;margin-bottom:6px">DropAgentX</h2>
      <p style="color:#8b94a8;font-size:12.5px;line-height:2">\u0622\u06cc\u062f\u06cc \u0639\u062f\u062f\u06cc \u062a\u0644\u06af\u0631\u0627\u0645\u062a \u0631\u0627 \u0648\u0627\u0631\u062f \u06a9\u0646.<br>\u06a9\u062f \u062a\u0623\u06cc\u06cc\u062f \u0628\u0647 \u0686\u062a \u0628\u0627\u062a \u0627\u0631\u0633\u0627\u0644 \u0645\u06cc‌\u0634\u0648\u062f.</p>
      <div id="loginStep1" style="margin-top:18px">
        <input id="lgId" type="number" aria-label="\u0622\u06cc\u062f\u06cc \u0639\u062f\u062f\u06cc \u062a\u0644\u06af\u0631\u0627\u0645"
          style="width:100%;background:#0B0F0D;border:1px solid #1c2420;border-radius:12px;
                 padding:13px 16px;color:#f4f7f5;font-family:inherit;font-size:14px;
                 outline:none;text-align:center;direction:ltr"
          onkeydown="if(event.key==='Enter')DGX._reqCode()">
        <button onclick="DGX._reqCode()" style="width:100%;margin-top:10px;
          background:linear-gradient(135deg,#00FF88,#00c46a);color:#03130a;border:0;
          border-radius:12px;padding:13px;font-family:inherit;font-size:14px;
          font-weight:800;cursor:pointer">📨 \u062f\u0631\u06cc\u0627\u0641\u062a \u06a9\u062f</button>
      </div>
      <div id="loginStep2" style="display:none;margin-top:18px">
        <p style="color:#00FF88;font-size:12px;margin-bottom:10px">✅ \u06a9\u062f \u0627\u0631\u0633\u0627\u0644 \u0634\u062f!</p>
        <input id="lgCode" type="text" maxlength="6" aria-label="\u06a9\u062f \u06f6 \u0631\u0642\u0645\u06cc"
          style="width:100%;background:#0B0F0D;border:1px solid #1c2420;border-radius:12px;
                 padding:13px 16px;color:#f4f7f5;font-family:inherit;font-size:20px;
                 letter-spacing:8px;outline:none;text-align:center;direction:ltr"
          onkeydown="if(event.key==='Enter')DGX._verifyCode()">
        <button onclick="DGX._verifyCode()" style="width:100%;margin-top:10px;
          background:linear-gradient(135deg,#00FF88,#00c46a);color:#03130a;border:0;
          border-radius:12px;padding:13px;font-family:inherit;font-size:14px;
          font-weight:800;cursor:pointer">🔓 \u0648\u0631\u0648\u062f</button>
      </div>
      <div id="lgMsg" style="margin-top:12px;font-size:12px;min-height:20px;color:#ff5c7a"></div>
      <br><button onclick="location.hash='#/home'" class="btn btn-ghost"
        style="width:100%">← \u062f\u06cc\u062f\u0646 \u0645\u062d\u0635\u0648\u0644\u0627\u062a \u0628\u062f\u0648\u0646 \u0648\u0631\u0648\u062f</button>
      <p style="color:#5c6862;font-size:10.5px;margin-top:20px;line-height:2">
        \u0622\u06cc\u062f\u06cc \u0639\u062f\u062f\u06cc \u0627\u0632 @userinfobot<br>\u06cc\u0627 ☰ \u062f\u0627\u062e\u0644 \u062a\u0644\u06af\u0631\u0627\u0645 \u0628\u0632\u0646</p>
    </div>`;
};

DGX._reqCode = async () => {
  const inp = document.getElementById('lgId');
  const msg = document.getElementById('lgMsg');
  const tid = parseInt(inp.value, 10);
  if (!tid || tid < 100) { msg.textContent = '⚠️ \u0622\u06cc\u062f\u06cc \u0645\u0639\u062a\u0628\u0631 \u0648\u0627\u0631\u062f \u06a9\u0646'; return; }
  msg.textContent = '⏳ …';
  try {
    const r = await fetch('/api/app/login-request', {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ telegram_id: tid }) });
    const d = await r.json().catch(() => ({}));
    if (!r.ok) throw d.detail || '\u062e\u0637\u0627';
    msg.textContent = '';
    document.getElementById('loginStep1').style.display = 'none';
    document.getElementById('loginStep2').style.display = 'block';
    document.getElementById('lgCode').focus();
    DGX._pendingId = tid;
  } catch (e) {
    msg.textContent = '⚠️ ' + (typeof e === 'string' ? e : '\u062e\u0637\u0627'); msg.style.color = '#ff5c7a';
  }
};

DGX._verifyCode = async () => {
  const code = document.getElementById('lgCode').value.trim();
  const msg = document.getElementById('lgMsg');
  if (!code || code.length !== 6) { msg.textContent = '\u06a9\u062f \u06f6 \u0631\u0642\u0645\u06cc'; return; }
  msg.textContent = '⏳ …';
  try {
    const r = await fetch('/api/app/login-verify', {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ telegram_id: DGX._pendingId, code }) });
    const d = await r.json().catch(() => ({}));
    if (!r.ok) throw d.detail || '\u062e\u0637\u0627';
    DGX.token = d.token;
    localStorage.setItem('dgx_token', DGX.token);
    DGX.user = d.user;
    DGX.isLoggedIn = true;
    location.hash = '#/home'; location.reload();
  } catch (e) {
    msg.textContent = '⚠️ ' + (typeof e === 'string' ? e : '\u062e\u0637\u0627'); msg.style.color = '#ff5c7a';
  }
};
