/* DropAgentX — hash router + bottom nav + topbar
   v1.1: role-aware topbar (admin shield only for admins), real notification
   dot (was a hardcoded fake), haptic nav taps, scroll reset per page. */
DGX.pages = {};

DGX.renderNav = () => {
  const items = [
    ['#/home', 'i-home', '\u062e\u0627\u0646\u0647'],
    ['#/explore', 'i-compass', '\u06a9\u0634\u0641'],
    ['CREATE', null, null],                       // special ➕
    ['#/agent', 'i-spark', '\u0647\u0631\u0645\u0633\u0627'],
    ['#/profile', 'i-user', '\u0645\u0646'],
  ];
  const nav = document.getElementById('nav');
  nav.innerHTML = '';
  nav.setAttribute('role', 'tablist');
  for (const [hash, icon, label] of items) {
    if (hash === 'CREATE') {
      const d = document.createElement('div');
      d.className = 'bn-create';
      d.innerHTML = `<button title="\u0633\u0627\u062e\u062a \u0645\u062d\u0635\u0648\u0644" aria-label="\u0633\u0627\u062e\u062a \u0645\u062d\u0635\u0648\u0644" aria-role="tab"><svg class="ic" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round"><path d="M12 5v14M5 12h14"/></svg></button>`;
      d.querySelector('button').onclick = () => { DGX.haptic('light'); location.hash = '#/create'; };
      nav.appendChild(d);
      continue;
    }
    const a = document.createElement('a');
    a.className = 'bn-item'; a.href = hash; a.dataset.hash = hash;
    a.setAttribute('role', 'tab'); a.setAttribute('aria-label', label);
    a.innerHTML = `<svg><use href="#${icon}"/></svg><span>${label}</span>`;
    a.onclick = () => DGX.haptic('light');
    nav.appendChild(a);
  }
};

DGX.renderTopbar = (title) => {
  const tb = document.getElementById('topbar');
  const unread = localStorage.getItem('dgx_unread') === '1';
  tb.innerHTML = `
    <img class="logo" src="/app/assets/logo.jpg" alt="DropAgentX" width="30" height="30">
    <span class="title">${title || ''}</span>
    <span class="spacer"></span>
    ${DGX.isAdmin() ? `<button class="icon-btn shield" onclick="location.href='/admin'" title="\u067e\u0646\u0644 \u0645\u062f\u06cc\u0631\u06cc\u062a (\u0627\u062f\u0645\u06cc\u0646)">
      ${DGX.icon('shield', 'ic-s')}
    </button>` : ''}
    <button class="icon-btn" style="position:relative" data-badge="${unread ? 1 : 0}"
      onclick="localStorage.setItem('dgx_unread','0');this.dataset.badge='0';location.hash='#/activity'" title="\u0641\u0639\u0627\u0644\u06cc\u062a">
      <svg width="18" height="18"><use href="#i-bell"/></svg><span class="dot"></span>
    </button>
    <button class="icon-btn" onclick="location.hash='#/search'" title="\u062c\u0633\u062a\u062c\u0648">
      <svg width="18" height="18"><use href="#i-search"/></svg>
    </button>`;
};

