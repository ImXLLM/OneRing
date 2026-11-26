/* DropAgentX v2.0 — CATALOG SUITE: categories / category / collections / collection */
DGX.pages = DGX.pages || {};

DGX.pages.categories = async (view) => {
  view.innerHTML = DGX.skelCards(2);
  let c;
  try { c = (await DGX.api('/api/app/categories')).items; }
  catch (e) { view.innerHTML = `<div class="empty">${e.msg || '\u062e\u0637\u0627'}</div>`; return; }
  view.innerHTML = `
    <p class="page-hint">${DGX.icon('puzzle','ic-s')} ${c.filter(x => x.count > 0).length} \u062f\u0633\u062a\u0647‌\u06cc \u0641\u0639\u0627\u0644 — \u0628\u0627\u0632\u0627\u0631 \u0631\u0627 \u0627\u0632 \u0627\u06cc\u0646\u062c\u0627 \u0628\u06af\u0631\u062f</p>
    <div class="cat-grid-big">
      ${c.map(k => `
        <a class="cat-big ${k.count ? '' : 'mute'}" href="#/category?key=${k.key}">
          <span class="cb-ic">${DGX.catIcon(k.key)}</span>
          <b>${DGX.esc(k.fa)}</b>
          <span>${DGX.kfmt(k.count || 0)} \u0645\u062d\u0635\u0648\u0644</span>
        </a>`).join('')}
    </div>
    <div style="text-align:center;padding:14px"><a class="lnk" href="#/sitemap">\u0646\u0642\u0634\u0647‌\u06cc \u06a9\u0627\u0645\u0644 \u06f5\u06f0 \u0635\u0641\u062d\u0647 ←</a></div>`;
};

DGX.pages.category = async (view, params) => {
  const key = params.key || 'all';
  const SORTS = { hot: ['\u067e\u0631\u0641\u0631\u0648\u0634', (a, b) => (b.sales_count || 0) - (a.sales_count || 0)],
                  cheap: ['\u0627\u0631\u0632\u0627\u0646‌\u062a\u0631', (a, b) => (a.price_credits || 0) - (b.price_credits || 0)],
                  new: ['\u062c\u062f\u06cc\u062f\u062a\u0631', (a, b) => (b.created_at || 0) - (a.created_at || 0)] };
  const s = SORTS[params.sort] ? params.sort : 'hot';
  let title = '\u062f\u0633\u062a\u0647', cats = [];
  try { cats = (await DGX.api('/api/app/categories')).items;
    title = (cats.find(c => c.key === key) || {}).fa || '\u062f\u0633\u062a\u0647'; } catch (_) {}
  view.innerHTML = `
    <div class="seg" style="margin-bottom:10px">
      ${Object.entries(SORTS).map(([k, v]) =>
        `<button class="${s === k ? 'on' : ''}" onclick="location.hash='#/category?key=${key}&sort=${k}'">${v[0]}</button>`).join('')}
    </div>
    <div class="mosaic" id="mg">${DGX.icon('box','big-ic ph-g')} \u062f\u0631 \u062d\u0627\u0644 \u0628\u0627\u0631\u06af\u0630\u0627\u0631\u06cc…</div>
    <div id="more" style="text-align:center;color:var(--dim2);padding:12px"></div>`;
  let cursor = 0, ended = false, all = [];
  const grid = DGX.$('#mg');
  async function load() {
    if (ended) return;
    try {
      const d = await DGX.api(`/api/app/feed?mode=foryou&cat=${encodeURIComponent(key)}&cursor=${cursor}`);
      ended = d.next === null; cursor = d.next ?? cursor; all = all.concat(d.items);
      const rows = [...all].sort(SORTS[s][1]);
      grid.innerHTML = rows.map(DGX.tile).join('') ||
        `<div class="empty" style="grid-column:1/-1"><div class="big">${DGX.icon('box','big-ic')}</div>
         \u0627\u06cc\u0646\u062c\u0627 \u0647\u0646\u0648\u0632 \u062e\u0627\u0644\u06cc\u0647<br><br><a class="btn btn-primary" href="#/create">${DGX.icon('plus','ic-s')} \u062a\u0648 \u067e\u0631\u0634 \u06a9\u0646</a></div>`;
      DGX.$('#more').textContent = ended ? '— \u067e\u0627\u06cc\u0627\u0646 \u062f\u0633\u062a\u0647 —' : '…';
    } catch (e) { DGX.$('#more').textContent = e.msg || '\u062e\u0637\u0627'; ended = true; }
  }
  await load();
  window.onscroll = () => { if (innerHeight + scrollY > document.body.scrollHeight - 500 && !ended) load(); };
};

