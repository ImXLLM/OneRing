/* DropAgentX — CREATE: 4-step wizard (info → price → images → publish) */
DGX.pages = DGX.pages || {};

DGX.pages.create = async (view) => {
  if (!DGX.requireAuth('\u0633\u0627\u062e\u062a \u0645\u062d\u0635\u0648\u0644')) { location.hash = '#/home'; return; }
  let step = 1;
  const data = { title: '', description: '', category: 'other', price_credits: 0 };
  const images = { main: null, feed: null, story: null };   // File objects
  const previews = { main: null, feed: null, story: null }; // object URLs
  let cats = [];
  try { cats = (await DGX.api('/api/app/categories')).items; } catch (_) {}

  view.innerHTML = `<div id="wiz"></div>`;

  const IMG_SPECS = {
    main:  { ratio: '\u06f1:\u06f1',  w: 1080, h: 1080, hint: '\u0639\u06a9\u0633 \u0627\u0635\u0644\u06cc \u0645\u062d\u0635\u0648\u0644 \u062f\u0631 \u0635\u0641\u062d\u0647\u0654 \u062c\u0632\u0626\u06cc\u0627\u062a', icon: 'image' },
    feed:  { ratio: '\u06f1\u06f6:\u06f9', w: 1920, h: 1080, hint: '\u0646\u0645\u0627\u06cc\u0634 \u062f\u0631 \u0641\u06cc\u062f \u062e\u0627\u0646\u0647 — \u0645\u062b\u0644 \u062a\u0627\u0645\u0628\u0646\u06cc\u0644 \u06cc\u0648\u062a\u06cc\u0648\u0628\u060c \u062c\u0630\u0627\u0628‌\u062a\u0631 \u0628\u0647\u062a\u0631', icon: 'film' },
    story: { ratio: '\u06f9:\u06f1\u06f6', w: 1080, h: 1920, hint: '\u0646\u0645\u0627\u06cc\u0634 \u062f\u0631 \u0627\u06a9\u0633\u067e\u0644\u0648\u0631 — \u0645\u0639\u0631\u0641\u06cc \u06a9\u0648\u062a\u0627\u0647 \u0645\u062d\u0635\u0648\u0644', icon: 'camera' },
  };

  function render() {
    const w = DGX.$('#wiz');
    if (step === 1) {
      w.innerHTML = `
        <div class="step-dots"><i class="on"></i><i></i><i></i><i></i></div>
        <h2 style="margin-bottom:4px;display:flex;align-items:center;gap:7px">${DGX.icon('box')} \u0645\u062d\u0635\u0648\u0644\u062a \u0631\u0627 \u0645\u0639\u0631\u0641\u06cc \u06a9\u0646</h2>
        <p style="color:var(--dim);font-size:12.5px;margin-bottom:6px">\u0627\u0648\u0644 \u0686\u06cc \u0645\u06cc‌\u0641\u0631\u0648\u0634\u06cc\u061f</p>
        <div class="field"><label>\u0639\u0646\u0648\u0627\u0646</label>
          <input id="wTitle" maxlength="120" value="${DGX.esc(data.title)}"
                 aria-label="\u0645\u062b\u0644\u0627\u064b: \u067e\u06a9\u06cc\u062c \u06f5\u06f0 \u067e\u0631\u0627\u0645\u067e\u062a \u0633\u06cc\u0646\u0645\u0627\u06cc\u06cc AI"></div>
        <div class="field"><label>\u062a\u0648\u0636\u06cc\u062d\u0627\u062a</label>
          <textarea id="wDesc" rows="5" maxlength="1000"
            aria-label="\u0648\u06cc\u0698\u06af\u06cc‌\u0647\u0627\u060c \u062f\u0633\u062a\u0627\u0648\u0631\u062f\u0647\u0627\u060c \u0628\u0631\u0627\u06cc \u06a9\u06cc \u0645\u0646\u0627\u0633\u0628 \u0627\u0633\u062a…">${DGX.esc(data.description)}</textarea></div>
        <button class="btn btn-primary" id="wNext">\u0628\u0639\u062f\u06cc ←</button>`;
      w.querySelector('#wNext').onclick = () => {
        data.title = w.querySelector('#wTitle').value.trim();
        data.description = w.querySelector('#wDesc').value.trim();
        if (data.title.length < 3) return DGX.toast('\u0639\u0646\u0648\u0627\u0646 \u062d\u062f\u0627\u0642\u0644 \u06f3 \u062d\u0631\u0641', true);
        step = 2; render(); DGX.haptic('light');
      };
    } else if (step === 2) {
      w.innerHTML = `
        <div class="step-dots"><i class="on"></i><i class="on"></i><i></i><i></i></div>
        <h2 style="margin-bottom:10px;display:flex;align-items:center;gap:7px">${DGX.icon('puzzle')} \u062f\u0633\u062a\u0647 \u0648 \u0642\u06cc\u0645\u062a</h2>
        <div class="field"><label>\u062f\u0633\u062a\u0647‌\u0628\u0646\u062f\u06cc</label>
          <select id="wCat">${cats.map(c =>
            `<option value="${c.key}" ${data.category === c.key ? 'selected' : ''}>
               ${c.icon} ${c.fa}</option>`).join('')}</select></div>
        <div class="field"><label>\u0642\u06cc\u0645\u062a (\u06a9\u0631\u062f\u06cc\u062a) — \u06f1\u06f0\u06f0\u06f0 \u06a9\u0631\u062f\u06cc\u062a ≈ \u06f1$</label>
          <input id="wPrice" type="number" min="5" max="200000"
            value="${data.price_credits || ''}" aria-label="\u0645\u062b\u0644\u0627\u064b \u06f2\u06f5\u06f0\u06f0">
          <div style="color:var(--dim);font-size:11px;margin-top:6px">
            \u0645\u0639\u0627\u062f\u0644: ≈<span id="usdLive">${data.price_credits ? (data.price_credits / 1000).toFixed(2) : '0.00'}</span>$
            · \u06a9\u0645\u06cc\u0633\u06cc\u0648\u0646 \u067e\u0644\u062a\u0641\u0631\u0645 \u0628\u0639\u062f \u0627\u0632 \u0641\u0631\u0648\u0634 \u06a9\u0633\u0631 \u0645\u06cc‌\u0634\u0648\u062f</div></div>
        <div style="display:flex;gap:9px">
          <button class="btn btn-ghost" onclick="DGX.pages.create._go(1)">→ \u0642\u0628\u0644</button>
          <button class="btn btn-primary" id="wNext" style="flex:1">\u0628\u0639\u062f\u06cc ← \u0639\u06a9\u0633‌\u0647\u0627 ${DGX.icon('image','ic-s')}</button></div>`;
      const pr = w.querySelector('#wPrice');
      pr.oninput = () => {
        const v = +pr.value || 0;
        w.querySelector('#usdLive').textContent = (v / 1000).toFixed(2);
      };
      w.querySelector('#wNext').onclick = () => {
        data.category = w.querySelector('#wCat').value;
        data.price_credits = Math.round(+pr.value || 0);
        if (data.price_credits < 5 || data.price_credits > 200000)
          return DGX.toast('\u0642\u06cc\u0645\u062a \u0628\u06cc\u0646 \u06f5 \u062a\u0627 \u06f2\u06f0\u06f0\u066c\u06f0\u06f0\u06f0 \u06a9\u0631\u062f\u06cc\u062a', true);
        step = 3; render(); DGX.haptic('light');
      };
    } else if (step === 3) {
      // ── IMAGE UPLOAD STEP ──
      w.innerHTML = `
        <div class="step-dots"><i class="on"></i><i class="on"></i><i class="on"></i><i></i></div>
        <h2 style="margin-bottom:4px;display:flex;align-items:center;gap:7px">${DGX.icon('image')} \u0633\u0647 \u0639\u06a9\u0633 \u0644\u0627\u0632\u0645 \u0627\u0633\u062a</h2>
        <p style="color:var(--dim);font-size:11.5px;margin-bottom:14px;line-height:2">
          \u0633\u06cc\u0633\u062a\u0645 \u062e\u0648\u062f\u06a9\u0627\u0631 \u0628\u0631\u0634 \u0645\u06cc‌\u0632\u0646\u062f \u0648 \u0628\u0647 \u0627\u0646\u062f\u0627\u0632\u0647\u0654 \u062f\u0631\u0633\u062a \u062a\u0628\u062f\u06cc\u0644 \u0645\u06cc‌\u06a9\u0646\u062f.<br>
          \u06a9\u06cc\u0641\u06cc\u062a \u0628\u0627\u0644\u0627\u062a\u0631 = \u0646\u0645\u0627\u06cc\u0634 \u0628\u0647\u062a\u0631 = \u0641\u0631\u0648\u0634 \u0628\u06cc\u0634\u062a\u0631 ✨</p>
        ${Object.entries(IMG_SPECS).map(([key, spec]) => `
          <div class="field">
            <label style="display:flex;align-items:center;gap:6px">${DGX.icon(spec.icon,'ic-s')} ${key.toUpperCase()} (${spec.ratio}) — ${spec.hint}</label>
            <div style="display:flex;gap:10px;align-items:center">
              <label class="dropzone" style="flex:none;width:${key==='story'?'70':'110'}px;
                height:${key==='main'||key==='feed'?'70':'110'}px;padding:0;display:flex;
                align-items:center;justify-content:center;font-size:22px;cursor:pointer;
                border-radius:12px;border:2px dashed var(--line);overflow:hidden"
                id="dz_${key}">
                ${previews[key] ? `<img src="${previews[key]}" style="width:100%;height:100%;object-fit:cover">` : DGX.icon('camera','ic-l ph-g')}
              </label>
              <div style="flex:1;font-size:11px;color:var(--dim)">
                ${spec.ratio} · ${spec.w}×${spec.h}px<br>
                <span id="sz_${key}">${images[key] ? '✓ \u0627\u0646\u062a\u062e\u0627\u0628 \u0634\u062f (' + Math.round(images[key].size/1024) + 'KB)' : '\u0627\u0646\u062a\u062e\u0627\u0628 \u0646\u0634\u062f\u0647'}</span>
              </div>
            </div>
            <input type="file" id="file_${key}" accept="image/jpeg,image/png,image/webp" hidden
              onchange="DGX.pages.create._pick('${key}',this)">
          </div>`).join('')}
        <div style="background:#fff3;color:var(--gold);border-radius:10px;padding:9px 14px;
             font-size:11px;margin:12px 0;line-height:1.9">
          ⚠️ \u0647\u0631 \u06f3 \u0639\u06a9\u0633 \u0627\u062c\u0628\u0627\u0631\u06cc \u0647\u0633\u062a\u0646\u062f — \u0628\u062f\u0648\u0646 \u0639\u06a9\u0633 \u0645\u062d\u0635\u0648\u0644 \u0630\u062e\u06cc\u0631\u0647 \u0646\u0645\u06cc‌\u0634\u0648\u062f.
          \u0633\u06cc\u0633\u062a\u0645 \u062e\u0648\u062f\u06a9\u0627\u0631 \u0628\u0631\u0634 \u0645\u06cc‌\u0632\u0646\u062f \u0648 resize \u0645\u06cc‌\u06a9\u0646\u062f.</div>
        <div style="display:flex;gap:9px">
          <button class="btn btn-ghost" onclick="DGX.pages.create._go(2)">→ \u0642\u0628\u0644</button>
          <button class="btn btn-primary" id="wNext" style="flex:1">\u0628\u0639\u062f\u06cc ← \u0686\u06a9 \u0646\u0647\u0627\u06cc\u06cc</button></div>`;

      // wire file inputs
      Object.keys(IMG_SPECS).forEach(key => {
        const inp = w.querySelector(`#file_${key}`);
        if (!inp) return;
        inp.onchange = () => {
          const f = inp.files[0]; if (!f) return;
          images[key] = f;
          previews[key] = URL.createObjectURL(f);
          render();
          DGX.haptic('light');
        };
        // click dropzone triggers file input
        const dz = w.querySelector(`#dz_${key}`);
        if (dz) dz.onclick = () => inp.click();
      });

      w.querySelector('#wNext').onclick = () => {
        const missing = Object.keys(images).filter(k => !images[k]);
        if (missing.length) return DGX.toast(`${missing.length} \u0639\u06a9\u0633 \u0627\u0646\u062a\u062e\u0627\u0628 \u0646\u0634\u062f\u0647!`, true);
        step = 4; render(); DGX.haptic('light');
      };
    } else {
      // Step 4 — final check with real image previews
      const catIcon = (cats.find(c => c.key === data.category) || {}).key ? DGX.catIcon(data.category) : DGX.icon('box','ic-l');
      const heroSrc = previews.feed || previews.main ||
        `data:image/svg+xml,${encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" width="400" height="225"><rect fill="#101613"/><text x="50%" y="50%" text-anchor="middle" dy=".3em" font-size="48">' + catIcon + '</text></svg>')}`;

      w.innerHTML = `
        <div class="step-dots"><i class="on"></i><i class="on"></i><i class="on"></i><i class="on"></i></div>
        <h2 style="margin-bottom:12px">✅ \u067e\u06cc\u0634‌\u0646\u0645\u0627\u06cc\u0634 \u0646\u0647\u0627\u06cc\u06cc</h2>

        <!-- feed preview -->
        <div class="post">
          <div class="post-head"><img class="avatar" src="/app/assets/logo.jpg">
            <div><div class="p-name">${DGX.esc(DGX.user?.name || '\u062a\u0648')} ✔</div>
              <div class="p-sub">@${DGX.esc(DGX.user?.username || 'dropagentx')} · \u0627\u0644\u0627\u0646</div></div></div>
          <div class="post-media">${heroSrc.startsWith('data:')
            ? `<img src="${heroSrc}" style="width:100%">`
            : `<img src="${heroSrc}" style="width:100%;aspect-ratio:16/9;object-fit:cover">`}</div>
          <div class="post-body">
            <div class="post-title">${DGX.esc(data.title)}</div>
            <div class="post-desc">${DGX.esc(data.description)}</div>
            <div class="price-row"><span class="price num">
              ${DGX.fmt(data.price_credits)} <small>\u06a9\u0631\u062f\u06cc\u062a</small></span>
              <span class="usd num">≈${(data.price_credits / 1000).toFixed(2)}$</span></div>
          </div></div>

        <!-- 3 image thumbnails -->
        <div style="display:flex;gap:8px;margin:10px 0">
          ${Object.entries(previews).map(([k, url]) => url ? `
            <div style="flex:1;text-align:center">
              <img src="${url}" style="width:100%;aspect-ratio:${k === 'main' ? '1' : k === 'feed' ? '16/9' : '9/16'};
                   object-fit:cover;border-radius:8px;border:1px solid var(--line)">
              <div style="font-size:9px;color:var(--dim);margin-top:3px">${IMG_SPECS[k].icon} ${k}</div>
            </div>` : '').join('')}
        </div>

        <div style="background:#fff3;color:var(--gold);border-radius:12px;padding:10px 14px;
             font-size:11.5px;line-height:1.9;margin-bottom:12px">
          ⏳ \u067e\u0633 \u0627\u0632 \u0627\u0646\u062a\u0634\u0627\u0631\u060c \u0627\u062f\u0645\u06cc\u0646 \u0628\u0631\u0631\u0633\u06cc \u0645\u06cc‌\u06a9\u0646\u062f \u0648 \u062f\u0631 \u0645\u0627\u0631\u06a9\u062a \u0645\u0646\u062a\u0634\u0631 \u0645\u06cc‌\u0634\u0648\u062f.</div>
        <div style="display:flex;gap:9px">
          <button class="btn btn-ghost" onclick="DGX.pages.create._go(3)">→ \u0642\u0628\u0644</button>
          <button class="btn btn-primary" id="wPub" style="flex:1">${DGX.icon('rocket','ic-s')} \u0627\u0646\u062a\u0634\u0627\u0631 \u0645\u062d\u0635\u0648\u0644</button></div>
        <div style="height:80px"></div>`;

      w.querySelector('#wPub').onclick = async () => {
        const b = w.querySelector('#wPub');
        b.disabled = true; b.textContent = '⏳ \u062f\u0631 \u062d\u0627\u0644 \u0627\u0646\u062a\u0634\u0627\u0631…';
        try {
          // Step A: create product (JSON)
          const r = await DGX.api('/api/app/create-product', { body: data });
          const newPid = r.id;

          // Step B: upload 3 images (multipart)
          const fd = new FormData();
          fd.append('img_main', images.main);
          fd.append('img_feed', images.feed);
          fd.append('img_story', images.story);
          const upRes = await fetch(`/api/app/product/${newPid}/images`, {
            method: 'POST',
            headers: { 'Authorization': 'Bearer ' + DGX.token, 'X-Requested-With': 'fetch' },
            body: fd
          });
          if (!upRes.ok) {
            const ed = await upRes.json().catch(() => ({}));
            throw new Error(ed.detail || '\u0622\u067e\u0644\u0648\u062f \u0639\u06a9\u0633 \u0646\u0627\u0645\u0648\u0641\u0642');
          }

          b.textContent = '🎉 \u0645\u0646\u062a\u0634\u0631 \u0634\u062f!';
          DGX.haptic('heavy');
          view.innerHTML = `
            <div class="empty" style="padding-top:90px">
              <div class="burst success-ring" style="width:74px;height:74px;border-radius:99px;
                background:var(--em);color:#03130a;display:inline-flex;align-items:center;
                justify-content:center;margin-bottom:14px">${DGX.icon('rocket','ic-xl')}</div>
              <b style="font-size:17px">\u0645\u062d\u0635\u0648\u0644\u062a \u0628\u0627 \u06f3 \u0639\u06a9\u0633 \u062b\u0628\u062a \u0634\u062f!</b><br>
              \u0634\u0645\u0627\u0631\u0647: #${newPid}<br>${r.note}<br><br>
              <button class="btn btn-primary" onclick="location.hash='#/home'">\u0628\u0631\u06af\u0634\u062a \u0628\u0647 \u0641\u06cc\u062f</button>
            </div>`;
        } catch (e) {
          b.disabled = false; b.textContent = '\u0627\u0646\u062a\u0634\u0627\u0631 \u0645\u062d\u0635\u0648\u0644';
          DGX.toast(e.msg || e.message || '', true);
        }
      };
    }
  }

  // navigation helper (re-render with target step)
  DGX.pages.create._go = s => { step = s; render(); };

  render();
};