DGX.route = () => {
  const raw = (location.hash || '#/home').slice(2);          // "home?x=1"
  const [name, qs] = raw.split('?');
  const params = Object.fromEntries(new URLSearchParams(qs || ''));
  const page = name || 'home';
  const fn = DGX.pages[page];
  const fab = document.getElementById('hubFab');
  if (fab) fab.style.display = (page === 'product' || page === 'agent') ? 'none' : 'flex';
  document.querySelectorAll('.bn-item').forEach(a =>
    a.classList.toggle('on', a.dataset.hash === '#/' + page));
  const titles = {
    home: '\u062e\u0627\u0646\u0647', explore: '\u06a9\u0634\u0641', search: '\u062c\u0633\u062a\u062c\u0648',
    product: '\u0645\u062d\u0635\u0648\u0644', create: '\u0633\u0627\u062e\u062a \u0645\u062d\u0635\u0648\u0644', profile: '\u067e\u0631\u0648\u0641\u0627\u06cc\u0644',
    wallet: '\u06a9\u06cc\u0641 \u067e\u0648\u0644', activity: '\u0641\u0639\u0627\u0644\u06cc\u062a', agent: '\u0647\u0631\u0645\u0633\u0627',
    reels: '\u0631\u06cc\u0644\u0632', drops: '\u062f\u0631\u0627\u067e‌\u0647\u0627\u06cc \u0632\u0646\u062f\u0647', trending: '\u062a\u0631\u0646\u062f\u0647\u0627',
    fresh: '\u062a\u0627\u0632\u0647‌\u0647\u0627', story: '\u0627\u0633\u062a\u0648\u0631\u06cc',
    categories: '\u062f\u0633\u062a\u0647‌\u0647\u0627', category: '\u062f\u0633\u062a\u0647', collections: '\u06a9\u0627\u0644\u06a9\u0634\u0646‌\u0647\u0627', collection: '\u06a9\u0627\u0644\u06a9\u0634\u0646',
    reviews: '\u062f\u06cc\u062f\u06af\u0627\u0647‌\u0647\u0627', seller: '\u0641\u0631\u0648\u0634\u06af\u0627\u0647', related: '\u0645\u0634\u0627\u0628\u0647‌\u0647\u0627', library: '\u06a9\u062a\u0627\u0628\u062e\u0627\u0646\u0647‌\u06cc \u0645\u0646',
    order: '\u0633\u0641\u0627\u0631\u0634', plans: '\u067e\u0644\u0646 \u0641\u0631\u0648\u0634\u0646\u062f\u0647',
    notifications: '\u0627\u0639\u0644\u0627\u0646‌\u0647\u0627', likes: '\u067e\u0633\u0646\u062f\u0647\u0627', saved: '\u0630\u062e\u06cc\u0631\u0647‌\u0647\u0627', people: '\u062e\u0627\u0644\u0642‌\u0647\u0627',
    follows: '\u0641\u0627\u0644\u0648\u0647\u0627', 'profile-edit': '\u0648\u06cc\u0631\u0627\u06cc\u0634 \u067e\u0631\u0648\u0641\u0627\u06cc\u0644', share: '\u062f\u0639\u0648\u062a \u062f\u0648\u0633\u062a\u0627\u0646',
    'create-drop': '\u0633\u0627\u062e\u062a \u062f\u0631\u0627\u067e', drafts: '\u067e\u06cc\u0634‌\u0646\u0648\u06cc\u0633‌\u0647\u0627', dashboard: '\u062f\u0627\u0634\u0628\u0648\u0631\u062f \u0641\u0631\u0648\u0634\u0646\u062f\u0647',
    analytics: '\u062a\u062d\u0644\u06cc\u0644 \u0641\u0631\u0648\u0634', 'products-manage': '\u0645\u062d\u0635\u0648\u0644\u0627\u062a \u0645\u0646', payouts: '\u062a\u0633\u0648\u06cc\u0647',
    deposit: '\u0648\u0627\u0631\u06cc\u0632', withdraw: '\u0628\u0631\u062f\u0627\u0634\u062a', transactions: '\u062a\u0631\u0627\u06a9\u0646\u0634‌\u0647\u0627', credits: '\u06a9\u0631\u062f\u06cc\u062a',
    settings: '\u062a\u0646\u0638\u06cc\u0645\u0627\u062a', help: '\u0631\u0627\u0647\u0646\u0645\u0627', about: '\u062f\u0631\u0628\u0627\u0631\u0647', terms: '\u0642\u0648\u0627\u0646\u06cc\u0646',
    privacy: '\u062d\u0631\u06cc\u0645 \u062e\u0635\u0648\u0635\u06cc', roadmap: '\u0646\u0642\u0634\u0647‌\u06cc \u0631\u0627\u0647', bot: '\u062f\u0633\u062a\u0648\u0631\u0627\u062a \u0628\u0627\u062a',
    status: '\u0648\u0636\u0639\u06cc\u062a \u0633\u0631\u0648\u06cc\u0633', sitemap: '\u0646\u0642\u0634\u0647‌\u06cc \u06f5\u06f0 \u0635\u0641\u062d\u0647',
  };
  DGX.renderTopbar(titles[page] || '');
  const view = document.getElementById('view');
  view.className = 'page';
  scrollTo(0, 0);                                  // mobile: every page starts at top
  if (!fn) { location.hash = '#/home'; return; }
  fn(view, params).catch(e => {
    if (e && e.silent) return;
    view.innerHTML = `<div class="empty"><div class="big">⚠️</div>${(e && e.msg) || '\u062e\u0637\u0627'}
      <br><br><button class="btn btn-ghost" onclick="history.back()">\u0628\u0631\u06af\u0634\u062a</button></div>`;
  });
};
