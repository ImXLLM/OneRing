/* DropAgentX v2.0 — SOCIAL SUITE: notifications / likes / saved / people /
   follows / profile-edit / share-center  (Instagram-like social surfaces) */
DGX.pages = DGX.pages || {};

/* ── NOTIFICATIONS — earnings + sales timeline (real data) ── */
DGX.pages.notifications = async (view) => {
  if (!DGX.requireAuth('\u0627\u0639\u0644\u0627\u0646‌\u0647\u0627')) { location.hash = '#/home'; return; }
  view.innerHTML = DGX.skelCards(2);
  let w, a;
  try { [w, a] = await Promise.all([DGX.api('/api/app/wallet'), DGX.api('/api/app/activity')]); }
  catch (e) { view.innerHTML = `<div class="empty">${e.msg || '\u062e\u0637\u0627'}</div>`; return; }
  const ev = [];
  (w.txs || []).slice(0, 12).forEach(t => ev.push({
    icon: { deposit: 'deposit', withdraw: 'withdraw', purchase: 'cart', sale: 'wallet',
            task_completion: 'check', referral_bonus: 'gift', admin_grant: 'shield',
            welcome: 'flame' }[t.tx_type] || 'wallet',
    title: ({ sale: '\u0641\u0631\u0648\u0634 \u062c\u062f\u06cc\u062f!', deposit: '\u0648\u0627\u0631\u06cc\u0632 \u0627\u0646\u062c\u0627\u0645 \u0634\u062f', purchase: '\u062e\u0631\u06cc\u062f \u0627\u0646\u062c\u0627\u0645 \u0634\u062f',
             withdraw: '\u0628\u0631\u062f\u0627\u0634\u062a \u062f\u0631 \u0635\u0641', task_completion: '\u062a\u06a9\u0645\u06cc\u0644 \u062a\u0633\u06a9',
             referral_bonus: '\u067e\u0627\u062f\u0627\u0634 \u0631\u06cc\u0641\u0631\u0627\u0644', admin_grant: '\u062a\u063a\u06cc\u06cc\u0631\u06cc \u0627\u0632 \u0627\u062f\u0645\u06cc\u0646',
             welcome: '\u062e\u0648\u0634 \u0622\u0645\u062f\u06cc!' })[t.tx_type] || t.tx_type,
    desc: t.description || '', t: t.created_at, amount: t.amount,
    hash: `#/transactions` }));
  (a.sold || []).slice(0, 8).forEach(o => ev.push({
    icon: 'shop', title: `${o.buyer || '\u06cc\u06a9 \u06a9\u0627\u0631\u0628\u0631'} «${o.title}» \u0631\u0627 \u062e\u0631\u06cc\u062f`,
    desc: '\u0648\u0627\u0631\u062f \u0633\u0628\u062f \u0641\u0631\u0648\u0634 \u062a\u0648 \u0634\u062f', t: o.purchased_at, amount: o.price_credits, hash: `#/order?id=${+o.id}` }));
  ev.sort((x, y) => (y.t || 0) - (x.t || 0));
  view.innerHTML = `
    <p class="page-hint">${DGX.icon('bell','ic-s')} ${ev.length} \u0631\u0648\u06cc\u062f\u0627\u062f \u0632\u0646\u062f\u0647 — \u0627\u0632 \u06a9\u06cc\u0641 \u067e\u0648\u0644 \u0648 \u0641\u0631\u0648\u0634‌\u0647\u0627\u06cc \u062a\u0648</p>
    ${ev.length ? ev.slice(0, 30).map(n => `
      <a class="notif-row" href="${n.hash || '#/wallet'}">
        <span class="nr-ic">${DGX.icon(n.icon)}</span>
        <div style="flex:1;min-width:0"><b>${DGX.esc(n.title)}</b>
          ${n.desc ? `<span class="p-sub">${DGX.esc(n.desc)}</span>` : ''}</div>
        <div style="text-align:end">
          ${n.amount != null ? `<b class="num" style="color:${(+n.amount || 0) >= 0 && n.icon === 'wallet' || n.icon === 'deposit' || n.icon === 'gift' ? 'var(--em)' : 'var(--txt)'}">${DGX.fmt(n.amount)}</b>` : ''}
          <span class="p-sub">${DGX.timeAgo(n.t || Date.now()/1000)}</span></div>
      </a>`).join('') : `<div class="empty"><div class="big">${DGX.icon('bell','big-ic')}</div>\u0647\u0646\u0648\u0632 \u062e\u0628\u0631\u06cc \u0646\u06cc\u0633\u062a<br><br>
        <a class="btn btn-primary" href="#/create">${DGX.icon('plus','ic-s')} \u0645\u062d\u0635\u0648\u0644 \u0628\u0633\u0627\u0632 \u062a\u0627 \u0641\u0631\u0648\u0634 \u0628\u06cc\u0627\u06cc\u062f</a></div>`}`;
};

