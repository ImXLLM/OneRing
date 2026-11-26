/* DropAgentX v2.0 — CREATOR SUITE: create-drop / drafts / dashboard / analytics /
   products-manage / payouts  (seller tools on real store+activity+wallet APIs) */
DGX.pages = DGX.pages || {};

/* ── CREATE-DROP — drop composer w/ live preview (0xBazaar #36 DNA) ── */
DGX.pages['create-drop'] = async (view) => {
  if (!DGX.requireAuth('\u0633\u0627\u062e\u062a \u062f\u0631\u0627\u067e')) { location.hash = '#/home'; return; }
  let s = null;
  try { s = await DGX.api(`/api/app/store/${DGX.user.id}`); } catch (_) {}
  const prods = (s && s.products) || [];
  view.innerHTML = `
    <p class="page-hint">${DGX.icon('flame','ic-s')} \u062f\u0631\u0627\u067e = \u0645\u0648\u062c \u0641\u0631\u0648\u0634 \u0645\u062d\u062f\u0648\u062f. \u0641\u0631\u0645 \u0631\u0627 \u067e\u0631 \u06a9\u0646\u060c \u0628\u0627\u062a \u0645\u0646\u062a\u0634\u0631\u0634 \u0645\u06cc‌\u06a9\u0646\u062f \u0648 \u062f\u0631 \u0635\u0641\u062d\u0647‌\u06cc «\u062f\u0631\u0627\u067e‌\u0647\u0627» \u0645\u06cc‌\u0646\u0634\u06cc\u0646\u062f.</p>
    <div class="post" style="padding:16px">
      <div class="field"><label>\u0645\u062d\u0635\u0648\u0644 \u062f\u0631\u0627\u067e</label>
        <select id="dProd">${prods.length ? prods.map(p => `<option value="${+p.id}">${DGX.esc(p.title)}</option>`).join('')
          : '<option value="">\u0627\u0648\u0644 \u06cc\u06a9 \u0645\u062d\u0635\u0648\u0644 \u0628\u0633\u0627\u0632</option>'}</select></div>
      <div class="field"><label>\u0639\u0646\u0648\u0627\u0646 \u062f\u0631\u0627\u067e</label>
        <input id="dTitle" maxlength="80" aria-label="\u0645\u062b\u0644\u0627\u064b: \u062f\u0631\u0627\u067e \u0637\u0644\u0627\u06cc\u06cc \u062c\u0645\u0639\u0647‌\u0634\u0628"></div>
      <div class="field"><label>\u0632\u0645\u0627\u0646 \u067e\u0627\u06cc\u0627\u0646</label>
        <select id="dEnd"><option value="\u0627\u0645\u0634\u0628">\u0627\u0645\u0634\u0628 (\u062a\u0627 \u06f2\u06f4)</option><option value="\u06f2\u06f4 \u0633\u0627\u0639\u062a">\u06f2\u06f4 \u0633\u0627\u0639\u062a</option><option value="\u06f4\u06f8 \u0633\u0627\u0639\u062a">\u06f4\u06f8 \u0633\u0627\u0639\u062a</option><option value="\u06cc\u06a9 \u0647\u0641\u062a\u0647">\u06cc\u06a9 \u0647\u0641\u062a\u0647</option></select></div>
      <div class="field"><label>\u0645\u062a\u0646 \u0627\u0639\u0644\u0627\u0646</label>
        <textarea id="dMsg" rows="3" maxlength="300" aria-label="\u06cc\u06a9 \u062c\u0645\u0644\u0647‌\u06cc \u0647\u06cc\u062c\u0627\u0646\u06cc \u0628\u0631\u0627\u06cc \u0641\u0627\u0644\u0648\u0648\u0631\u0647\u0627…"></textarea></div>
      <button class="btn btn-primary" id="dGo" ${prods.length ? '' : 'disabled style="opacity:.5"'}>${DGX.icon('send','ic-s')} \u062b\u0628\u062a \u062f\u0631\u0627\u067e \u0627\u0632 \u0637\u0631\u06cc\u0642 \u0628\u0627\u062a</button>
      <a class="btn btn-ghost" href="#/drafts">${DGX.icon('doc','ic-s')} \u067e\u06cc\u0634‌\u0646\u0648\u06cc\u0633‌\u0647\u0627</a>
    </div>
    <h3 style="margin:16px 0 8px">${DGX.icon('eye','ic-s')} \u067e\u06cc\u0634‌\u0646\u0645\u0627\u06cc\u0634 \u0632\u0646\u062f\u0647</h3>
    <div class="post" id="dPrev"></div>`;
  const t = DGX.$('#dTitle'), m = DGX.$('#dMsg'), prev = DGX.$('#dPrev');
  const draw = () => {
    const p = prods.find(x => +x.id === +DGX.$('#dProd').value) || prods[0];
    prev.innerHTML = `
      <div class="post-head"><span class="nr-ic">${DGX.icon('flame')}</span>
        <div><div class="p-name">${DGX.esc(t.value || '\u062f\u0631\u0627\u067e \u062c\u062f\u06cc\u062f \u062a\u0648')}</div>
        <div class="p-sub">\u067e\u0627\u06cc\u0627\u0646: ${DGX.$('#dEnd').value}</div></div>
        <span class="pill p-live" style="margin-inline-start:auto">\u0632\u0646\u062f\u0647</span></div>
      ${p ? `<div class="ph-media" style="height:110px">${p.photo_url ? `<img src="${DGX.esc(p.photo_url)}" style="width:100%;height:100%;object-fit:cover">` : DGX.icon('box','ic-xl')}</div>` : ''}
      <div class="post-body"><div class="post-desc">${DGX.esc(m.value || '\u0645\u062a\u0646 \u0627\u0639\u0644\u0627\u0646 \u0627\u06cc\u0646\u062c\u0627 \u0646\u0645\u0627\u06cc\u0634 \u062f\u0627\u062f\u0647 \u0645\u06cc‌\u0634\u0648\u062f…')}</div></div>`;
  };
  [t, m].forEach(el => el.oninput = draw);
  DGX.$('#dProd').onchange = draw; draw();
  DGX.$('#dGo').onclick = () => {
    const p = prods.find(x => +x.id === +DGX.$('#dProd').value);
    const txt = `\u062f\u0631\u0627\u067e «${t.value || '\u0628\u062f\u0648\u0646 \u0639\u0646\u0648\u0627\u0646'}» — \u0645\u062d\u0635\u0648\u0644: ${p ? p.title : '\u061f'} — \u067e\u0627\u06cc\u0627\u0646: ${DGX.$('#dEnd').value}\n${m.value}`;
    const d = DGX.localList('dgx_drafts');
    d.unshift({ k: 'drop', txt, t: Date.now() });
    localStorage.setItem('dgx_drafts', JSON.stringify(d.slice(0, 30)));
    DGX.openBot('/drop ' + txt.slice(0, 120));
    DGX.toast('\u067e\u06cc\u0634‌\u0646\u0648\u06cc\u0633 \u0630\u062e\u06cc\u0631\u0647 \u0634\u062f + \u0628\u0627\u062a \u0628\u0627\u0632 \u0634\u062f');
  };
};

