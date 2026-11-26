/* DropAgentX v2.0 — COMMERCE SUITE: reviews / seller / related / library / order / plans */
DGX.pages = DGX.pages || {};

/* ── REVIEWS — full comment list of a product ── */
DGX.pages.reviews = async (view, params) => {
  const pid = +params.id; if (!pid) { location.hash = '#/home'; return; }
  view.innerHTML = DGX.skelCards(2);
  let d;
  try { d = await DGX.api(`/api/app/product/${pid}`); }
  catch (e) { view.innerHTML = `<div class="empty">${e.msg || '\u062e\u0637\u0627'}</div>`; return; }
  const p = d.item, cms = d.comments || [];
  view.innerHTML = `
    <div class="col-head">${DGX.icon('comment')} <b>\u062f\u06cc\u062f\u06af\u0627\u0647‌\u0647\u0627</b>
      <span>${p.title} · ${DGX.icon('star-f','ic-s star')} ${d.stars} \u0627\u0632 ${d.reviews} \u0646\u0638\u0631</span></div>
    ${cms.length ? cms.map(c => `
      <div class="notif-row">
        <img class="avatar" style="width:34px;height:34px" src="/app/assets/logo.jpg">
        <div style="flex:1;min-width:0">
          <b>${DGX.esc(c.name || c.username || '\u06a9\u0627\u0631\u0628\u0631')}</b>
          <p style="color:var(--dim);font-size:12.5px;line-height:1.8;margin-top:2px">${DGX.esc(c.text)}</p>
        </div>
        <span class="p-sub">${DGX.timeAgo(c.created_at || Date.now()/1000)}</span>
      </div>`).join('') : `<div class="empty"><div class="big">${DGX.icon('comment','big-ic')}</div>\u0627\u0648\u0644\u06cc\u0646 \u062f\u06cc\u062f\u06af\u0627\u0647 \u0631\u0627 \u0628\u0646\u0648\u06cc\u0633<br><br>
        <a class="btn btn-primary" href="#/product?id=${pid}">\u0631\u0641\u062a\u0646 \u0628\u0647 \u0645\u062d\u0635\u0648\u0644</a></div>`}
    <a class="lnk" style="display:block;text-align:center;padding:10px" href="#/product?id=${pid}">← \u0635\u0641\u062d\u0647‌\u06cc \u0645\u062d\u0635\u0648\u0644</a>`;
};

/* ── SELLER — public storefront (0xBazaar #19 DNA) ── */
DGX.pages.seller = async (view, params) => {
  const uid = +params.uid; if (!uid) { location.hash = '#/people'; return; }
  view.innerHTML = DGX.skelCards(2);
  let s;
  try { s = await DGX.api(`/api/app/store/${uid}`); }
  catch (e) { view.innerHTML = `<div class="empty">${e.msg || '\u0641\u0631\u0648\u0634\u06af\u0627\u0647 \u067e\u06cc\u062f\u0627 \u0646\u0634\u062f'}</div>`; return; }
  const isMe = s.is_me;
  view.innerHTML = `
    <div class="store-hero">
      ${s.cover_url ? `<img class="cover" src="${DGX.esc(s.cover_url)}">` : '<div class="cover"></div>'}
      ${s.avatar_url ? `<img class="logo" src="${DGX.esc(s.avatar_url)}">`
                     : `<span class="logo">${DGX.icon('user')}</span>`}
      <b>${DGX.esc(s.name || '\u0641\u0631\u0648\u0634\u06af\u0627\u0647')}</b>
      <span class="p-sub">@${DGX.esc(s.username || '')} ${DGX.icon('check','ic-xs vf')}</span>
      <div class="store-stats">
        <span><b class="num">${DGX.kfmt(s.followers)}</b> \u0641\u0627\u0644\u0648\u0648\u0631</span>
        <span><b class="num">${DGX.kfmt(s.products.length)}</b> \u0645\u062d\u0635\u0648\u0644</span>
        <span><b class="num">${DGX.kfmt(s.total_sales)}</b> \u0641\u0631\u0648\u0634</span>
      </div>
      <div class="post-actions" style="padding:12px 0 0">
        ${isMe
          ? `<a class="btn btn-ghost" href="#/profile">${DGX.icon('user','ic-s')} \u067e\u0631\u0648\u0641\u0627\u06cc\u0644 \u0645\u0646</a>`
          : `<button class="btn ${s.following_me ? 'btn-ghost' : 'btn-primary'}" id="folBtn">
               ${s.following_me ? DGX.icon('check','ic-s') + ' \u0641\u0627\u0644\u0648 \u0645\u06cc‌\u06a9\u0646\u06cc' : '+ \u0641\u0627\u0644\u0648'}</button>`}
        <button class="btn btn-ghost" data-share="1">${DGX.icon('link','ic-s')} \u0627\u0634\u062a\u0631\u0627\u06a9 \u0641\u0631\u0648\u0634\u06af\u0627\u0647</button>
      </div>
    </div>
    <h3 style="margin:16px 0 10px;display:flex;align-items:center;gap:6px">${DGX.icon('shop','ic-s')} \u0648\u06cc\u062a\u0631\u06cc\u0646</h3>
    <div class="mosaic">${s.products.map(DGX.tile).join('') ||
      `<div class="empty" style="grid-column:1/-1">\u0647\u0646\u0648\u0632 \u0645\u062d\u0635\u0648\u0644\u06cc \u0646\u062f\u0627\u0631\u062f</div>`}</div>`;
  const fb = DGX.$('#folBtn');
  if (fb) fb.onclick = async () => {
    if (!DGX.requireAuth('\u0641\u0627\u0644\u0648')) return;
    const on = !fb.dataset.on; fb.dataset.on = on ? '1' : '';
    try {
      await DGX.api('/api/app/follow', { body: { target: uid, on: !!on } });
      fb.className = 'btn ' + (on ? 'btn-ghost' : 'btn-primary');
      fb.innerHTML = on ? DGX.icon('check','ic-s') + ' \u0641\u0627\u0644\u0648 \u0645\u06cc‌\u06a9\u0646\u06cc' : '+ \u0641\u0627\u0644\u0648';
      DGX.toast(on ? '\u0641\u0627\u0644\u0648 \u0634\u062f\u06cc ✅' : '\u0622\u0646\u0641\u0627\u0644\u0648 \u0634\u062f\u06cc');
    } catch (e) { DGX.toast(e.msg || '', true); }
  };
};

