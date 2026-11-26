/* DropAgentX — ACTIVITY: purchases / sales */
DGX.pages = DGX.pages || {};

DGX.pages.activity = async (view) => {
  view.innerHTML = `
    <div class="seg" style="margin-bottom:14px">
      <button class="on" data-t="bought">${DGX.icon('cart','ic-s')} \u062e\u0631\u06cc\u062f\u0647\u0627\u06cc \u0645\u0646</button>
      <button data-t="sold">${DGX.icon('wallet','ic-s')} \u0641\u0631\u0648\u0634‌\u0647\u0627\u06cc \u0645\u0646</button>
    </div>
    <div id="actOut">${DGX.skelCards(2)}</div>`;
  const out = DGX.$('#actOut');
  let tab = 'bought';

  const d = await DGX.api('/api/app/activity');

  function render() {
    const rows = d[tab];
    if (!rows.length) {
      out.innerHTML = `<div class="empty"><div class="big">${DGX.icon(tab === 'bought' ? 'box' : 'trophy', 'big-ic')}</div>
        ${tab === 'bought' ? '\u0647\u0646\u0648\u0632 \u0686\u06cc\u0632\u06cc \u0646\u062e\u0631\u06cc\u062f\u06cc — \u0641\u06cc\u062f \u062e\u0627\u0646\u0647 \u067e\u0631 \u0627\u0632 \u0627\u06cc\u062f\u0647 \u0627\u0633\u062a!' :
          '\u0647\u0646\u0648\u0632 \u0641\u0631\u0648\u0634\u06cc \u0646\u062f\u0627\u0634\u062a\u06cc — \u0645\u062d\u0635\u0648\u0644\u062a \u0631\u0627 \u0645\u0646\u062a\u0634\u0631 \u06a9\u0646 \u0648 \u0628\u0641\u0631\u0648\u0634!'}</div>`;
      return;
    }
    out.innerHTML = rows.map(r => `
      <div class="rank-row">
        <span class="ic" style="width:42px;height:42px;border-radius:11px;flex:none;
          background:var(--surface2);display:flex;align-items:center;
          justify-content:center">${DGX.icon('box')}</span>
        <div style="flex:1;min-width:0">
          <div class="rank-t">${DGX.esc(r.title)}</div>
          <div class="rank-s">${tab === 'bought'
            ? (r.download_url
               ? `<a href="${DGX.esc(r.download_url)}" target="_blank"
                    style="color:var(--em)">⬇️ \u062f\u0627\u0646\u0644\u0648\u062f \u0641\u0627\u06cc\u0644</a>`
               : '\u0645\u062d\u0635\u0648\u0644 \u0645\u062a\u0646/\u0644\u06cc\u0646\u06a9\u06cc')
            : '👤 ' + DGX.esc(r.buyer || '—')} · ${DGX.timeAgo(r.purchased_at)}</div>
        </div>
        <span class="num" style="font-weight:800;color:${tab === 'bought' ? 'var(--red)' : 'var(--em)'}">
          ${tab === 'bought' ? '-' : '+'}${DGX.fmt(r.price_credits)}</span>
      </div>`).join('');
  }

  view.querySelectorAll('.seg button').forEach(b => b.onclick = () => {
    tab = b.dataset.t;
    view.querySelectorAll('.seg button').forEach(z => z.classList.remove('on'));
    b.classList.add('on');
    render();
  });
  render();
};