/* ── LIKES / SAVED — local collections (recorded on this device) ── */
const _localGrid = (key, icon, title, hint, emptyCta) => async (view) => {
  const l = DGX.localList(key);
  view.innerHTML = `
    <p class="page-hint">${DGX.icon(icon,'ic-s')} ${hint}</p>
    ${l.length ? `<div class="mosaic">${l.map(x => `
      <a class="tile" href="#/product?id=${+x.id}">
        <div class="tile-img">${x.photo ? `<img src="${DGX.esc(x.photo)}" loading="lazy">` : DGX.icon('box','ic-xl ph-g')}</div>
        <b>${DGX.esc(x.title || '\u0645\u062d\u0635\u0648\u0644')}</b>
        <span class="num">${DGX.esc(x.price || '')}</span>
      </a>`).join('')}</div>
      <div style="text-align:center;padding:14px">
        <button class="lnk" id="clearL">\u062e\u0627\u0644\u06cc \u06a9\u0631\u062f\u0646 \u0641\u0647\u0631\u0633\u062a</button></div>`
      : `<div class="empty"><div class="big">${DGX.icon(icon,'big-ic')}</div>${title}<br><br>
         <a class="btn btn-primary" href="${emptyCta[1]}">${DGX.icon(emptyCta[2] || 'compass','ic-s')} ${emptyCta[0]}</a></div>`}`;
  const c = DGX.$('#clearL');
  if (c) c.onclick = () => { localStorage.removeItem(key); DGX.toast('\u062e\u0627\u0644\u06cc \u0634\u062f'); view.innerHTML = '<div class="empty">\u062e\u0627\u0644\u06cc \u0634\u062f — \u062f\u0648\u0628\u0627\u0631\u0647 \u067e\u0631\u0634 \u06a9\u0646 🌱</div>'; };
};
DGX.pages.likes = _localGrid('dgx_liked', 'heart', '\u0647\u0646\u0648\u0632 \u0686\u06cc\u0632\u06cc \u0631\u0627 \u0646\u067e\u0633\u0646\u062f\u06cc\u062f\u06cc', '\u067e\u0633\u0646\u062f\u0647\u0627\u06cc \u062a\u0648 \u0631\u0648\u06cc \u0627\u06cc\u0646 \u062f\u0633\u062a\u06af\u0627\u0647 — \u0628\u0627 \u062f\u0627\u0628\u0644‌\u062a\u067e \u0631\u0648\u06cc \u0639\u06a9\u0633‌\u0647\u0627 \u0647\u0645 \u0645\u06cc‌\u062a\u0648\u0646\u06cc \u067e\u0633\u0646\u062f \u06a9\u0646\u06cc', ['\u06af\u0634\u062a\u0646 \u062f\u0631 \u0641\u06cc\u062f', '#/home', 'flame']);
DGX.pages.saved = _localGrid('dgx_saved', 'save', '\u0647\u0646\u0648\u0632 \u0686\u06cc\u0632\u06cc \u0630\u062e\u06cc\u0631\u0647 \u0646\u06a9\u0631\u062f\u06cc', '\u0630\u062e\u06cc\u0631\u0647‌\u0634\u062f\u0647‌\u0647\u0627 \u0631\u0648\u06cc \u0627\u06cc\u0646 \u062f\u0633\u062a\u06af\u0627\u0647 — \u0627\u0632 \u062f\u06a9\u0645\u0647‌\u06cc \u0646\u0634\u0627\u0646\u06a9 \u0631\u0648\u06cc \u0647\u0631 \u06a9\u0627\u0631\u062a', ['\u06a9\u0634\u0641 \u0628\u0627\u0632\u0627\u0631', '#/explore', 'compass']);