/* ── DRAFTS — local composer drafts ── */
DGX.pages.drafts = async (view) => {
  const l = DGX.localList('dgx_drafts');
  view.innerHTML = `
    <p class="page-hint">${DGX.icon('doc','ic-s')} ${l.length} \u067e\u06cc\u0634‌\u0646\u0648\u06cc\u0633 \u0631\u0648\u06cc \u0627\u06cc\u0646 \u062f\u0633\u062a\u06af\u0627\u0647</p>
    ${l.length ? l.map((d, i) => `
      <div class="notif-row">
        <span class="nr-ic">${DGX.icon(d.k === 'drop' ? 'flame' : 'doc')}</span>
        <div style="flex:1;min-width:0"><b>${d.k === 'drop' ? '\u062f\u0631\u0627\u067e' : '\u06cc\u0627\u062f\u062f\u0627\u0634\u062a'}</b>
          <span class="p-sub" style="display:block;white-space:pre-line">${DGX.esc((d.txt || '').slice(0, 140))}</span></div>
        <button class="lnk" data-i="${i}">\u062d\u0630\u0641</button>
      </div>`).join('') : `<div class="empty"><div class="big">${DGX.icon('doc','big-ic')}</div>\u067e\u06cc\u0634‌\u0646\u0648\u06cc\u0633\u06cc \u0646\u06cc\u0633\u062a<br><br>
        <a class="btn btn-primary" href="#/create-drop">${DGX.icon('flame','ic-s')} \u0633\u0627\u062e\u062a \u062f\u0631\u0627\u067e</a></div>`}`;
  view.querySelectorAll('[data-i]').forEach(b => b.onclick = () => {
    const l2 = DGX.localList('dgx_drafts'); l2.splice(+b.dataset.i, 1);
    localStorage.setItem('dgx_drafts', JSON.stringify(l2));
    DGX.toast('\u062d\u0630\u0641 \u0634\u062f'); DGX.pages.drafts(view, {});
  });
};

