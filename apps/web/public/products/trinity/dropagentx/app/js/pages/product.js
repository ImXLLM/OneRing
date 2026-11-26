/* DropAgentX — PRODUCT detail: hero, comments, buy with game-motion */
DGX.pages = DGX.pages || {};

DGX.pages.product = async (view, params) => {
  const pid = +params.id;
  if (!pid) { location.hash = '#/home'; return; }
  view.innerHTML = `<div class="skel" style="height:260px"></div>
    <div class="skel" style="height:60px;margin-top:12px"></div>
    <div class="buybar"><span style="flex:1;color:var(--dim);font-size:13px">…</span></div>`;

  const d = await DGX.api('/api/app/product/' + pid);
  const p = d.item, per = DGX.perUsdt || 1000;
  const owned = params.owned;

  view.innerHTML = `
    <a href="#/home" style="color:var(--dim);font-size:12px;text-decoration:none">‹ \u0628\u0631\u06af\u0634\u062a \u0628\u0647 \u0641\u06cc\u062f</a>
    <img class="hero burst" src="${p.photo_url ? DGX.esc(p.photo_url) : ''}"
         onerror="this.src=''" alt="" style="${p.photo_url ? '' : 'display:flex;align-items:center;justify-content:center;font-size:64px'}">
    <h1 style="font-size:19px;margin:14px 0 6px;line-height:1.7">${DGX.esc(p.title)}</h1>
    <div style="display:flex;align-items:center;gap:8px;margin-bottom:12px">
      <span class="pill p-ok" style="font-size:11px">${DGX.icon('star-f','ic-s star')} ${d.stars} (${d.reviews} \u062f\u06cc\u062f\u06af\u0627\u0647)</span>
      <span style="color:var(--dim);font-size:12px">${DGX.icon('cart','ic-s')} ${DGX.kfmt(p.sales_count)} \u0641\u0631\u0648\u0634</span>
      <span style="color:var(--dim2);font-size:12px">${DGX.icon('eye','ic-s')} ${DGX.kfmt(p.views)}</span>
    </div>
    <div style="background:var(--surface);border:1px solid var(--line);border-radius:var(--r-m);
                padding:13px 15px;font-size:13.5px;line-height:2.1;color:#dbe3dd">
      ${(p.description || '—').replace(/\n/g, '<br>')}
    </div>

    <h3 style="margin:18px 0 8px">${DGX.icon('comment','ic-s')} \u062f\u06cc\u062f\u06af\u0627\u0647‌\u0647\u0627 (${p.comment_count || 0})</h3>
    <div id="cList">${(d.comments || []).map(c => `
      <div class="comment">
        <img class="avatar" src="/app/assets/logo.jpg">
        <div><span class="c-name">${DGX.esc(c.first_name || c.username || '\u06a9\u0627\u0631\u0628\u0631')}</span>
          <span class="c-time">${DGX.timeAgo(c.created_at)}</span>
          <div class="c-text">${DGX.esc(c.text)}</div></div>
      </div>`).join('') || '<div style="color:var(--dim);font-size:12.5px">\u0627\u0648\u0644\u06cc\u0646 \u0646\u0638\u0631 \u0631\u0648 \u0628\u0630\u0627\u0631 💬</div>'}
    </div>
    <div class="search-bar2" style="margin-top:10px;padding:8px 10px">
      <input id="cIn" placeholder="\u0646\u0638\u0631\u062a \u0631\u0627 \u0628\u0646\u0648\u06cc\u0633…" maxlength="500" enterkeyhint="send"
             style="padding:8px;background:none;border:0">
      <button class="btn btn-primary" id="cSend" style="flex:none;padding:9px 16px">\u0627\u0631\u0633\u0627\u0644</button>
    </div>

    <div style="height:70px"></div>
    <div class="buybar">
      <div>
        <b class="num" style="color:var(--em);font-size:17px">${DGX.fmt(p.price_credits)} \u06a9\u0631\u062f\u06cc\u062a</b><br>
        <small style="color:var(--dim)">≈${(+p.usd).toFixed(2)}$</small>
      </div>
      ${owned ? `<button class="btn btn-ghost" onclick="history.back()">✓ \u062e\u0631\u06cc\u062f\u0647‌\u0627\u06cc</button>`
              : `<button class="btn btn-primary" id="buyNow" style="flex:1">${DGX.icon('cart','ic-s')} \u062e\u0631\u06cc\u062f \u0641\u0648\u0631\u06cc</button>`}
      <button class="icon-btn" onclick="DGX.shareProduct(${pid})" title="\u0627\u0634\u062a\u0631\u0627\u06a9" aria-label="\u0627\u0634\u062a\u0631\u0627\u06a9">
        <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/><path d="m8.6 13.5 6.8 4M15.4 6.5l-6.8 4"/></svg>
      </button>
    </div>`;

  // buy with celebration
  /* v1.3: same neon confirm-sheet flow as the feed */
  const buyBtn = DGX.$('#buyNow');
  if (buyBtn) buyBtn.onclick = () => {
    if (!DGX.requireAuth('\u062e\u0631\u06cc\u062f')) return;
    DGX.confirmBuy({ id: +p.id, title: p.title, price_credits: +p.price_credits || 0,
                     seller: (d && d.store_name) || p.store_name || '',
                     photo_url: p.photo_url || '' },
      () => { buyBtn.outerHTML = `
        <div style="flex:1;text-align:center">
          <div class="burst success-ring" style="width:44px;height:44px;border-radius:99px;
            background:var(--em);color:#03130a;display:inline-flex;align-items:center;
            justify-content:center">${DGX.icon('check')}</div>
          <div style="font-size:12px;color:var(--em);margin-top:4px">\u062e\u0631\u06cc\u062f \u0645\u0648\u0641\u0642</div>
        </div>`; });
  };

  const send = DGX.$('#cSend');
  send.onclick = async () => {
    if (!DGX.requireAuth('\u062b\u0628\u062a \u0646\u0638\u0631')) return;
    const inp = DGX.$('#cIn'), text = inp.value.trim();
    if (!text) return;
    send.disabled = true;
    try {
      await DGX.api('/api/app/comment', { body: { product_id: pid, text } });
      inp.value = ''; DGX.toast('\u0646\u0638\u0631 \u062b\u0628\u062a \u0634\u062f +\u06f1 \u06a9\u0631\u062f\u06cc\u062a 💬');
      DGX.pages.product(view, params);
    } catch (e) { DGX.toast(e.msg || '', true); send.disabled = false; }
  };
};

DGX.shareProduct = pid => {
  const t = document.querySelector('h1');
  DGX.shareUrl(location.origin + '/#/product?id=' + pid,
               '🛍 ' + (t ? t.textContent : '') + ' — \u0631\u0648\u06cc DropAgentX');
};