/* ── PEOPLE — top creators + user search ── */
DGX.pages.people = async (view) => {
  view.innerHTML = DGX.skelCards(2);
  let lb, sr = null;
  try { lb = await DGX.api('/api/pub/leaderboard'); } catch (_) { lb = { items: [] }; }
  view.innerHTML = `
    <div class="search-bar2" style="margin-bottom:12px">
      ${DGX.icon('search','ic-s')}<input id="pq" placeholder="\u062c\u0633\u062a\u062c\u0648\u06cc \u06a9\u0627\u0631\u0628\u0631 \u0648 \u0641\u0631\u0648\u0634\u06af\u0627\u0647…">
    </div>
    <p class="page-hint">${DGX.icon('trophy','ic-s')} \u062c\u062f\u0648\u0644 \u0642\u0647\u0631\u0645\u0627\u0646\u0627\u0646 — \u0628\u06cc\u0634\u062a\u0631\u06cc\u0646 \u0641\u0631\u0648\u0634 \u062f\u0631 \u06a9\u0644 \u0628\u0627\u0632\u0627\u0631</p>
    <div id="lb">${lb.items.map((u, i) => `
      <a class="notif-row" href="#/seller?uid=${+u.user_id}">
        <span class="rank-badge ${i < 3 ? 'top' : ''}">${i + 1}</span>
        <img class="avatar" style="width:36px;height:36px" src="/app/assets/logo.jpg">
        <div style="flex:1;min-width:0"><b>${DGX.esc(u.name)}</b>
          <span class="p-sub">${DGX.kfmt(u.sold)} \u0641\u0631\u0648\u0634 · ${DGX.kfmt(u.credits)} \u06a9\u0631\u062f\u06cc\u062a</span></div>
        <span class="cc-go">${DGX.icon('chevron-l','ic-s')}</span>
      </a>`).join('') || '<div class="empty" style="padding:16px">\u0647\u0646\u0648\u0632 \u062c\u062f\u0648\u0644\u06cc \u0646\u06cc\u0633\u062a</div>'}</div>
    <div id="pRes"></div>`;
  const q = DGX.$('#pq'), res = DGX.$('#pRes');
  let tm = null;
  q.oninput = () => {
    clearTimeout(tm);
    tm = setTimeout(async () => {
      if (q.value.trim().length < 2) { res.innerHTML = ''; return; }
      try {
        const d = await DGX.api(`/api/app/search?q=${encodeURIComponent(q.value.trim())}`);
        res.innerHTML = (d.users || []).length ? `<p class="page-hint" style="margin-top:12px">\u06a9\u0627\u0631\u0628\u0631\u0627\u0646</p>` +
          d.users.map(u => `
            <a class="notif-row" href="#/seller?uid=${+u.user_id}">
              <img class="avatar" style="width:36px;height:36px" src="/app/assets/logo.jpg">
              <div style="flex:1"><b>${DGX.esc(u.first_name || '')}</b>
                <span class="p-sub">@${DGX.esc(u.username || '')}</span></div>
              <span class="cc-go">${DGX.icon('chevron-l','ic-s')}</span></a>`).join('') : '';
      } catch (_) {}
    }, 350);
  };
};

