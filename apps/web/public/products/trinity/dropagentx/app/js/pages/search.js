/* DropAgentX — SEARCH: products + users */
DGX.pages = DGX.pages || {};

DGX.pages.search = async (view, params) => {
  const q0 = params.q || '';
  view.innerHTML = `
    <div class="search-bar2">
      <svg width="18" height="18" style="color:var(--dim);flex:none"><use href="#i-search"/></svg>
      <input id="sq" placeholder="\u0645\u062d\u0635\u0648\u0644 \u06cc\u0627 \u06a9\u0627\u0631\u0628\u0631…" value="${DGX.esc(q0)}" autocomplete="off"
         enterkeyhint="search" autocapitalize="off" spellcheck="false">
    </div>
    <div id="sOut"><div class="empty"><div class="big">${DGX.icon('search','big-ic')}</div>\u062f\u0646\u0628\u0627\u0644 \u0686\u06cc \u0645\u06cc‌\u06af\u0631\u062f\u06cc\u061f<br>
      \u0627\u0633\u0645 \u0645\u062d\u0635\u0648\u0644\u060c \u062f\u0633\u062a\u0647 \u06cc\u0627 \u0633\u0627\u0632\u0646\u062f\u0647 \u0631\u0627 \u0628\u0646\u0648\u06cc\u0633</div></div>`;
  const inp = DGX.$('#sq'), out = DGX.$('#sOut');
  inp.focus();

  let deb = null, lastQ = '';
  async function run(q) {
    if (q.length < 2) {
      out.innerHTML = `<div class="empty"><div class="big">${DGX.icon('search','big-ic')}</div>\u062d\u062f\u0627\u0642\u0644 \u06f2 \u062d\u0631\u0641…</div>`;
      return;
    }
    lastQ = q;
    out.innerHTML = DGX.skelCards(2);
    try {
      const d = await DGX.api('/api/app/search?q=' + encodeURIComponent(q));
      if (lastQ !== q) return;                       // stale guard
      let html = '';
      if (d.users.length) {
        html += `<h2 style="margin:4px 0 8px;display:flex;align-items:center;gap:6px">${DGX.icon('users','ic-s')} \u06a9\u0627\u0631\u0628\u0631\u0627\u0646</h2>`;
        html += d.users.map(u => `
          <div class="user-row">
            <img class="avatar" src="/app/assets/logo.jpg" style="width:38px;height:38px">
            <div style="flex:1;min-width:0">
              <b style="font-size:13px">${DGX.esc(u.first_name || '\u06a9\u0627\u0631\u0628\u0631')}</b>
              ${u.username ? `<div style="color:var(--dim);font-size:11px">@${DGX.esc(u.username)}</div>` : ''}
            </div>
            <button class="follow-btn" data-fu="${u.user_id}">\u0641\u0627\u0644\u0648 +</button>
          </div>`).join('');
      }
      if (d.products.length) {
        html += `<h2 style="margin:14px 0 10px">${DGX.icon('box','ic-s')} \u0645\u062d\u0635\u0648\u0644\u0627\u062a</h2>` +
          d.products.map(DGX.postCard).join('');
      }
      out.innerHTML = html || `<div class="empty"><div class="big">🫥</div>
        \u0628\u0631\u0627\u06cc «${DGX.esc(q)}» \u0686\u06cc\u0632\u06cc \u067e\u06cc\u062f\u0627 \u0646\u0634\u062f</div>`;

      // follow buttons
      out.querySelectorAll('[data-fu]').forEach(b => b.onclick = async () => {
        const on = !b.classList.contains('on');
        try {
          await DGX.api('/api/app/follow', { body: { target: +b.dataset.fu, on } });
          b.classList.toggle('on', on);
          b.textContent = on ? '✓ \u0641\u0627\u0644\u0648 \u0634\u062f' : '\u0641\u0627\u0644\u0648 +';
          DGX.haptic('light');
        } catch (e) { DGX.toast(e.msg || '', true); }
      });
      // product card interactions (like/buy reuse home wiring)
      DGX.wireFeed && DGX.wireFeed(out);
    } catch (e) { out.innerHTML = `<div class="empty">⚠️ ${e.msg}</div>`; }
  }

  inp.oninput = () => { clearTimeout(deb); deb = setTimeout(() => run(inp.value.trim()), 380); };
  inp.onkeydown = e => { if (e.key === 'Enter') { clearTimeout(deb); run(inp.value.trim()); } };
  if (q0) run(q0);
};