const COLS = {
  cheap: { t: '\u062c\u0648\u0648\u0646‌\u0628\u0627\u0632\u0627\u0631', d: '\u0647\u0645\u0647‌\u0686\u06cc\u0632 \u0632\u06cc\u0631 \u06f1\u06f0\u06f0 \u06a9\u0631\u062f\u06cc\u062a', icon: 'wallet',
           q: () => '/api/app/feed?mode=foryou&cursor=0', f: rows => rows.filter(p => p.price_credits <= 100) },
  best:  { t: '\u067e\u0631\u0641\u0631\u0648\u0634‌\u0647\u0627\u06cc \u062a\u0627\u0631\u06cc\u062e', d: '\u0645\u062d\u0635\u0648\u0644\u0627\u062a\u06cc \u06a9\u0647 \u0628\u06cc\u0634\u062a\u0631\u06cc\u0646 \u0641\u0631\u0648\u0634 \u0631\u0627 \u062f\u0627\u0634\u062a\u0647‌\u0627\u0646\u062f', icon: 'trophy',
           q: () => '/api/app/trending?limit=20', f: rows => rows },
  fresh: { t: '\u062a\u0627\u0632\u0647 \u0631\u0633\u06cc\u062f\u0647‌\u0647\u0627', d: '\u0622\u062e\u0631\u06cc\u0646 \u0633\u0627\u062e\u062a\u0647‌\u0634\u062f\u0647‌\u0647\u0627 \u062a\u0648\u0633\u0637 \u062e\u0627\u0644\u0642‌\u0647\u0627', icon: 'deposit',
           q: () => '/api/app/feed?mode=foryou&cursor=0', f: rows => [...rows].sort((a, b) => (b.created_at || 0) - (a.created_at || 0)) },
  loved: { t: '\u0645\u062d\u0628\u0648\u0628‌\u0647\u0627', d: '\u0628\u06cc\u0634\u062a\u0631\u06cc\u0646 \u067e\u0633\u0646\u062f\u0650 community', icon: 'heart',
           q: () => '/api/app/trending?limit=20', f: rows => [...rows].sort((a, b) => (b.like_count || 0) - (a.like_count || 0)) },
};

DGX.pages.collections = async (view) => {
  view.innerHTML = `
    <p class="page-hint">${DGX.icon('star-f','ic-s')} \u06a9\u0627\u0644\u06a9\u0634\u0646‌\u0647\u0627\u06cc \u0632\u0646\u062f\u0647 — \u0627\u0632 \u062f\u0627\u062f\u0647‌\u06cc \u0648\u0627\u0642\u0639\u06cc \u0628\u0627\u0632\u0627\u0631\u060c \u0647\u0631 \u0631\u0648\u0632 \u0628\u0647‌\u0631\u0648\u0632</p>
    <div class="col-grid">
      ${Object.entries(COLS).map(([k, c]) => `
        <a class="col-card" href="#/collection?kind=${k}">
          <span class="cc-ic">${DGX.icon(c.icon)}</span>
          <b>${c.t}</b><span>${c.d}</span>
          <i class="cc-go">${DGX.icon('chevron-l','ic-s')}</i>
        </a>`).join('')}
    </div>
    <div class="col-grid" style="margin-top:10px">
      <a class="col-card" href="#/categories"><span class="cc-ic">${DGX.icon('puzzle')}</span>
        <b>\u0647\u0645\u0647‌\u06cc \u062f\u0633\u062a\u0647‌\u0647\u0627</b><span>\u06f1\u06f3 \u062f\u0633\u062a\u0647‌\u06cc \u0641\u0639\u0627\u0644 \u0628\u0627\u0632\u0627\u0631</span></a>
      <a class="col-card" href="#/trending"><span class="cc-ic">${DGX.icon('chart')}</span>
        <b>\u062a\u0631\u0646\u062f\u0647\u0627</b><span>\u06f1\u06f0 \u0645\u062d\u0635\u0648\u0644 \u0628\u0631\u062a\u0631 \u0627\u0645\u0631\u0648\u0632</span></a>
    </div>`;
};

DGX.pages.collection = async (view, params) => {
  const c = COLS[params.kind];
  if (!c) { location.hash = '#/collections'; return; }
  view.innerHTML = `<div class="col-head">${DGX.icon(c.icon)} <b>${c.t}</b><span>${c.d}</span></div>
    <div class="mosaic" id="mg">${DGX.icon('box','big-ic ph-g')} …</div>`;
  try {
    const d = await DGX.api(c.q());
    const rows = c.f(d.items || []);
    DGX.$('#mg').innerHTML = rows.map(DGX.tile).join('') ||
      `<div class="empty" style="grid-column:1/-1"><div class="big">${DGX.icon('box','big-ic')}</div>\u0641\u0639\u0644\u0627\u064b \u0645\u0648\u0631\u062f\u06cc \u0646\u06cc\u0633\u062a</div>`;
  } catch (e) { DGX.$('#mg').innerHTML = `<div class="empty">${e.msg || '\u062e\u0637\u0627'}</div>`; }
};