/* ── FOLLOWS — stores you follow (server truth) + suggestions ── */
DGX.pages.follows = async (view) => {
  if (!DGX.requireAuth('\u0641\u0627\u0644\u0648\u0647\u0627')) { location.hash = '#/home'; return; }
  view.innerHTML = DGX.skelCards(2);
  let ids, lb;
  try { [ids, lb] = await Promise.all([DGX.api('/api/app/following-ids'), DGX.api('/api/pub/leaderboard')]); }
  catch (e) { view.innerHTML = `<div class="empty">${e.msg || '\u062e\u0637\u0627'}</div>`; return; }
  const list = ids.items || ids.ids || ids || [];
  const arr = Array.isArray(list) ? list : [];
  view.innerHTML = `
    <p class="page-hint">${DGX.icon('users','ic-s')} ${arr.length} \u0641\u0631\u0648\u0634\u06af\u0627\u0647 \u062f\u0646\u0628\u0627\u0644 \u0645\u06cc‌\u06a9\u0646\u06cc — \u0641\u06cc\u062f «\u0641\u0627\u0644\u0648\u06cc\u0646\u06af‌\u0647\u0627» \u0627\u0632 \u0647\u0645\u06cc\u0646‌\u0647\u0627 \u0633\u0627\u062e\u062a\u0647 \u0645\u06cc‌\u0634\u0648\u062f</p>
    <div id="fl">${arr.length ? '' : `<div class="empty"><div class="big">${DGX.icon('users','big-ic')}</div>\u0647\u0646\u0648\u0632 \u06a9\u0633\u06cc \u0631\u0627 \u062f\u0646\u0628\u0627\u0644 \u0646\u06a9\u0631\u062f\u0647‌\u0627\u06cc<br><br>
      <a class="btn btn-primary" href="#/people">${DGX.icon('trophy','ic-s')} \u062e\u0627\u0644\u0642‌\u0647\u0627\u06cc \u0628\u0631\u062a\u0631</a></div>`}</div>
    <div id="flCards"></div>
    ${arr.length ? '' : `<p class="page-hint" style="margin-top:8px">\u067e\u06cc\u0634\u0646\u0647\u0627\u062f \u0645\u0627</p>
      <div>${(lb.items || []).slice(0, 5).map(u => `
        <a class="notif-row" href="#/seller?uid=${+u.user_id}">
          <span class="nr-ic">${DGX.icon('user')}</span>
          <div style="flex:1"><b>${DGX.esc(u.name)}</b><span class="p-sub">${DGX.kfmt(u.sold)} \u0641\u0631\u0648\u0634</span></div>
          <span class="cc-go">${DGX.icon('chevron-l','ic-s')}</span></a>`).join('')}</div>`}`;
  const box = DGX.$('#flCards');
  for (const uid of arr.slice(0, 20)) {
    try {
      const s = await DGX.api(`/api/app/store/${+uid}`);
      box.insertAdjacentHTML('beforeend', `
        <a class="notif-row" href="#/seller?uid=${+uid}">
          ${s.avatar_url ? `<img class="avatar" style="width:38px;height:38px" src="${DGX.esc(s.avatar_url)}">` : `<span class="nr-ic">${DGX.icon('user')}</span>`}
          <div style="flex:1;min-width:0"><b>${DGX.esc(s.name)}</b>
            <span class="p-sub">${DGX.kfmt(s.products.length)} \u0645\u062d\u0635\u0648\u0644 · ${DGX.kfmt(s.total_sales)} \u0641\u0631\u0648\u0634</span></div>
          <span class="cc-go">${DGX.icon('chevron-l','ic-s')}</span></a>`);
    } catch (_) {}
  }
};