/* ── RELATED — same-category products ── */
DGX.pages.related = async (view, params) => {
  const pid = +params.id; if (!pid) { location.hash = '#/home'; return; }
  view.innerHTML = DGX.skelCards(2);
  let d;
  try { d = await DGX.api(`/api/app/product/${pid}`); }
  catch (e) { view.innerHTML = `<div class="empty">${e.msg || '\u062e\u0637\u0627'}</div>`; return; }
  const p = d.item;
  try {
    const f = await DGX.api(`/api/app/feed?mode=foryou&cat=${encodeURIComponent(p.category || 'all')}&cursor=0`);
    const rows = f.items.filter(x => +x.id !== pid).slice(0, 12);
    view.innerHTML = `
      <div class="col-head">${DGX.icon('puzzle')} <b>\u0645\u0634\u0627\u0628\u0647‌\u0647\u0627\u06cc «${DGX.esc(p.title)}»</b>
        <span>\u062f\u0633\u062a\u0647‌\u06cc ${DGX.esc(p.category)}</span></div>
      <div class="mosaic">${rows.map(DGX.tile).join('') ||
        `<div class="empty" style="grid-column:1/-1"><div class="big">${DGX.icon('box','big-ic')}</div>\u0645\u0634\u0627\u0628\u0647\u06cc \u067e\u06cc\u062f\u0627 \u0646\u0634\u062f</div>`}</div>
      <div style="text-align:center;padding:12px"><a class="lnk" href="#/product?id=${pid}">← \u0628\u0631\u06af\u0634\u062a \u0628\u0647 \u0645\u062d\u0635\u0648\u0644</a></div>`;
  } catch (e) { view.innerHTML = `<div class="empty">${e.msg || '\u062e\u0637\u0627'}</div>`; }
};