/* ── DASHBOARD — seller home (0xBazaar #08 DNA) ── */
DGX.pages.dashboard = async (view) => {
  if (!DGX.requireAuth('\u062f\u0627\u0634\u0628\u0648\u0631\u062f')) { location.hash = '#/home'; return; }
  view.innerHTML = DGX.skelCards(2);
  let a, w, s;
  try { [a, w, s] = await Promise.all([
    DGX.api('/api/app/activity'), DGX.api('/api/app/wallet'),
    DGX.api(`/api/app/store/${DGX.user.id}`).catch(() => null)]); }
  catch (e) { view.innerHTML = `<div class="empty">${e.msg || '\u062e\u0637\u0627'}</div>`; return; }
  const sold = a.sold || [];
  const revenue = sold.reduce((x, o) => x + (o.price_credits || 0), 0);
  view.innerHTML = `
    <p class="page-hint">${DGX.icon('shop','ic-s')} ${DGX.esc((DGX.user || {}).name || '\u0631\u0641\u06cc\u0642')} — \u0641\u0631\u0648\u0634\u06af\u0627\u0647\u062a \u062f\u0631 \u062d\u0627\u0644 \u0631\u0634\u062f \u0627\u0633\u062a</p>
    <div class="stat-grid">
      <div class="stat-card"><span>${DGX.icon('cart','ic-s')} \u0641\u0631\u0648\u0634</span><b class="num">${sold.length}</b></div>
      <div class="stat-card"><span>${DGX.icon('wallet','ic-s')} \u062f\u0631\u0622\u0645\u062f \u06a9\u0644</span><b class="num">${DGX.fmt(w.earned || 0)}</b></div>
      <div class="stat-card"><span>${DGX.icon('box','ic-s')} \u0645\u062d\u0635\u0648\u0644</span><b class="num">${s ? s.products.length : 0}</b></div>
      <div class="stat-card"><span>${DGX.icon('users','ic-s')} \u0641\u0627\u0644\u0648\u0648\u0631</span><b class="num">${DGX.kfmt(s ? s.followers : 0)}</b></div>
    </div>
    <div class="post" style="padding:14px">
      <h3 style="margin-bottom:8px">${DGX.icon('chart','ic-s')} \u062f\u0631\u0622\u0645\u062f \u06f7 \u0631\u0648\u0632 \u0627\u062e\u06cc\u0631</h3>
      <div class="bars">${DGX.weekBars(sold)}</div>
    </div>
    <div class="col-grid">
      <a class="col-card" href="#/create"><span class="cc-ic">${DGX.icon('plus')}</span><b>\u0645\u062d\u0635\u0648\u0644 \u062c\u062f\u06cc\u062f</b></a>
      <a class="col-card" href="#/create-drop"><span class="cc-ic">${DGX.icon('flame')}</span><b>\u062f\u0631\u0627\u067e \u062c\u062f\u06cc\u062f</b></a>
      <a class="col-card" href="#/analytics"><span class="cc-ic">${DGX.icon('chart')}</span><b>\u062a\u062d\u0644\u06cc\u0644</b></a>
      <a class="col-card" href="#/payouts"><span class="cc-ic">${DGX.icon('withdraw')}</span><b>\u0628\u0631\u062f\u0627\u0634\u062a</b></a>
    </div>
    <h3 style="margin:16px 0 8px">${DGX.icon('history','ic-s')} \u0622\u062e\u0631\u06cc\u0646 \u0633\u0641\u0627\u0631\u0634‌\u0647\u0627</h3>
    ${sold.slice(0, 3).map(o => `
      <a class="notif-row" href="#/order?id=${+o.id}">
        <span class="rr-img">${DGX.icon('box')}</span>
        <div style="flex:1;min-width:0"><b>${DGX.esc(o.title)}</b>
          <span class="p-sub">${DGX.esc(o.buyer || '')} · ${DGX.timeAgo(o.purchased_at || 0)}</span></div>
        <b class="num" style="color:var(--em)">+${DGX.fmt(o.price_credits)}</b></a>`).join('') ||
      `<div class="empty" style="padding:16px">\u0647\u0646\u0648\u0632 \u0641\u0631\u0648\u0634\u06cc \u0646\u06cc\u0633\u062a — \u0627\u0648\u0644\u06cc\u0646 \u0645\u062d\u0635\u0648\u0644 \u0631\u0627 \u0628\u0633\u0627\u0632<br><br>
        <a class="btn btn-primary" href="#/create">${DGX.icon('plus','ic-s')} \u0634\u0631\u0648\u0639 \u0641\u0631\u0648\u0634</a></div>`}`;
};