/* ── PROFILE-EDIT — avatar/cover via real API + local prefs ── */
DGX.pages['profile-edit'] = async (view) => {
  if (!DGX.requireAuth('\u0648\u06cc\u0631\u0627\u06cc\u0634')) { location.hash = '#/profile'; return; }
  const u = DGX.user || {};
  view.innerHTML = `
    <p class="page-hint">${DGX.icon('image','ic-s')} \u0639\u06a9\u0633‌\u0647\u0627 \u0645\u0633\u062a\u0642\u06cc\u0645 \u0631\u0648\u06cc \u0633\u0631\u0648\u0631 \u0630\u062e\u06cc\u0631\u0647 \u0645\u06cc‌\u0634\u0648\u0646\u062f — \u0646\u0627\u0645 \u0648 \u0628\u0627\u06cc\u0648 \u0627\u0632 \u0628\u0627\u062a</p>
    <div class="post" style="padding:16px">
      <div class="field"><label>\u0622\u0648\u0627\u062a\u0627\u0631</label>
        <div class="post-actions" style="padding:0">
          <button class="btn btn-ghost" onclick="DGX.upPhoto('avatar')">${DGX.icon('image','ic-s')} \u0627\u0646\u062a\u062e\u0627\u0628 \u0622\u0648\u0627\u062a\u0627\u0631</button></div></div>
      <div class="field"><label>\u06a9\u0627\u0648\u0631 \u067e\u0631\u0648\u0641\u0627\u06cc\u0644</label>
        <div class="post-actions" style="padding:0">
          <button class="btn btn-ghost" onclick="DGX.upPhoto('cover')">${DGX.icon('palette','ic-s')} \u0627\u0646\u062a\u062e\u0627\u0628 \u06a9\u0627\u0648\u0631</button></div></div>
      <div class="field"><label>\u0646\u0627\u0645 \u0646\u0645\u0627\u06cc\u0634\u06cc (\u0627\u0632 \u0628\u0627\u062a)</label>
        <input value="${DGX.esc(u.name || '')}" disabled style="opacity:.6">
      </div>
      <a class="btn btn-primary" href="#" onclick="DGX.openBot('/profile');return false">${DGX.icon('send','ic-s')} \u062a\u063a\u06cc\u06cc\u0631 \u0646\u0627\u0645 \u0627\u0632 \u0628\u0627\u062a</a>
    </div>
    <div class="post" style="padding:16px">
      <h3 style="margin-bottom:8px">${DGX.icon('settings','ic-s')} \u062a\u0631\u062c\u06cc\u062d\u0627\u062a \u0627\u06cc\u0646 \u062f\u0633\u062a\u06af\u0627\u0647</h3>
      <div class="set-row" id="setHaptic"><span class="nr-ic">${DGX.icon('live')}</span>
        <div style="flex:1"><b>\u0644\u0631\u0632\u0634 \u0647\u0646\u06af\u0627\u0645 \u0644\u0645\u0633</b><span class="p-sub">haptic feedback</span></div>
        <b id="hv" style="color:var(--em)"></b></div>
      <div class="set-row" onclick="DGX.toast('\u062a\u0645 NEON \u062a\u0646\u0647\u0627 \u062a\u0645 \u0631\u0633\u0645\u06cc v2.0 \u0627\u0633\u062a')">
        <span class="nr-ic">${DGX.icon('palette')}</span>
        <div style="flex:1"><b>\u062a\u0645</b><span class="p-sub">NEON — \u067e\u06cc\u0634‌\u0641\u0631\u0636 \u0628\u0631\u0646\u062f</span></div>
        <b style="color:var(--em)">\u0641\u0639\u0627\u0644</b></div>
    </div>
    <div style="text-align:center;padding:10px"><a class="lnk" href="#/profile">← \u0628\u0631\u06af\u0634\u062a \u0628\u0647 \u067e\u0631\u0648\u0641\u0627\u06cc\u0644</a></div>`;
  const h = DGX.$('#setHaptic'), hv = DGX.$('#hv');
  const sync = () => hv.textContent = localStorage.getItem('dgx_haptic') === '0' ? '\u062e\u0627\u0645\u0648\u0634' : '\u0631\u0648\u0634\u0646';
  sync();
  h.onclick = () => {
    const off = localStorage.getItem('dgx_haptic') !== '0';
    localStorage.setItem('dgx_haptic', off ? '0' : '1');
    if (!off) DGX.haptic('medium');
    sync();
  };
};

