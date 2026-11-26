/* DropAgentX v2.0 — WALLET SUITE: deposit / withdraw / transactions / credits */
DGX.pages = DGX.pages || {};

DGX.pages.deposit = async (view) => {
  view.innerHTML = `
    <p class="page-hint">${DGX.icon('deposit','ic-s')} \u0648\u0627\u0631\u06cc\u0632 \u06cc\u0639\u0646\u06cc \u0634\u0627\u0631\u0698 \u06a9\u0631\u062f\u06cc\u062a — \u0647\u0645\u0647‌\u0686\u06cc\u0632 \u0627\u0632 \u0628\u0627\u062a \u0648 \u0627\u0645\u0646 \u0627\u0646\u062c\u0627\u0645 \u0645\u06cc‌\u0634\u0648\u062f</p>
    <div class="post" style="padding:16px">
      <div class="chips" id="amts">${[100, 300, 500, 1000].map(a =>
        `<button class="chip num" data-a="${a}">${DGX.fmt(a)} \u06a9\u0631\u062f\u06cc\u062a</button>`).join('')}
        <button class="chip" data-a="custom">\u0633\u0627\u06cc\u0631…</button></div>
      <div class="field" id="custWrap" style="display:none"><label>\u0645\u0628\u0644\u063a \u062f\u0644\u062e\u0648\u0627\u0647 (\u06a9\u0631\u062f\u06cc\u062a)</label><input id="custAmt" type="number" min="10"></div>
      <div class="insuff" style="background:#0f2b18;border-color:#1d4a2c;color:#8fffc4">
        ${DGX.icon('info','ic-s')} \u0645\u0639\u0627\u062f\u0644 \u062f\u0644\u0627\u0631\u06cc \u0631\u0648\u06cc \u0647\u0645\u0627\u0646 \u0644\u062d\u0638\u0647 \u062f\u0631 \u0628\u0627\u062a \u0646\u0634\u0627\u0646 \u062f\u0627\u062f\u0647 \u0645\u06cc‌\u0634\u0648\u062f.</div>
      <button class="btn btn-primary" id="depGo">${DGX.icon('send','ic-s')} \u0627\u062f\u0627\u0645\u0647 \u062f\u0631 \u0628\u0627\u062a</button>
      <a class="btn btn-ghost" href="#/credits">${DGX.icon('wallet','ic-s')} \u06a9\u0631\u062f\u06cc\u062a \u0686\u06cc\u0633\u062a\u061f</a>
    </div>
    <div class="post" style="padding:16px">
      <h3 style="margin-bottom:8px">${DGX.icon('shield','ic-s')} \u0686\u0631\u0627 \u0627\u0632 \u0628\u0627\u062a\u061f</h3>
      <div class="sumrow"><span>\u067e\u0631\u062f\u0627\u062e\u062a \u062f\u0627\u062e\u0644 \u062a\u0644\u06af\u0631\u0627\u0645</span><b>\u0628\u062f\u0648\u0646 \u062e\u0631\u0648\u062c \u0627\u0632 \u0627\u067e</b></div>
      <div class="sumrow"><span>\u062a\u0623\u06cc\u06cc\u062f \u062f\u0648\u0637\u0631\u0641\u0647</span><b>\u067e\u06cc\u0627\u0645 + \u0631\u0633\u06cc\u062f</b></div>
      <div class="sumrow"><span>\u0634\u0627\u0631\u0698 \u0622\u0646\u06cc</span><b>\u06a9\u0645\u062a\u0631 \u0627\u0632 \u06f1 \u062f\u0642\u06cc\u0642\u0647</b></div>
    </div>`;
  let amt = 100;
  const chips = view.querySelectorAll('#amts .chip');
  chips.forEach(c => c.onclick = () => {
    chips.forEach(x => x.classList.remove('on')); c.classList.add('on');
    DGX.$('#custWrap').style.display = c.dataset.a === 'custom' ? 'block' : 'none';
    amt = c.dataset.a;
  });
  chips[0].classList.add('on');
  DGX.$('#depGo').onclick = () => {
    const v = amt === 'custom' ? (+DGX.$('#custAmt').value || 0) : amt;
    DGX.openBot(v ? `/deposit ${v}` : '/deposit');
  };
};

