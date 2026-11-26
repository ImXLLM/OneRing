/* DropAgentX — WALLET: credits with Web3 skin + game-motion */
DGX.pages = DGX.pages || {};

DGX.pages.wallet = async (view) => {
  if (!DGX.user) await DGX.refreshMe();
  view.innerHTML = `<div class="skel" style="height:150px"></div>${DGX.skelCards(1)}`;

  const d = await DGX.api('/api/app/wallet');
  DGX.perUsdt = d.per_usdt;
  const usd = (d.credits / d.per_usdt).toFixed(2);

  const icon = { deposit: ['deposit', 'pos'], withdraw: ['withdraw', 'neg'],
                 purchase: ['cart', 'neg'], sale: ['wallet', 'pos'],
                 task_completion: ['check', 'pos'], referral_bonus: ['gift', 'pos'],
                 ref_commission: ['users', 'pos'], mystery_box: ['gift', 'pos'],
                 admin_grant: ['shield', 'pos'], admin_deduct: ['shield', 'neg'],
                 welcome: ['flame', 'pos'] };

  const txHtml = d.txs.length ? d.txs.map(t => {
    const [ic, cls] = icon[t.tx_type] || ['•', t.amount >= 0 ? 'pos' : 'neg'];
    return `<div class="tx">
      <span class="ic">${DGX.icon(ic)}</span>
      <div style="flex:1;min-width:0">
        <div style="font-size:12.5px;font-weight:700">${esc(t.description) || t.tx_type}</div>
        <div style="color:var(--dim2);font-size:10px">${DGX.timeAgo(t.created_at)}</div></div>
      <span class="amt ${cls}">${t.amount >= 0 ? '+' : ''}${DGX.fmt(t.amount)}</span>
    </div>`;
  }).join('') : '<div class="empty">\u0647\u0646\u0648\u0632 \u062a\u0631\u0627\u06a9\u0646\u0634\u06cc \u0646\u062f\u0627\u0631\u06cc</div>';

  function esc(s) { return DGX.esc(s); }

  view.innerHTML = `
    <div class="wcard burst">
      <div class="lbl" style="display:flex;align-items:center;gap:7px">${DGX.icon('wallet')} \u0645\u0648\u062c\u0648\u062f\u06cc DropAgentX</div>
      <div class="bal num">${DGX.fmt(d.credits)} <small style="font-size:15px;color:var(--dim)">\u06a9\u0631\u062f\u06cc\u062a</small></div>
      <div class="addr">≈ ${usd} USDT · wallet: ${String(DGX.user?.id || '0x0000').slice(-6)}…${String(DGX.user?.id || '').slice(-4)}</div>
      <div class="w-actions">
        <div class="w-btn" onclick="location.hash='#/activity'"><span class="ic">${DGX.icon('history')}</span>\u062a\u0627\u0631\u06cc\u062e\u0686\u0647</div>
        <div class="w-btn" onclick="DGX.openBot('/start')"><span class="ic">${DGX.icon('deposit')}</span>\u0648\u0627\u0631\u06cc\u0632</div>
        <div class="w-btn" onclick="DGX.openBot('/start')"><span class="ic">${DGX.icon('withdraw')}</span>\u0628\u0631\u062f\u0627\u0634\u062a</div>
        <div class="w-btn" onclick="location.hash='#/create'"><span class="ic">${DGX.icon('rocket')}</span>\u0628\u0641\u0631\u0648\u0634</div>
      </div>
    </div>

    <div class="stat3">
      <div><b class="num">${DGX.fmt(d.earned)}</b><span>\u06a9\u0644 \u062f\u0631\u0622\u0645\u062f</span></div>
      <div><b class="num">${DGX.fmt(d.spent)}</b><span>\u06a9\u0644 \u062e\u0631\u062c</span></div>
      <div><b>${(d.earned / d.per_usdt).toFixed(2)}$</b><span>\u0627\u0631\u0632\u0634 \u06a9\u0644</span></div>
    </div>

    <h3 style="margin:16px 0 6px;display:flex;align-items:center;gap:6px">${DGX.icon('history','ic-s')} \u062a\u0631\u0627\u06a9\u0646\u0634‌\u0647\u0627\u06cc \u0627\u062e\u06cc\u0631</h3>
    ${txHtml}
    <div style="color:var(--dim2);font-size:10.5px;text-align:center;margin-top:14px">
      🔒 \u0645\u0648\u062c\u0648\u062f\u06cc \u0631\u0648\u06cc \u0632\u06cc\u0631\u0633\u0627\u062e\u062a \u0627\u0645\u0646 DropAgentX · \u0646\u0631\u062e \u062b\u0627\u0628\u062a \u06f1\u066c\u06f0\u06f0\u06f0 \u06a9\u0631\u062f\u06cc\u062a = \u06f1 USDT</div>`;

  DGX.ptr(view, async () => { await DGX.refreshMe(); await DGX.pages.wallet(view); });

  // count-up animation on load (game-motion)
  const balEl = view.querySelector('.bal');
  const target = d.credits, dur = 900, t0 = performance.now();
  (function tick(t) {
    const k = Math.min(1, (t - t0) / dur), e = 1 - Math.pow(1 - k, 3);
    balEl.firstChild.textContent = DGX.fmt(Math.round(target * e)) + ' ';
    if (k < 1) requestAnimationFrame(tick);
  })(t0);
};

DGX.openBot = path => {
  const u = window.Telegram?.WebApp?.initDataUnsafe?.user?.username;
  location.href = `https://t.me/DropAgentXBot${path ? '?start=' + encodeURIComponent(path.replace('/', '')) : ''}`;
};
