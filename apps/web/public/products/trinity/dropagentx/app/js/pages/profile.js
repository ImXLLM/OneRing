/* DropAgentX — PROFILE: personal store (avatar 1:1 + cover 16:9) */
DGX.pages = DGX.pages || {};

DGX.pages.profile = async (view, params) => {
  if (!DGX.user) await DGX.refreshMe();
  const uid = +(params.uid || (DGX.user && DGX.user.id) || 0);
  if (!uid) { DGX.needTelegram(); return; }

  const d = await DGX.api(`/api/app/store/${uid}`);
  const per = DGX.perUsdt || 1000;
  const isMe = !!d.is_me;

  view.innerHTML = `
    <div class="cover-wrap" id="coverWrap">
      ${d.cover_url ? `<img src="${DGX.esc(d.cover_url)}">` :
        `<div style="height:100%;display:flex;align-items:center;justify-content:center;
           color:#1e2a24;font-size:15px;font-weight:800;letter-spacing:2px">
           DROPAGENTX · ${DGX.kfmt(d.total_sales)} SALES</div>`}
    </div>
    <div style="display:flex;align-items:flex-end;gap:12px;padding:0 var(--pad)">
      <img class="avatar-big" id="avImg"
           src="${d.avatar_url ? DGX.esc(d.avatar_url) : '/app/assets/logo.jpg'}">
      <div style="flex:1;padding-bottom:6px">
        <b style="font-size:17px">${DGX.esc(d.name || '\u0634\u0647\u0631\u0648\u0646\u062f')} ✔</b>
          ${DGX.isAdmin() && isMe ? `<span class="badge-role admin">${DGX.icon('shield','ic-xs')} \u0627\u062f\u0645\u06cc\u0646</span>` :
            (isMe && DGX.user?.role === 'hunter' ? `<span class="badge-role hunter">${DGX.icon('target','ic-xs')} \u0647\u0627\u0646\u062a\u0631</span>` : '')}
        <div style="color:var(--dim);font-size:11.5px">@${DGX.esc(d.username || 'dropagentx')}
          ${d.following_me ? '· \u0634\u0645\u0627 \u0631\u0627 \u0641\u0627\u0644\u0648 \u0645\u06cc‌\u06a9\u0646\u062f' : ''}</div>
        ${isMe && !DGX.isAdmin() && DGX.user?.role !== 'hunter' ?
          `<div style="color:var(--dim2);font-size:10.5px;margin-top:2px">${DGX.icon('trophy','ic-xs')} ${DGX.roleFa(DGX.user?.role)}</div>` : ''}
      </div>
      ${!isMe ? `<button class="follow-btn ${d.following_me ? 'on' : ''}" id="folBtn">
        ${d.following_me ? '✓ \u0641\u0627\u0644\u0648 \u0634\u062f' : '+ \u0641\u0627\u0644\u0648'}</button>` : ''}
      ${isMe ? `<button class="pill-btn" onclick="DGX.upPhoto('avatar')">${DGX.icon('image','ic-s')} \u0622\u0648\u0627\u062a\u0627\u0631</button>
                <button class="pill-btn" onclick="DGX.upPhoto('cover')">${DGX.icon('palette','ic-s')} \u06a9\u0627\u0648\u0631</button>` : ''}
    </div>
    <input type="file" id="phFile" accept="image/jpeg,image/png" hidden>

    <div style="padding:0 var(--pad)">
      <div class="stat3">
        <div><b>${DGX.kfmt(d.followers)}</b><span>\u0641\u0627\u0644\u0648\u0648\u0631</span></div>
        <div><b>${d.products.length}</b><span>\u0645\u062d\u0635\u0648\u0644</span></div>
        <div><b>${DGX.kfmt(d.total_sales)}</b><span>\u0641\u0631\u0648\u0634</span></div>
        <div><b class="num">${(d.products.reduce((s, p) => s + (+p.price_credits || 0), 0) / 1000).toFixed(2)}$</b><span>\u0627\u0631\u0632\u0634 \u0645\u063a\u0627\u0632\u0647</span></div>
      </div>
      <div style="display:flex;gap:8px;margin-bottom:12px;flex-wrap:wrap">
        <a class="btn btn-ghost" href="#/wallet">${DGX.icon('wallet','ic-s')} \u06a9\u06cc\u0641 \u067e\u0648\u0644</a>
        <a class="btn btn-ghost" href="#/activity">${DGX.icon('history','ic-s')} \u0641\u0639\u0627\u0644\u06cc\u062a</a>
        ${DGX.isAdmin() ? `<a class="btn btn-gold" href="/admin" target="_blank">${DGX.icon('shield','ic-s')} \u067e\u0646\u0644 \u0645\u062f\u06cc\u0631\u06cc\u062a</a>
        <a class="btn btn-ghost" href="/insights" target="_blank">${DGX.icon('chart','ic-s')} \u062a\u062d\u0644\u06cc\u0644</a>` : ''}
        ${DGX.isHunter() ? `<a class="btn btn-ghost" href="https://t.me/DropAgentXBot?start=hunter">${DGX.icon('target','ic-s')} \u0627\u0628\u0632\u0627\u0631 \u0647\u0627\u0646\u062a\u0631</a>` : ''}
      </div>
      <div style="margin-bottom:4px">${DGX.hubBar()}</div>
      <h3 style="margin-bottom:10px;display:flex;align-items:center;gap:6px">${DGX.icon('shop','ic-s')} \u0648\u06cc\u062a\u0631\u06cc\u0646 ${isMe ? '\u0645\u0646' : '\u0627\u06cc\u0646 \u0633\u0627\u0632\u0646\u062f\u0647'}</h3>
      <div class="grid3" id="storeGrid"></div>
      <div style="height:80px"></div>
    </div>`;

  const grid = DGX.$('#storeGrid');
  grid.innerHTML = d.products.length ? d.products.map(p => `
    <a class="g-item" href="#/product?id=${+p.id}">
      ${p.photo_url ? `<img src="${DGX.esc(p.photo_url)}" loading="lazy">` : DGX.icon('box','ic-l ph-g')}
      <span class="g-price">${DGX.fmt(p.price_credits)}</span></a>`).join('')
    : `<div style="grid-column:1/-1;text-align:center;color:var(--dim);padding:26px;line-height:2">
         ${isMe ? '\u0647\u0646\u0648\u0632 \u0645\u062d\u0635\u0648\u0644\u06cc \u0646\u0633\u0627\u062e\u062a\u06cc<br>' : ''}
         <button class="btn btn-primary" onclick="location.hash='#/create'"
           style="${isMe ? '' : 'display:none'};max-width:220px;margin-top:8px">${DGX.icon('plus','ic-s')} \u0633\u0627\u062e\u062a \u0627\u0648\u0644\u06cc\u0646 \u0645\u062d\u0635\u0648\u0644</button>
       </div>`;

  if (!isMe) {
    DGX.$('#folBtn').onclick = async () => {
      const on = !d.following_me;
      try {
        await DGX.api('/api/app/follow', { body: { target: uid, on } });
        DGX.toast(on ? `\u0641\u0627\u0644\u0648 \u06a9\u0631\u062f\u06cc ✅` : '\u0622\u0646\u0641\u0627\u0644\u0648 \u0634\u062f');
        DGX.pages.profile(view, params);
      } catch (e) { DGX.toast(e.msg || '', true); }
    };
  }
};

