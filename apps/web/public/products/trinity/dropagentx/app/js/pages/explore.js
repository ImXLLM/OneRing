/* DropAgentX — EXPLORE: trending + categories */
DGX.pages = DGX.pages || {};

DGX.pages.explore = async (view, params) => {
  view.innerHTML = `
    <div class="chips" id="exRange">
      <button class="chip on" data-r="trend">${DGX.icon('flame','ic-s')} \u062a\u0631\u0646\u062f</button>
      <button class="chip" data-r="new">🆕 \u062c\u062f\u06cc\u062f</button>
      <button class="chip" data-r="featured">⭐ \u0648\u06cc\u0698\u0647</button>
      <button class="chip" data-r="budget">${DGX.icon('wallet','ic-s')} \u0627\u0631\u0632\u0627\u0646‌\u0647\u0627 (\u0632\u06cc\u0631 \u06f1\u06f0\u06f0)</button>
    </div>
    <div id="rankBox">${DGX.skelCards(3)}</div>
    <h2 style="margin:18px 0 10px">🗂 \u062f\u0633\u062a\u0647‌\u0628\u0646\u062f\u06cc‌\u0647\u0627</h2>
    <div class="cat-grid" id="catGrid"></div>
    <div id="gridOut"></div>`;

  let mode = params.r || 'trend';
  const rankBox = DGX.$('#rankBox'), grid = DGX.$('#catGrid');

  async function loadRank() {
    rankBox.innerHTML = DGX.skelCards(3);
    try {
      let items;
      if (mode === 'featured') {
        items = (await DGX.api('/api/app/trending?limit=20')).items.filter(x => x.is_featured);
      } else if (mode === 'new') {
        items = (await DGX.api('/api/app/feed?mode=foryou&cursor=0')).items
          .sort((a, b) => b.id - a.id);
      } else if (mode === 'budget') {
        items = (await DGX.api('/api/app/feed?mode=foryou&cursor=0')).items
          .filter(x => x.price_credits < 100)
          .sort((a, b) => a.price_credits - b.price_credits);
      } else {
        items = (await DGX.api('/api/app/trending?limit=10')).items;
      }
      if (!items.length) { rankBox.innerHTML =
        '<div class="empty"><div class="big">' + DGX.icon('box','big-ic') + '</div>\u0641\u0639\u0644\u0627\u064b \u0686\u06cc\u0632\u06cc \u0627\u06cc\u0646\u062c\u0627 \u0646\u06cc\u0633\u062a</div>'; return; }
      rankBox.innerHTML = items.map(p => `
        <a class="rank-row ${p.rank === 1 ? 'rank-1' : ''}" href="#/product?id=${+p.id}"
           style="text-decoration:none;color:inherit">
          <span class="rank-num">#${p.rank || '•'}</span>
          ${p.photo_url ? `<img class="rank-img" src="${DGX.esc(p.photo_url)}" loading="lazy">`
                        : `<div class="rank-img" style="display:flex;align-items:center;justify-content:center">${DGX.icon('box','ic-l')}</div>`}
          <div class="rank-info">
            <div class="rank-t">${DGX.esc(p.title)}</div>
            <div class="rank-s">@${DGX.esc(p.creator_username || p.creator_name || '')} ·
              ${DGX.icon('cart','ic-s')} ${DGX.kfmt(p.sales_count)} \u0641\u0631\u0648\u0634</div>
          </div>
          <span class="num" style="color:var(--em);font-weight:800;font-size:12px;white-space:nowrap">
            ${DGX.fmt(p.price_credits)}</span>
        </a>`).join('');
    } catch (e) { rankBox.innerHTML = `<div class="empty">⚠️ ${e.msg || ''}</div>`; }
  }

  try {
    const cats = (await DGX.api('/api/app/categories')).items;
    grid.innerHTML = cats.map(c => `
      <div class="cat-card" data-c="${c.key}">
        <div class="ic">${DGX.catIcon(c.key)}</div><div class="nm">${c.fa}</div>
        <div class="ct num">${DGX.kfmt(c.count)}</div></div>`).join('');
    grid.querySelectorAll('.cat-card').forEach(el => el.onclick = () => {
      location.hash = `#/explore?r=${mode}&cat=${el.dataset.c}`;
    });
  } catch (_) {}

  view.querySelectorAll('#exRange .chip').forEach(ch => ch.onclick = () => {
    mode = ch.dataset.r;
    view.querySelectorAll('#exRange .chip').forEach(z => z.classList.remove('on'));
    ch.classList.add('on');
    loadRank();
  });

  await loadRank();
};