/* ── SHARE CENTER — growth hub (referral + share) ── */
DGX.pages.share = async (view) => {
  const u = DGX.user || {};
  const link = `${location.origin}/#/home`;
  view.innerHTML = `
    <p class="page-hint">${DGX.icon('link','ic-s')} \u0647\u0631 \u062f\u0648\u0633\u062a \u06a9\u0647 \u0628\u0627 \u0644\u06cc\u0646\u06a9 \u062a\u0648 \u0628\u06cc\u0627\u062f \u0648 \u062e\u0631\u06cc\u062f \u06a9\u0646\u0647\u060c \u0647\u0631 \u062f\u0648\u062a\u0648\u0646 \u067e\u0627\u062f\u0627\u0634 \u0645\u06cc‌\u06af\u06cc\u0631\u06cc\u062f — \u062d\u0644\u0642\u0647‌\u06cc \u0631\u0634\u062f DropAgentX</p>
    <div class="post" style="padding:16px;text-align:center">
      <div class="burst-wrap" style="width:64px;height:64px"><span class="success-ring">${DGX.icon('gift')}</span></div>
      <b style="font-size:16px">\u062f\u0639\u0648\u062a = \u06a9\u0631\u062f\u06cc\u062a \u0631\u0627\u06cc\u06af\u0627\u0646</b>
      <p class="p-sub" style="margin:6px 0 12px">\u0644\u06cc\u0646\u06a9 \u0631\u0627 \u062f\u0631 \u0647\u0631 \u0686\u062a \u062a\u0644\u06af\u0631\u0627\u0645\u06cc \u0628\u0641\u0631\u0633\u062a — \u0628\u0642\u06cc\u0647‌\u0627\u0634 \u0628\u0627 \u0628\u0627\u062a</p>
      <button class="btn btn-primary" id="shMain">${DGX.icon('send','ic-s')} \u0627\u0634\u062a\u0631\u0627\u06a9 \u062f\u0631 \u062a\u0644\u06af\u0631\u0627\u0645</button>
      <button class="btn btn-ghost" id="shCopy">${DGX.icon('link','ic-s')} \u06a9\u067e\u06cc \u0644\u06cc\u0646\u06a9</button>
    </div>
    <div class="col-grid">
      <div class="col-card"><span class="cc-ic">${DGX.icon('send')}</span><b>\u0627\u0634\u062a\u0631\u0627\u06a9 \u0645\u062d\u0635\u0648\u0644</b><span>\u0627\u0632 \u062f\u06a9\u0645\u0647‌\u06cc \u0627\u0634\u062a\u0631\u0627\u06a9 \u0631\u0648\u06cc \u0647\u0631 \u06a9\u0627\u0631\u062a</span></div>
      <div class="col-card" onclick="DGX.openBot('/referral')"><span class="cc-ic">${DGX.icon('gift')}</span><b>\u06a9\u062f \u0631\u06cc\u0641\u0631\u0627\u0644 \u0645\u0646</b><span>\u0645\u0634\u0627\u0647\u062f\u0647‌\u06cc \u067e\u0627\u062f\u0627\u0634‌\u0647\u0627 \u062f\u0631 \u0628\u0627\u062a</span></div>
    </div>`;
  DGX.$('#shMain').onclick = () => DGX.shareUrl(link, '\u0628\u06cc\u0627 \u062a\u0648 DropAgentX — \u0628\u0627\u0632\u0627\u0631 \u0645\u062d\u0635\u0648\u0644\u0627\u062a \u062f\u06cc\u062c\u06cc\u062a\u0627\u0644 \u062f\u0627\u062e\u0644 \u062a\u0644\u06af\u0631\u0627\u0645 🛍');
  DGX.$('#shCopy').onclick = async () => {
    try { await navigator.clipboard.writeText(link); DGX.toast('\u06a9\u067e\u06cc \u0634\u062f ✅'); }
    catch (_) { DGX.toast(link, true); }
  };
};