DGX.pages.withdraw = async (view) => {
  let w = { credits: 0 };
  try { w = await DGX.api('/api/app/wallet'); } catch (_) {}
  view.innerHTML = `
    <p class="page-hint">${DGX.icon('withdraw','ic-s')} \u0628\u0631\u062f\u0627\u0634\u062a \u06a9\u0631\u062f\u06cc\u062a \u0627\u0632 \u0641\u0631\u0648\u0634‌\u0647\u0627 — \u0645\u0633\u062a\u0642\u06cc\u0645 \u0628\u0647 \u06a9\u06cc\u0641 \u062a\u0644\u06af\u0631\u0627\u0645\u06cc \u062a\u0648</p>
    <div class="post" style="padding:16px">
      <div class="sumrow"><span>\u0645\u0648\u062c\u0648\u062f\u06cc \u0641\u0639\u0644\u06cc</span><b class="num">${DGX.fmt(w.credits)} \u06a9\u0631\u062f\u06cc\u062a</b></div>
      <div class="sumrow total"><span>\u0642\u0627\u0628\u0644 \u0628\u0631\u062f\u0627\u0634\u062a</span><b class="num" style="color:var(--em)">${DGX.fmt(Math.max(0, w.credits))} </b></div>
      <button class="btn btn-primary" onclick="DGX.openBot('/withdraw')">${DGX.icon('send','ic-s')} \u062f\u0631\u062e\u0648\u0627\u0633\u062a \u0628\u0631\u062f\u0627\u0634\u062a</button>
      <a class="btn btn-ghost" href="#/payouts">${DGX.icon('shield','ic-s')} \u0642\u0648\u0627\u0646\u06cc\u0646 \u062a\u0633\u0648\u06cc\u0647</a>
    </div>`;
};

DGX.pages.transactions = async (view) => {
  if (!DGX.requireAuth('\u062a\u0631\u0627\u06a9\u0646\u0634‌\u0647\u0627')) { location.hash = '#/wallet'; return; }
  view.innerHTML = DGX.skelCards(2);
  let w;
  try { w = await DGX.api('/api/app/wallet'); }
  catch (e) { view.innerHTML = `<div class="empty">${e.msg || '\u062e\u0637\u0627'}</div>`; return; }
  const F = { all: null, sale: ['wallet', '\u0641\u0631\u0648\u0634'], purchase: ['cart', '\u062e\u0631\u06cc\u062f'],
              deposit: ['deposit', '\u0648\u0627\u0631\u06cc\u0632'], withdraw: ['withdraw', '\u0628\u0631\u062f\u0627\u0634\u062a'],
              task_completion: ['check', '\u062a\u0633\u06a9'], referral_bonus: ['gift', '\u0631\u06cc\u0641\u0631\u0627\u0644'] };
  const f = F[location.hash.split('f=')[1]?.slice(0, 12)] || 'all';
  const rows = (w.txs || []).filter(t => !F[f] || t.tx_type === f);
  const name = t => ({ sale: '\u0641\u0631\u0648\u0634', purchase: '\u062e\u0631\u06cc\u062f', deposit: '\u0648\u0627\u0631\u06cc\u0632', withdraw: '\u0628\u0631\u062f\u0627\u0634\u062a',
    task_completion: '\u062a\u0633\u06a9', referral_bonus: '\u067e\u0627\u062f\u0627\u0632 \u0631\u06cc\u0641\u0631\u0627\u0644', admin_grant: '\u0627\u062f\u0645\u06cc\u0646',
    admin_deduct: '\u06a9\u0633\u0631 \u0627\u062f\u0645\u06cc\u0646', welcome: '\u062e\u0648\u0634‌\u0622\u0645\u062f' })[t.tx_type] || t.tx_type;
  const icon = t => (F[t.tx_type] ? F[t.tx_type][0] : 'wallet');
  view.innerHTML = `
    <div class="chips">${Object.keys(F).map(k =>
      `<button class="chip ${f === k ? 'on' : ''}" onclick="location.hash='#/transactions?f=${k}'">
        ${k === 'all' ? '\u0647\u0645\u0647' : name({ tx_type: k })}</button>`).join('')}</div>
    ${rows.length ? rows.map(t => `
      <div class="notif-row">
        <span class="nr-ic">${DGX.icon(icon(t))}</span>
        <div style="flex:1;min-width:0"><b>${name(t)}</b>
          <span class="p-sub">${DGX.esc(t.description || '')} · ${DGX.timeAgo(t.created_at || 0)}</span></div>
        <b class="num" style="color:${String(t.amount || '')[0] === '-' ? 'var(--dim)' : 'var(--em)'}">${DGX.fmt(t.amount)}</b>
      </div>`).join('') : `<div class="empty"><div class="big">${DGX.icon('history','big-ic')}</div>\u062a\u0631\u0627\u06a9\u0646\u0634\u06cc \u062f\u0631 \u0627\u06cc\u0646 \u0641\u06cc\u0644\u062a\u0631 \u0646\u06cc\u0633\u062a<br><br>
        <a class="btn btn-ghost" href="#/transactions?f=all">\u0647\u0645\u0647‌\u06cc \u062a\u0631\u0627\u06a9\u0646\u0634‌\u0647\u0627</a></div>`}
    <div style="text-align:center;padding:12px"><a class="lnk" href="#/wallet">← \u06a9\u06cc\u0641 \u067e\u0648\u0644</a></div>`;
};