DGX.upPhoto = kind => {
  const inp = document.createElement('input');
  inp.type = 'file'; inp.accept = 'image/jpeg,image/png';
  inp.onchange = async () => {
    const f = inp.files[0]; if (!f) return;
    // client-side aspect hint: avatar→square, cover→16:9 (server stores raw)
    const fd = new FormData(); fd.append('file', f);
    DGX.haptic('medium');
    try {
      const r = await fetch(`/api/app/me/photo/${kind}`, {
        method: 'POST', headers: { 'X-Requested-With': 'fetch',
                                   Authorization: 'Bearer ' + DGX.token },
        body: fd });
      const d = await r.json().catch(() => ({}));
      if (!r.ok) throw d.detail || '\u0622\u067e\u0644\u0648\u062f \u0646\u0627\u0645\u0648\u0641\u0642';
      DGX.toast(kind === 'avatar' ? '\u0622\u0648\u0627\u062a\u0627\u0631 \u06f1:\u06f1 \u0633\u062a \u0634\u062f ✨' : '\u06a9\u0627\u0648\u0631 \u06f1\u06f6:\u06f9 \u0633\u062a \u0634\u062f ✨');
      location.reload();
    } catch (e) { DGX.toast(typeof e === 'string' ? e : '\u0622\u067e\u0644\u0648\u062f \u0646\u0627\u0645\u0648\u0641\u0642', true); }
  };
  inp.click();
};