/* ── LIBRARY — purchased products (0xBazaar #35 DNA) ── */
DGX.pages.library = async (view) => {
  if (!DGX.requireAuth('\u06a9\u062a\u0627\u0628\u062e\u0627\u0646\u0647')) { location.hash = '#/home'; return; }
  view.innerHTML = DGX.skelCards(2);
  let a;
  try { a = await DGX.api('/api/app/activity'); }
  catch (e) { view.innerHTML = `<div class="empty">${e.msg || '\u062e\u0637\u0627'}</div>`; return; }
  const b = a.bought || [];
  view.innerHTML = `
    <p class="page-hint">${DGX.icon('book','ic-s')} ${b.length} \u0645\u062d\u0635\u0648\u0644 \u062f\u06cc\u062c\u06cc\u062a\u0627\u0644 \u062f\u0631 \u0645\u0627\u0644\u06a9\u06cc\u062a \u062a\u0648 — \u0647\u0631 \u0648\u0642\u062a \u062e\u0648\u0627\u0633\u062a\u06cc \u062f\u0627\u0646\u0644\u0648\u062f \u06a9\u0646</p>
    ${b.length ? b.map(o => `
      <div class="notif-row own-row">
        <span class="rr-img">${DGX.icon('box')}</span>
        <div style="flex:1;min-width:0">
          <b>${DGX.esc(o.title)}</b>
          <span class="p-sub">${DGX.timeAgo(o.purchased_at || Date.now()/1000)} · <span class="pill p-ok" style="font-size:9.5px">\u0645\u0627\u0644\u06a9 \u062a\u0648</span></span>
        </div>
        ${o.download_url ? `<a class="btn btn-primary" style="flex:none;padding:8px 12px" href="${DGX.esc(o.download_url)}">${DGX.icon('deposit','ic-s')} \u062f\u0627\u0646\u0644\u0648\u062f</a>` : ''}
      </div>`).join('') : `<div class="empty"><div class="big">${DGX.icon('book','big-ic')}</div>\u06a9\u062a\u0627\u0628\u062e\u0627\u0646\u0647‌\u0627\u062a \u062e\u0627\u0644\u06cc\u0647<br><br>
        <a class="btn btn-primary" href="#/explore">${DGX.icon('compass','ic-s')} \u06af\u0634\u062a\u0646 \u062f\u0631 \u0628\u0627\u0632\u0627\u0631</a></div>`}
    <div style="text-align:center;padding:12px"><a class="lnk" href="#/activity">\u062a\u0627\u0631\u06cc\u062e\u0686\u0647‌\u06cc \u06a9\u0627\u0645\u0644 \u062e\u0631\u06cc\u062f/\u0641\u0631\u0648\u0634 ←</a></div>`;
};

/* ── ORDER — single order detail ── */
DGX.pages.order = async (view, params) => {
  const oid = +params.id; if (!oid) { location.hash = '#/activity'; return; }
  let a;
  try { a = await DGX.api('/api/app/activity'); }
  catch (e) { view.innerHTML = `<div class="empty">${e.msg || '\u062e\u0637\u0627'}</div>`; return; }
  const o = (a.bought || []).find(x => +x.id === oid) || (a.sold || []).find(x => +x.id === oid);
  const isSold = o && !(a.bought || []).some(x => +x.id === oid);
  if (!o) { view.innerHTML = `<div class="empty"><div class="big">${DGX.icon('doc','big-ic')}</div>\u0633\u0641\u0627\u0631\u0634 \u067e\u06cc\u062f\u0627 \u0646\u0634\u062f<br><br>
    <a class="btn btn-ghost" href="#/activity">\u062a\u0627\u0631\u06cc\u062e\u0686\u0647‌\u06cc \u0641\u0639\u0627\u0644\u06cc\u062a</a></div>`; return; }
  view.innerHTML = `
    <div class="col-head">${DGX.icon('doc')} <b>\u0633\u0641\u0627\u0631\u0634 #${oid}</b><span>${isSold ? '\u0641\u0631\u0648\u0634 \u062a\u0648' : '\u062e\u0631\u06cc\u062f \u062a\u0648'}</span></div>
    <div class="post"><div class="post-body" style="padding:16px">
      <div class="sumrow"><span>\u0645\u062d\u0635\u0648\u0644</span><b>${DGX.esc(o.title)}</b></div>
      ${isSold ? `<div class="sumrow"><span>\u062e\u0631\u06cc\u062f\u0627\u0631</span><b>${DGX.esc(o.buyer || '\u06a9\u0627\u0631\u0628\u0631')}</b></div>` : ''}
      <div class="sumrow"><span>\u0632\u0645\u0627\u0646</span><b>${DGX.timeAgo(o.purchased_at || Date.now()/1000)}</b></div>
      <div class="sumrow"><span>\u0645\u0628\u0644\u063a</span><b class="num">${DGX.fmt(o.price_credits)} \u06a9\u0631\u062f\u06cc\u062a</b></div>
      <div class="sumrow total"><span>\u0648\u0636\u0639\u06cc\u062a</span><b style="font-size:13px">\u062a\u06a9\u0645\u06cc\u0644‌\u0634\u062f\u0647 ${DGX.icon('check','ic-s')}</b></div>
      ${(!isSold && o.download_url) ? `<a class="btn btn-primary" href="${DGX.esc(o.download_url)}">${DGX.icon('deposit','ic-s')} \u062f\u0627\u0646\u0644\u0648\u062f \u062f\u0648\u0628\u0627\u0631\u0647</a>` : ''}
      <a class="btn btn-ghost" href="#/library">${DGX.icon('book','ic-s')} \u06a9\u062a\u0627\u0628\u062e\u0627\u0646\u0647‌\u06cc \u0645\u0646</a>
    </div></div>`;
};