/* ── ANALYTICS — sales analysis (0xBazaar #14 DNA) ── */
DGX.pages.analytics = async (view) => {
  if (!DGX.requireAuth('\u062a\u062d\u0644\u06cc\u0644')) { location.hash = '#/home'; return; }
  let a, w;
  try { [a, w] = await Promise.all([DGX.api('/api/app/activity'), DGX.api('/api/app/wallet')]); }
  catch (e) { view.innerHTML = `<div class="empty">${e.msg || '\u062e\u0637\u0627'}</div>`; return; }
  const sold = a.sold || [], bought = a.bought || [];
  const rev = sold.reduce((x, o) => x + (o.price_credits || 0), 0);
  const spend = bought.reduce((x, o) => x + (o.price_credits || 0), 0);
  const aov = sold.length ? Math.round(rev / sold.length) : 0;
  view.innerHTML = `
    <div class="stat-grid">
      <div class="stat-card"><span>\u062f\u0631\u0622\u0645\u062f</span><b class="num" style="color:var(--em)">${DGX.fmt(w.earned || rev)}</b></div>
      <div class="stat-card"><span>\u0633\u0641\u0627\u0631\u0634‌\u0647\u0627</span><b class="num">${sold.length}</b></div>
      <div class="stat-card"><span>\u0645\u06cc\u0627\u0646\u06af\u06cc\u0646 \u0633\u0641\u0627\u0631\u0634</span><b class="num">${DGX.fmt(aov)}</b></div>
      <div class="stat-card"><span>\u0645\u0635\u0631\u0641 \u062a\u0648</span><b class="num">${DGX.fmt(w.spent || spend)}</b></div>
    </div>
    <div class="post" style="padding:14px">
      <h3 style="margin-bottom:8px">${DGX.icon('chart','ic-s')} \u0641\u0631\u0648\u0634 \u06f7 \u0631\u0648\u0632 \u0627\u062e\u06cc\u0631</h3>
      <div class="bars">${DGX.weekBars(sold)}</div>
    </div>
    <div class="post" style="padding:14px">
      <h3 style="margin-bottom:10px">${DGX.icon('trophy','ic-s')} \u0645\u062d\u0635\u0648\u0644\u0627\u062a \u067e\u0631\u0641\u0631\u0648\u0634 \u062a\u0648</h3>
      ${( sold.reduce((acc, o) => { const f = acc.find(x => x.t === o.title);
          if (f) f.n++; else acc.push({ t: o.title, n: 1, c: o.price_credits || 0 }); return acc; }, [])
        .sort((x, y) => y.n - x.n).slice(0, 6).map(x => `
        <div class="bar-row"><span>${DGX.esc(x.t)}</span>
          <i style="width:${Math.min(100, x.n * 25)}%"></i><b class="num">${x.n}</b></div>`).join('')) ||
        '<div class="empty" style="padding:12px">\u062f\u0627\u062f\u0647‌\u0627\u06cc \u0646\u06cc\u0633\u062a — \u0628\u0627 \u0627\u0648\u0644\u06cc\u0646 \u0641\u0631\u0648\u0634 \u067e\u0631 \u0645\u06cc‌\u0634\u0648\u062f</div>'}
    </div>
    <div style="text-align:center;padding:10px"><a class="lnk" href="#/dashboard">← \u062f\u0627\u0634\u0628\u0648\u0631\u062f \u0641\u0631\u0648\u0634\u0646\u062f\u0647</a></div>`;
};