DGX.pages.credits = async (view) => {
  let info = { credits_per_usdt: 100 };
  try { info = await DGX.api('/api/pub/info'); } catch (_) {}
  const per = info.credits_per_usdt || 100;
  view.innerHTML = `
    <div class="post" style="padding:16px;text-align:center">
      <span class="nr-ic" style="width:52px;height:52px;margin:0 auto 8px">${DGX.icon('wallet')}</span>
      <b style="font-size:16px">\u06a9\u0631\u062f\u06cc\u062a\u060c \u067e\u0648\u0644\u0650 \u062f\u0627\u062e\u0644 DropAgentX \u0627\u0633\u062a</b>
      <p class="p-sub" style="margin-top:6px">\u0647\u0631 <b class="num">${DGX.fmt(per)}</b> \u06a9\u0631\u062f\u06cc\u062a ≈ \u06f1 \u062f\u0644\u0627\u0631 — \u0628\u0631\u0627\u06cc \u062e\u0631\u06cc\u062f\u060c \u0641\u0631\u0648\u0634 \u0648 \u0647\u062f\u06cc\u0647</p>
    </div>
    <div class="post" style="padding:16px">
      <h3 style="margin-bottom:8px">${DGX.icon('chart','ic-s')} \u062a\u0628\u062f\u06cc\u0644 \u0633\u0631\u06cc\u0639</h3>
      ${[50, 100, 500, 1000].map(c => `
        <div class="bar-row"><span class="num">${DGX.fmt(c)} \u06a9\u0631\u062f\u06cc\u062a</span><i></i><b class="num">≈${(c / per).toFixed(2)}$</b></div>`).join('')}
    </div>
    <div class="col-grid">
      <a class="col-card" href="#/deposit"><span class="cc-ic">${DGX.icon('deposit')}</span><b>\u0634\u0627\u0631\u0698 \u06a9\u0631\u062f\u06cc\u062a</b></a>
      <a class="col-card" href="#/share"><span class="cc-ic">${DGX.icon('gift')}</span><b>\u06a9\u0631\u062f\u06cc\u062a \u0631\u0627\u06cc\u06af\u0627\u0646 \u0628\u0627 \u062f\u0639\u0648\u062a</b></a>
    </div>`;
};