/* ── PLANS — seller subscription tiers (0xBazaar #38 DNA) ── */
DGX.pages.plans = async (view) => {
  const P = [
    ['hunted', '\u0647\u0627\u0646\u062a\u0631', 0, ['\u06a9\u0645\u06cc\u0633\u06cc\u0648\u0646 \u0627\u0633\u062a\u0627\u0646\u062f\u0627\u0631\u062f', '\u06f5 \u0645\u062d\u0635\u0648\u0644 \u0641\u0639\u0627\u0644', '\u0641\u0631\u0648\u0634 \u0627\u0632 \u0637\u0631\u06cc\u0642 \u0628\u0627\u062a'], false],
    ['creator', '\u0633\u0627\u0632\u0646\u062f\u0647', 0, ['\u0628\u062f\u0648\u0646 \u0647\u0632\u06cc\u0646\u0647‌\u06cc \u0645\u0627\u0647\u0627\u0646\u0647', '\u0645\u062d\u0635\u0648\u0644 \u0646\u0627\u0645\u062d\u062f\u0648\u062f', '\u0641\u0631\u0648\u0634\u06af\u0627\u0647 \u0639\u0645\u0648\u0645\u06cc', '\u062f\u0631\u0627\u067e \u0648\u06cc\u0698\u0647 \u0628\u0627 \u062a\u0623\u06cc\u06cc\u062f \u0627\u062f\u0645\u06cc\u0646'], true],
    ['studio', '\u0627\u0633\u062a\u0648\u062f\u06cc\u0648', 50, ['\u0647\u0645\u0647‌\u06cc \u0645\u0632\u0627\u06cc\u0627\u06cc \u0633\u0627\u0632\u0646\u062f\u0647', '\u0627\u0648\u0644\u0648\u06cc\u062a \u0628\u0631\u0631\u0633\u06cc \u0645\u062d\u0635\u0648\u0644', '\u0646\u0645\u0627\u06cc\u0634 \u0648\u06cc\u0698\u0647 \u062f\u0631 \u062a\u0631\u0646\u062f\u0647\u0627', '\u067e\u0634\u062a\u06cc\u0628\u0627\u0646\u06cc \u0633\u0631\u06cc\u0639'], false],
  ];
  view.innerHTML = `
    <p class="page-hint">${DGX.icon('rocket','ic-s')} \u0641\u0631\u0648\u0634 \u062f\u0631 DropAgentX \u0628\u0631\u0627\u06cc \u0647\u0645\u0647 \u0631\u0627\u06cc\u06af\u0627\u0646 \u0627\u0633\u062a — \u067e\u0644\u0646‌\u0647\u0627 \u0641\u0642\u0637 \u0628\u0648\u0646\u0648\u0633 \u062f\u06cc\u062f\u0647‌\u0634\u062f\u0646 \u0645\u06cc‌\u062f\u0647\u0646\u062f</p>
    <div class="plan-list">
      ${P.map(([k, t, price, feats, hot]) => `
        <div class="plan-card ${hot ? 'hot' : ''}">
          ${hot ? `<span class="plan-tag">\u0641\u0639\u0644\u06cc \u062a\u0648</span>` : ''}
          <b>${t}</b>
          <span class="plan-price num">${price ? price + ' \u06a9\u0631\u062f\u06cc\u062a/\u0645\u0627\u0647' : '\u0631\u0627\u06cc\u06af\u0627\u0646'}</span>
          <ul>${feats.map(f => `<li>${DGX.icon('check','ic-xs')} ${f}</li>`).join('')}</ul>
          <button class="btn ${hot ? 'btn-ghost' : 'btn-primary'}" onclick="DGX.openBot('/start')">
            ${DGX.icon('send','ic-s')} ${hot ? '\u0627\u062f\u0627\u0645\u0647‌\u06cc \u0641\u0631\u0648\u0634' : '\u0627\u0631\u062a\u0642\u0627 \u0627\u0632 \u0628\u0627\u062a'}</button>
        </div>`).join('')}
    </div>
    <p class="page-hint" style="margin-top:12px">\u06a9\u0627\u0631\u0645\u0632\u062f \u0628\u0627\u0632\u0627\u0631 \u062f\u0631 \u0647\u0645\u0647‌\u06cc \u067e\u0644\u0646‌\u0647\u0627 \u06cc\u06a9\u0633\u0627\u0646 \u0648 \u0634\u0641\u0627\u0641 \u0627\u0633\u062a — \u062c\u0632\u0626\u06cc\u0627\u062a \u062f\u0631 \u0628\u0627\u062a: /start → \u0641\u0631\u0648\u0634\u06af\u0627\u0647</p>`;
};
