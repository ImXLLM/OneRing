/* DropAgentX — HOME: algorithmic feed (following-first + trending) */
DGX.pages = DGX.pages || {};

DGX.pages.home = async (view, params) => {
  /* v2.0: feed modes doubled — foryou | following | trend | fresh */
  const tab = ['foryou','following','trend','fresh'].includes(params.tab) ? params.tab : 'foryou';
  const mode = tab === 'following' ? 'following' : (tab === 'trend' ? 'trend' : 'foryou');
  const cat = params.cat || 'all';
  let cursor = parseInt(params.cursor || 0, 10);
  let busy = false, ended = false;

  view.innerHTML = `
    <div id="stories" class="stories"></div>
    ${DGX.hubBar()}
    <div class="seg" style="margin-bottom:10px">
      <button data-tab="foryou" class="${tab === 'foryou' ? 'on' : ''}">${DGX.icon('flame','ic-s')} \u0628\u0631\u0627\u06cc \u062a\u0648</button>
      <button data-tab="following" class="${tab === 'following' ? 'on' : ''}">${DGX.icon('users','ic-s')} \u0641\u0627\u0644\u0648\u06cc\u0646\u06af‌\u0647\u0627</button>
      <button data-tab="trend" class="${tab === 'trend' ? 'on' : ''}">${DGX.icon('chart','ic-s')} \u062a\u0631\u0646\u062f</button>
      <button data-tab="fresh" class="${tab === 'fresh' ? 'on' : ''}">${DGX.icon('deposit','ic-s')} \u062a\u0627\u0632\u0647‌\u0647\u0627</button>
    </div>
    <div class="chips" id="homeCats"></div>
    <div id="feed">${DGX.skelCards(3)}</div>
    <div id="more" style="text-align:center;color:var(--dim2);padding:14px"></div>`;

  // Instagram-style stories (built-in "All" + categories)
  const stBox = DGX.$('#stories');
  if (stBox) stBox.innerHTML = DGX.storiesBox([]);

  // categories chips
  try {
    const cats = (await DGX.api('/api/app/categories')).items;
    if (stBox) stBox.innerHTML = DGX.storiesBox(cats);
    const box = DGX.$('#homeCats');
    if (box) {
      box.innerHTML = `<button class="chip ${cat === 'all' ? 'on' : ''}"
        data-c="all">\u0647\u0645\u0647</button>` +
        cats.filter(c => c.count > 0 || c.key === cat).map(c =>
          `<button class="chip ${cat === c.key ? 'on' : ''}" data-c="${c.key}">
             ${c.icon} ${c.fa}${c.count ? ` · ${DGX.kfmt(c.count)}` : ''}</button>`).join('');
      box.querySelectorAll('.chip').forEach(ch => ch.onclick = () => {
        location.hash = `#/home?tab=${tab}&cat=${ch.dataset.c}`;
      });
    }
  } catch (_) {}

  view.querySelectorAll('.seg button').forEach(b => b.onclick = () => {
    location.hash = `#/home?tab=${b.dataset.tab}&cat=${cat}`;
  });

  const feed = DGX.$('#feed'), more = DGX.$('#more');

  async function load(reset) {
    if (busy || (!reset && ended)) return;
    busy = true;
    more.textContent = reset ? '' : '…';
    if (reset) { cursor = 0; ended = false; feed.innerHTML = DGX.skelCards(3); }
    try {
      const d = await DGX.api(
        `/api/app/feed?mode=${mode}&cat=${encodeURIComponent(cat)}&cursor=${cursor}`);
      if (reset) feed.innerHTML = '';
      ended = d.next === null;
      const items = tab === 'fresh'
        ? [...d.items].sort((a, b) => (b.created_at || 0) - (a.created_at || 0)) : d.items;
      for (const p of items) feed.insertAdjacentHTML('beforeend', DGX.postCard(p));
      wire(feed);
      cursor = d.next ?? cursor;
      if (ended && !d.items.length && reset)
        feed.innerHTML = `<div class="empty"><div class="big">${DGX.icon('flame','big-ic')}</div>
          \u0647\u0646\u0648\u0632 \u067e\u0633\u062a\u06cc \u0646\u06cc\u0633\u062a — \u0627\u0648\u0644\u06cc\u0646 \u0633\u0627\u0632\u0646\u062f\u0647 \u0628\u0627\u0634!<br><br>
          <button class="btn btn-primary" onclick="location.hash='#/create'">${DGX.icon('plus','ic-s')} \u0633\u0627\u062e\u062a \u0645\u062d\u0635\u0648\u0644</button></div>`;
      more.textContent = ended ? (d.items.length ? '\u0647\u0645\u0647 \u0631\u0627 \u062f\u06cc\u062f\u06cc' : '') : '…';
    } catch (e) {
      if (reset) feed.innerHTML =
        `<div class="empty"><div class="big">${DGX.icon('live','big-ic')}</div>${e.msg || '\u062e\u0637\u0627'}
         <br><br><button class="btn btn-ghost" onclick="location.reload()">\u062a\u0644\u0627\u0634 \u0645\u062c\u062f\u062f</button></div>`;
    }
    busy = false;
  }

  const wire = DGX.wireFeed;

  // infinite scroll
  const io = new IntersectionObserver(es => {
    if (es.some(x => x.isIntersecting)) load(false);
  }, { rootMargin: '600px' });
  io.observe(more);

  DGX.ptr(view, async () => { await DGX.refreshMe(); await load(true); });

  await load(true);
};