/* ── PRODUCTS-MANAGE — my catalog (0xBazaar #13 DNA) ── */
DGX.pages['products-manage'] = async (view) => {
  if (!DGX.requireAuth('\u0645\u062f\u06cc\u0631\u06cc\u062a')) { location.hash = '#/home'; return; }
  let s;
  try { s = await DGX.api(`/api/app/store/${DGX.user.id}`); }
  catch (e) { view.innerHTML = `<div class="empty">${e.msg || '\u062e\u0637\u0627'}</div>`; return; }
  const ps = s.products || [];
  view.innerHTML = `
    <div class="chips">
      <button class="chip on">${DGX.icon('box','ic-xs')} ${ps.length} \u0645\u062d\u0635\u0648\u0644</button>
      <button class="chip">${DGX.kfmt(s.total_sales)} \u0641\u0631\u0648\u0634 \u06a9\u0644</button>
    </div>
    ${ps.length ? ps.map(p => `
      <div class="notif-row">
        <div class="rr-img">${p.photo_url ? `<img src="${DGX.esc(p.photo_url)}">` : DGX.icon('box')}</div>
        <div style="flex:1;min-width:0"><b>${DGX.esc(p.title)}</b>
          <span class="p-sub">${DGX.kfmt(p.sales_count)} \u0641\u0631\u0648\u0634 · ${DGX.kfmt(p.like_count)} \u067e\u0633\u0646\u062f</span></div>
        <b class="num" style="color:var(--em)">${DGX.fmt(p.price_credits)}</b>
      </div>`).join('') : `<div class="empty"><div class="big">${DGX.icon('box','big-ic')}</div>\u0642\u0641\u0633\u0647‌\u0627\u062a \u062e\u0627\u0644\u06cc\u0647<br><br>
        <a class="btn btn-primary" href="#/create">${DGX.icon('plus','ic-s')} \u0627\u0648\u0644\u06cc\u0646 \u0645\u062d\u0635\u0648\u0644</a></div>`}
    <a class="btn btn-ghost" style="margin-top:10px" href="#/create">${DGX.icon('plus','ic-s')} \u0645\u062d\u0635\u0648\u0644 \u062c\u062f\u06cc\u062f</a>`;
};

/* ── PAYOUTS — settlement center ── */
DGX.pages.payouts = async (view) => {
  if (!DGX.requireAuth('\u062a\u0633\u0648\u06cc\u0647')) { location.hash = '#/home'; return; }
  let w;
  try { w = await DGX.api('/api/app/wallet'); }
  catch (e) { view.innerHTML = `<div class="empty">${e.msg || '\u062e\u0637\u0627'}</div>`; return; }
  view.innerHTML = `
    <div class="post" style="padding:16px">
      <div class="sumrow"><span>\u0645\u0648\u062c\u0648\u062f\u06cc \u0642\u0627\u0628\u0644 \u0628\u0631\u062f\u0627\u0634\u062a</span><b class="num" style="color:var(--em)">${DGX.fmt(w.credits)} \u06a9\u0631\u062f\u06cc\u062a</b></div>
      <div class="sumrow"><span>\u0627\u0631\u0632\u0634 \u062a\u0642\u0631\u06cc\u0628\u06cc</span><b class="num">≈${(w.credits / (w.per_usdt || 100)).toFixed(2)}$</b></div>
      <div class="sumrow total"><span>\u06a9\u0644 \u062f\u0631\u0622\u0645\u062f \u062a\u0627 \u0627\u0645\u0631\u0648\u0632</span><b class="num">${DGX.fmt(w.earned || 0)}</b></div>
      <button class="btn btn-primary" onclick="DGX.openBot('/withdraw')">${DGX.icon('withdraw','ic-s')} \u062f\u0631\u062e\u0648\u0627\u0633\u062a \u0628\u0631\u062f\u0627\u0634\u062a \u0627\u0632 \u0628\u0627\u062a</button>
      <a class="btn btn-ghost" href="#/transactions">${DGX.icon('history','ic-s')} \u062a\u0627\u0631\u06cc\u062e\u0686\u0647‌\u06cc \u062a\u0631\u0627\u06a9\u0646\u0634‌\u0647\u0627</a>
    </div>
    <div class="post" style="padding:16px">
      <h3 style="margin-bottom:8px">${DGX.icon('shield','ic-s')} \u0642\u0648\u0627\u0646\u06cc\u0646 \u062a\u0633\u0648\u06cc\u0647</h3>
      <div class="sumrow"><span>\u062d\u062f\u0627\u0642\u0644 \u0628\u0631\u062f\u0627\u0634\u062a</span><b>\u06f1\u06f0\u06f0 \u06a9\u0631\u062f\u06cc\u062a</b></div>
      <div class="sumrow"><span>\u0632\u0645\u0627\u0646 \u067e\u0631\u062f\u0627\u0632\u0634</span><b>\u062a\u0627 \u06f2\u06f4 \u0633\u0627\u0639\u062a</b></div>
      <div class="sumrow"><span>\u06a9\u0627\u0631\u0645\u0632\u062f \u0628\u0631\u062f\u0627\u0634\u062a</span><b class="free">\u06f0 — \u0631\u0627\u06cc\u06af\u0627\u0646</b></div>
    </div>`;
};
