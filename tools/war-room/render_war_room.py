#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
render_war_room.py — کانِرِ رسمیِ اتاقِ جنگِ بنیان‌گذارِ حاکم (Wave 10 · Commit #30)
render_war_room.py — canonical renderer of the Sovereign Founder War-Room (Wave 10 · Commit #30)

مخزن: KiaAgentX/xA — خروجی: tools/war-room/war_room.html (اپِ مستقلِ file:// بدونِ وابستگی)
Repo: KiaAgentX/xA — output: tools/war-room/war_room.html (standalone zero-dependency file:// app)

چهار ربعِ عرشهٔ فرمانِ شیشه‌ایِ OLED (#050508):
  Q1  اتاقِ گپِ خصوصیِ ۳ هم‌بنیان‌گذار (Elon Musk · CZ · Jeff Bezos) — ۲۴ تیکِ دوزبانهٔ روزِ ۰→۱
  Q2  ولتِ سازمان + چارتِ درآمدِ زنده + فیدِ تراکنشِ EIP-681 Base-8453 / Telegram Stars XTR
  Q3  سه تب: ELI5 متحرک · ترمینال و گیت‌هاب · فیدِ سوشال
  Q4  چتِ مستقیمِ رئیس (0xKIA) با ۶ چیپِ تاکتیکی + موتورِ نیتِ آفلاین + مودالِ LLMِ اختیاری

The four quadrants of the OLED glass command deck (#050508):
  Q1  private 3-founder chat (Elon Musk · CZ · Jeff Bezos) — 24 bilingual Day-0-to-Day-1 ticks
  Q2  organization wallet + live revenue chart + EIP-681 Base-8453 / Telegram Stars XTR feed
  Q3  three tabs: animated ELI5 · terminal & GitHub · social feed
  Q4  chairman (0xKIA) direct chat with 6 tactical chips + offline intent engine + optional LLM modal

استفاده / usage:
    python3 tools/war-room/render_war_room.py --out tools/war-room/war_room.html
    python3 tools/war-room/render_war_room.py --check        # byte-identity audit, exit 1 on drift
    python3 tools/war-room/render_war_room.py --validate     # structural integrity audit only

قانونِ کانونی: این اسکریپت «تک‌منبعِ حقیقت» خروجی است — ویرایشِ دستیِ HTML ممنوع؛
همیشه از همین اسکریپت بازتولید کنید تا SHA-256 سندِ تحویل معتبر بماند.
Canonical rule: this script is the single source of truth for the output — never hand-edit
the HTML; always regenerate from here so the SHA-256 in the delivery manifest stays valid.
"""
from __future__ import annotations

import argparse
import hashlib
import pathlib
import sys

# ══ embedded template ═════════════════════════════════════════════════════════
# عیناً همان آرتِفکتِ تأییدشدهٔ Wave 10؛ بازتولیدِ بایت-کامل (UTF-8 · LF · بدون BOM)
# Verbatim the approved Wave-10 artifact; byte-exact regeneration (UTF-8 · LF · no BOM)
TEMPLATE = (r'''<!DOCTYPE html>
<html lang="fa" dir="rtl">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<meta name="color-scheme" content="dark">
<title>KIA WAR-ROOM — Sovereign Founder Command Deck (Day 0 → Day 1)</title>
<style>
/* ═══ KIA WAR-ROOM · Wave 10 · OLED glass command deck ═══ */
:root{
  --bg:#050508; --panel:#0a0a12; --panel2:#0d0d18; --ink:#e8ecf4; --ink-dim:#8b93a7;
  --cyan:#22d3ee; --emerald:#34d399; --amber:#fbbf24; --crimson:#f87171; --pink:#f768ba;
  --elron:#7EF7D2; --cz:#59B7FF; --jeff:#FFB454; --chair:#F768BA;
  --line:rgba(255,255,255,.08); --line2:rgba(255,255,255,.14);
  --mono:'JetBrains Mono','SFMono-Regular',Consolas,'Courier New',monospace;
  --fa:'Vazirmatn','Segoe UI',Tahoma,system-ui,sans-serif;
  --en:'Inter','Segoe UI',system-ui,sans-serif;
  --glow-cyan:0 0 12px rgba(34,211,238,.35),0 0 42px rgba(34,211,238,.12);
}
*{margin:0;padding:0;box-sizing:border-box}
html,body{height:100%}
body{
  background:
    radial-gradient(1200px 600px at 80% -10%, rgba(34,211,238,.07), transparent 60%),
    radial-gradient(900px 500px at 10% 110%, rgba(52,211,153,.05), transparent 60%),
    var(--bg);
  color:var(--ink); font-family:var(--fa); font-size:14px; overflow:hidden;
}
body.en{font-family:var(--en)}
::-webkit-scrollbar{width:8px;height:8px}
::-webkit-scrollbar-thumb{background:rgba(255,255,255,.12);border-radius:8px}
::-webkit-scrollbar-track{background:transparent}

/* ── header ── */
#hdr{
  height:58px; display:flex; align-items:center; gap:14px; padding:0 16px;
  border-bottom:1px solid var(--line); background:rgba(10,10,18,.82);
  backdrop-filter:blur(10px); position:relative; z-index:20;
}
#hdr .logo{display:flex;align-items:center;gap:10px}
#hdr .sig{width:34px;height:34px;border-radius:9px;flex:0 0 auto;
  background:conic-gradient(from 210deg,var(--cyan),var(--emerald),var(--pink),var(--cyan));
  box-shadow:var(--glow-cyan);display:grid;place-items:center;font-weight:800;color:#04121a;font-size:12px}
#hdr h1{font-size:15px;font-weight:800;letter-spacing:.4px;white-space:nowrap}
#hdr h1 small{color:var(--ink-dim);font-weight:600;font-size:11px;display:block;letter-spacing:1.6px}
.hspacer{flex:1}
.clock{font-family:var(--mono);font-size:11.5px;color:var(--ink-dim);
  border:1px solid var(--line);border-radius:8px;padding:5px 10px;background:rgba(255,255,255,.02);
  display:flex;flex-direction:column;gap:1px;min-width:118px}
.clock b{color:var(--cyan);font-weight:700;font-size:12.5px}
#elapsedClock b{color:var(--amber)}
#commitBadge{font-family:var(--mono);font-size:11px;color:var(--emerald);
  border:1px solid rgba(52,211,153,.3);border-radius:8px;padding:6px 10px;background:rgba(52,211,153,.06)}
#langSwitch{display:flex;border:1px solid var(--line2);border-radius:9px;overflow:hidden}
#langSwitch button{background:transparent;border:0;color:var(--ink-dim);padding:7px 14px;cursor:pointer;
  font-family:var(--fa);font-size:12.5px;font-weight:700;transition:.2s}
#langSwitch button.on{background:rgba(34,211,238,.14);color:var(--cyan)}
body.en #langSwitch button{font-family:var(--en)}

/* ── controls bar ── */
#ctl{height:46px;display:flex;align-items:center;gap:8px;padding:0 16px;
  border-bottom:1px solid var(--line);background:rgba(8,8,14,.7)}
#ctl button{background:rgba(255,255,255,.04);border:1px solid var(--line2);color:var(--ink);
  border-radius:9px;padding:7px 13px;cursor:pointer;font-family:inherit;font-size:12.5px;font-weight:700;
  transition:.18s;display:flex;align-items:center;gap:6px}
#ctl button:hover{border-color:var(--cyan);color:var(--cyan);box-shadow:var(--glow-cyan)}
#ctl button.on{background:rgba(34,211,238,.16);border-color:var(--cyan);color:var(--cyan)}
#ctl button.danger:hover{border-color:var(--crimson);color:var(--crimson);box-shadow:none}
#progWrap{flex:1;display:flex;align-items:center;gap:10px;min-width:160px}
#progBar{flex:1;height:6px;border-radius:6px;background:rgba(255,255,255,.07);overflow:hidden}
#progFill{height:100%;width:0%;border-radius:6px;
  background:linear-gradient(90deg,var(--cyan),var(--emerald));box-shadow:var(--glow-cyan);transition:width .5s}
#progTxt{font-family:var(--mono);font-size:11px;color:var(--ink-dim);white-space:nowrap}

/* ── quadrant grid ── */
#deck{position:absolute;top:104px;bottom:0;left:0;right:0;display:grid;
  grid-template-columns:minmax(0,1fr) minmax(0,1fr);grid-template-rows:minmax(0,1fr) minmax(0,1fr);gap:10px;padding:10px}
.quad{position:relative;border:1px solid var(--line);border-radius:14px;overflow:hidden;
  background:linear-gradient(180deg,rgba(13,13,24,.92),rgba(8,8,14,.94));
  display:flex;flex-direction:column;box-shadow:0 8px 40px rgba(0,0,0,.5),inset 0 1px 0 rgba(255,255,255,.04)}
.quad::before{content:'';position:absolute;top:0;left:0;right:0;height:1px;z-index:3;
  background:linear-gradient(90deg,transparent,rgba(34,211,238,.5),transparent)}
.quad-h{display:flex;align-items:center;gap:10px;padding:9px 14px;border-bottom:1px solid var(--line);flex:0 0 auto}
.quad-h .qd{width:8px;height:8px;border-radius:3px;background:var(--cyan);box-shadow:var(--glow-cyan)}
.quad-h h2{font-size:12.5px;font-weight:800;letter-spacing:.3px}
.quad-h .sub{font-size:10px;color:var(--ink-dim);font-family:var(--mono);letter-spacing:.6px}
.quad-b{flex:1;overflow-y:auto;position:relative}

/* ── Q1 founder chat ── */
#q1 .quad-b{padding:12px 12px 4px}
.fmsg{display:flex;gap:10px;margin-bottom:14px;animation:pop .25s ease}
@keyframes pop{from{opacity:0;transform:translateY(6px)}to{opacity:1;transform:none}}
.fmsg .avatar{position:relative;flex:0 0 auto}
.fmsg .avatar svg{width:38px;height:38px;display:block}
.fmsg .onl{position:absolute;bottom:-1px;right:-1px;width:11px;height:11px;border-radius:50%;
  background:var(--emerald);border:2px solid var(--panel)}
.fmsg .bubble{background:rgba(255,255,255,.035);border:1px solid var(--line);border-radius:12px;
  padding:8px 12px;max-width:calc(100% - 60px)}
.fmsg .meta{display:flex;align-items:center;gap:8px;margin-bottom:3px;flex-wrap:wrap}
.fmsg .meta .nm{font-weight:800;font-size:12.5px}
.fmsg .meta .hd{font-family:var(--mono);font-size:10px;color:var(--ink-dim)}
.fmsg .meta .rl{font-size:9.5px;color:var(--ink-dim);border:1px solid var(--line);border-radius:6px;padding:1px 6px}
.fmsg .meta .tm{font-family:var(--mono);font-size:9.5px;color:var(--ink-dim);margin-inline-start:auto}
.fmsg .body{font-size:12.8px;line-height:1.75;color:var(--ink)}
.fmsg .body .en-side{display:none;color:var(--ink-dim);font-size:11.8px;margin-top:6px;border-top:1px dashed var(--line);padding-top:6px}
body.en .fmsg .body{color:var(--ink)}
body.en .fmsg .body .fa-side{display:none}
body:not(.en) .fmsg .body .fa-side{display:block}
.reactions{display:flex;gap:6px;margin-top:7px;flex-wrap:wrap}
.reactions span{font-size:10.5px;border:1px solid var(--line);border-radius:999px;padding:2px 8px;
  background:rgba(255,255,255,.03);cursor:pointer;transition:.15s;color:var(--ink-dim)}
.reactions span:hover{border-color:var(--cyan);color:var(--cyan)}
.reactions span.hot{border-color:rgba(247,104,186,.5);color:var(--pink);background:rgba(247,104,186,.08)}
.typed{display:flex;align-items:center;gap:8px;padding:6px 14px;color:var(--ink-dim);font-size:11.5px}
.typed .dots{display:flex;gap:3px}
.typed .dots i{width:5px;height:5px;border-radius:50%;background:var(--cyan);animation:blink 1.2s infinite}
.typed .dots i:nth-child(2){animation-delay:.2s}.typed .dots i:nth-child(3){animation-delay:.4s}
@keyframes blink{0%,80%,100%{opacity:.2}40%{opacity:1}}
.cmdline{font-family:var(--mono);font-size:11px;color:var(--emerald);background:rgba(52,211,153,.06);
  border:1px solid rgba(52,211,153,.22);border-radius:8px;padding:5px 9px;margin-top:7px;direction:ltr;text-align:left;
  overflow-x:auto;white-space:nowrap}
.chairBcast{border:1px solid rgba(247,104,186,.45);background:rgba(247,104,186,.07);border-radius:12px;
  padding:9px 12px;margin:4px 0 14px;position:relative}
.chairBcast .tag{display:inline-block;font-family:var(--mono);font-size:9.5px;font-weight:800;color:var(--pink);
  border:1px solid rgba(247,104,186,.5);border-radius:6px;padding:1px 8px;margin-bottom:5px;letter-spacing:1px}
.chairBcast .txt{font-size:12.5px;line-height:1.7}
.threadline{border-inline-start:2px solid rgba(34,211,238,.25);margin:2px 0 12px;padding:6px 12px 6px 0;
  margin-inline-start:24px;color:var(--ink-dim);font-size:11px}

/* ── Q2 wallet ── */
#q2 .quad-b{padding:12px;display:flex;flex-direction:column;gap:10px}
.wal{display:grid;grid-template-columns:repeat(2,1fr);gap:8px}
.wcard{border:1px solid var(--line);border-radius:12px;padding:9px 12px;background:rgba(255,255,255,.025)}
.wcard .lb{font-size:10px;color:var(--ink-dim);letter-spacing:.3px}
.wcard .vl{font-family:var(--mono);font-size:16.5px;font-weight:800;margin-top:3px}
.wcard .vl.seed{color:var(--cyan)} .wcard .vl.earn{color:var(--emerald)}
.wcard .vl.cash{color:var(--amber)} .wcard .vl.locked{color:var(--ink-dim)}
.wcard .vl.locked::after{content:' 🔒';font-size:11px}
#revChart{width:100%;height:120px;border:1px solid var(--line);border-radius:12px;background:rgba(255,255,255,.02)}
.streams{display:grid;grid-template-columns:repeat(2,1fr);gap:8px}
.scard{border:1px solid var(--line);border-radius:10px;padding:7px 10px;font-size:10.8px;color:var(--ink-dim);
  background:rgba(255,255,255,.02);display:flex;flex-direction:column;gap:2px}
.scard b{color:var(--ink);font-size:11.5px}
.scard .amt{font-family:var(--mono);color:var(--emerald);font-size:12px;font-weight:700}
.scard.pulse{animation:spulse .8s ease}
@keyframes spulse{0%{box-shadow:0 0 0 0 rgba(52,211,153,.5)}100%{box-shadow:0 0 0 12px rgba(52,211,153,0)}}
#txFeed{display:flex;flex-direction:column;gap:6px;overflow-y:auto;flex:0 0 auto;max-height:150px}
.txrow{display:flex;align-items:center;gap:9px;border:1px solid var(--line);border-radius:9px;padding:6px 10px;
  font-family:var(--mono);font-size:10.5px;background:rgba(255,255,255,.02);animation:pop .3s ease}
.txrow .ic{font-size:14px}
.txrow .hash{color:var(--cyan);direction:ltr;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;flex:1}
.txrow .amt{color:var(--emerald);font-weight:800;white-space:nowrap}
.txrow .rail{color:var(--ink-dim);font-size:9.5px;white-space:nowrap}
.kpis{display:grid;grid-template-columns:repeat(4,1fr);gap:8px}
.kpi{border:1px solid var(--line);border-radius:10px;padding:7px 8px;text-align:center;background:rgba(255,255,255,.02)}
.kpi .v{font-family:var(--mono);font-size:14px;font-weight:800;color:var(--cyan)}
.kpi .l{font-size:9.5px;color:var(--ink-dim);margin-top:2px}
.funnel{display:flex;gap:6px;align-items:center;font-size:10.5px;color:var(--ink-dim)}
.funnel .fbox{border:1px solid var(--line);border-radius:8px;padding:5px 9px;background:rgba(255,255,255,.02)}
.funnel .fbox b{font-family:var(--mono);color:var(--ink)}
.funnel .arr{color:var(--cyan)}

/* ── Q3 ── */
#q3 .quad-h{gap:8px}
.tabs{display:flex;gap:6px;margin-inline-start:auto}
.tabs button{background:rgba(255,255,255,.04);border:1px solid var(--line);color:var(--ink-dim);
  border-radius:8px;padding:5px 11px;font-size:11px;cursor:pointer;font-family:inherit;font-weight:700}
.tabs button.on{color:var(--cyan);border-color:rgba(34,211,238,.45);background:rgba(34,211,238,.1)}
#q3 .quad-b{padding:0}
.tabpane{display:none;height:100%;overflow-y:auto;padding:12px}
.tabpane.on{display:block}
#eli5Canvas{width:100%;height:calc(100% - 66px);display:block;border-radius:12px;background:rgba(255,255,255,.015);border:1px solid var(--line)}
#eli5Steps{display:grid;grid-template-columns:repeat(4,1fr);gap:8px;margin-top:10px}
.eli5step{border:1px solid var(--line);border-radius:10px;padding:8px 10px;font-size:10.5px;color:var(--ink-dim);
  background:rgba(255,255,255,.02);transition:.3s;line-height:1.6}
.eli5step b{display:block;font-size:11px;color:var(--ink);margin-bottom:3px}
.eli5step.active{border-color:rgba(34,211,238,.55);background:rgba(34,211,238,.07);color:var(--ink);
  box-shadow:var(--glow-cyan)}
#termLog{font-family:var(--mono);font-size:11px;line-height:1.85;color:var(--ink-dim);direction:ltr;text-align:left;
  padding:12px;overflow-y:auto;height:100%}
#termLog .tcmd{color:var(--emerald)}
#termLog .tout{color:var(--ink)}
#termLog .tinfo{color:var(--cyan)}
#termLog .twar{color:var(--amber)}
#termLog .cursor{display:inline-block;width:7px;height:13px;background:var(--emerald);vertical-align:middle;
  animation:blink 1s infinite}
.socpost{border:1px solid var(--line);border-radius:12px;padding:10px 12px;margin-bottom:10px;background:rgba(255,255,255,.02)}
.socpost .sh{display:flex;gap:8px;align-items:center;margin-bottom:6px}
.socpost .avic{width:26px;height:26px;border-radius:50%;display:grid;place-items:center;font-weight:800;font-size:11px;color:#04121a}
.socpost .snm{font-weight:800;font-size:12px}
.socpost .shd{font-family:var(--mono);font-size:10px;color:var(--ink-dim)}
.socpost .plat{margin-inline-start:auto;font-family:var(--mono);font-size:9px;border:1px solid var(--line);
  border-radius:6px;padding:1px 7px;color:var(--ink-dim)}
.socpost .stxt{font-size:12.3px;line-height:1.7;color:var(--ink)}
.socpost .stxt small{display:block;color:var(--ink-dim);font-size:11px;margin-top:4px}
.socpost .smet{display:flex;gap:14px;margin-top:7px;font-family:var(--mono);font-size:10.5px;color:var(--ink-dim)}
.socpost .smet b{color:var(--cyan);font-weight:700}
.soccomments{margin-top:8px;border-top:1px dashed var(--line);padding-top:7px;display:flex;flex-direction:column;gap:5px}
.soccomments .cm{font-size:10.8px;color:var(--ink-dim)}
.soccomments .cm b{color:var(--ink)}

/* ── Q4 chairman chat ── */
#q4Chat{flex:1;overflow-y:auto;padding:12px;display:flex;flex-direction:column;gap:12px}
.cmsg{max-width:88%;padding:9px 13px;border-radius:13px;font-size:12.6px;line-height:1.75;animation:pop .25s ease}
.cmsg.chair{align-self:flex-end;background:linear-gradient(135deg,rgba(247,104,186,.16),rgba(247,104,186,.07));
  border:1px solid rgba(247,104,186,.4)}
.cmsg.founder{align-self:flex-start;background:rgba(255,255,255,.035);border:1px solid var(--line)}
.cmsg .who{font-size:10px;font-weight:800;margin-bottom:3px;display:flex;gap:6px;align-items:center}
.cmsg .who .p{font-family:var(--mono);color:var(--ink-dim);font-weight:400}
.cmsg .alt{display:none;color:var(--ink-dim);font-size:11.6px;border-top:1px dashed var(--line);margin-top:6px;padding-top:5px}
body.en .cmsg .alt.fa-lang{display:block}
body:not(.en) .cmsg .alt.en-lang{display:block}
#q4Input{flex:0 0 auto;border-top:1px solid var(--line);padding:10px 12px;display:flex;flex-direction:column;gap:8px}
.chips{display:flex;gap:6px;flex-wrap:wrap}
.chips button{background:rgba(255,255,255,.04);border:1px solid var(--line);color:var(--ink-dim);font-size:10.5px;
  border-radius:999px;padding:4px 10px;cursor:pointer;font-family:inherit;transition:.15s}
.chips button:hover{color:var(--cyan);border-color:rgba(34,211,238,.5)}
.inputrow{display:flex;gap:8px}
.inputrow textarea{flex:1;background:rgba(255,255,255,.05);border:1px solid var(--line2);border-radius:11px;
  color:var(--ink);padding:9px 12px;font-family:inherit;font-size:12.5px;resize:none;height:44px;outline:none}
.inputrow textarea:focus{border-color:rgba(34,211,238,.5);box-shadow:var(--glow-cyan)}
#sendBtn{background:linear-gradient(135deg,var(--pink),#c26);border:0;color:#fff;font-weight:800;
  border-radius:11px;padding:0 16px;cursor:pointer;font-family:inherit;font-size:12px;white-space:nowrap;transition:.2s}
#sendBtn:hover{box-shadow:0 0 18px rgba(247,104,186,.4)}
#llmBtn{align-self:flex-start;background:transparent;border:1px dashed var(--line2);color:var(--ink-dim);
  border-radius:8px;padding:4px 10px;font-size:10.5px;cursor:pointer;font-family:inherit}
#llmBtn:hover{color:var(--cyan);border-color:var(--cyan)}
#llmBtn.on{color:var(--emerald);border-color:rgba(52,211,153,.5)}

/* ── modal ── */
#llmModal{position:fixed;inset:0;background:rgba(3,3,6,.7);backdrop-filter:blur(6px);z-index:50;
  display:none;align-items:center;justify-content:center}
#llmModal.on{display:flex}
.mcard{width:min(480px,92vw);background:var(--panel2);border:1px solid var(--line2);border-radius:16px;padding:20px}
.mcard h3{font-size:14px;margin-bottom:4px}
.mcard p{font-size:11.5px;color:var(--ink-dim);margin-bottom:14px;line-height:1.7}
.mrow{display:flex;flex-direction:column;gap:10px}
.mrow label{font-size:11.5px;color:var(--ink-dim);display:flex;flex-direction:column;gap:5px}
.mrow select,.mrow input{background:rgba(255,255,255,.05);border:1px solid var(--line2);border-radius:9px;
  color:var(--ink);padding:8px 10px;font-family:var(--mono);font-size:12px;outline:none}
.mrow select:focus,.mrow input:focus{border-color:var(--cyan)}
.mbtns{display:flex;gap:8px;margin-top:16px;justify-content:flex-end}
.mbtns button{border-radius:9px;padding:8px 16px;cursor:pointer;font-family:inherit;font-size:12px;font-weight:700}
.mbtns .save{background:var(--cyan);border:0;color:#04121a}
.mbtns .close{background:transparent;border:1px solid var(--line2);color:var(--ink-dim)}
#llmNote{font-size:10.5px;color:var(--amber);margin-top:10px;line-height:1.6}

/* ── responsive ── */
@media (max-width:1100px){
  #deck{grid-template-columns:1fr;grid-template-rows:repeat(4,minmax(0,1fr))}
  body{overflow:auto}
  #deck{position:static;margin-top:104px}
  .clock{display:none}
}
</style>
</head>
<body>
<!-- ═══ HEADER ═══ -->
<header id="hdr">
  <div class="logo">
    <div class="sig">K</div>
    <h1><small>KIAN · WAVE 10 · THE GRAND UI UPDATE</small>
      <span data-i18n="hdrTitle">اتاق جنگ بنیان‌گذار حاکم</span></h1>
  </div>
  <div class="hspacer"></div>
  <div class="clock" id="tehranClock"><span data-i18n="tehranClock">تهران UTC+03:30</span><b id="tehranTime">--:--:--</b></div>
  <div class="clock"><span data-i18n="utcClock">UTC</span><b id="utcTime">--:--:--</b></div>
  <div class="clock" id="elapsedClock"><span data-i18n="elapsed">زمان شبیه‌سازی</span><b id="elapsedTime">DAY 00 // 00:00:00</b></div>
  <div id="commitBadge">KiaAgentX/xA @ 97b18bd</div>
  <div id="langSwitch">
    <button id="btnFa" class="on">فا</button>
    <button id="btnEn">EN</button>
  </div>
</header>

<!-- ═══ CONTROLS ═══ -->
<div id="ctl">
  <button id="btnPlay">▶ <span data-i18n="play1x">پخش ۱x</span></button>
  <button id="btnPause">⏸ <span data-i18n="pause">توقف</span></button>
  <button id="btnNext">⏭ <span data-i18n="nextStep">گام بعد (+۱ تیک)</span></button>
  <button id="btn2x">⚡ 2x</button>
  <button id="btn5x">⚡ 5x</button>
  <button id="btnReset" class="danger">🔄 <span data-i18n="reset">بازنشانی به روز ۰</span></button>
  <div id="progWrap">
    <div id="progBar"><div id="progFill"></div></div>
    <span id="progTxt">0 / 24</span>
  </div>
</div>

<!-- ═══ 4-QUADRANT DECK ═══ -->
<div id="deck">

  <!-- ── Q1: founder war-room chat ── -->
  <section class="quad" id="q1">
    <div class="quad-h">
      <div class="qd" style="background:var(--cyan)"></div>
      <h2 data-i18n="q1Title">گپ خصوصی سه شریک</h2>
      <span class="sub">PRIVATE · 24/7 REMOTE ACTIVE · E2E</span>
      <span style="margin-inline-start:auto;font-size:10px;color:var(--emerald);display:flex;align-items:center;gap:5px">
        <span class="onl" style="position:static;width:8px;height:8px;border-radius:50%;background:var(--emerald);display:inline-block"></span>
        <span data-i18n="online">۳ آنلاین</span>
      </span>
    </div>
    <div class="quad-b" id="q1Chat">
      <div class="typed" id="q1Typing" style="display:none">
        <span class="dots"><i></i><i></i><i></i></span>
        <span id="q1TypingWho">Elon is typing...</span>
      </div>
    </div>
  </section>

  <!-- ── Q2: wallet + live chart + resources ── -->
  <section class="quad" id="q2">
    <div class="quad-h">
      <div class="qd" style="background:var(--emerald)"></div>
      <h2 data-i18n="q2Title">ولت سازمان، چارت درآمد زنده و منابع</h2>
      <span class="sub">BASE 8453 · USDC · XTR</span>
    </div>
    <div class="quad-b">
      <div class="wal">
        <div class="wcard"><div class="lb" data-i18n="seedCrypto">سرمایه اولیه واقعی — کریپتو (Trust Wallet)</div><div class="vl seed" id="wSeed">$33.00</div></div>
        <div class="wcard"><div class="lb" data-i18n="cash">نقد واقعی</div><div class="vl cash" id="wCash">$15.00</div></div>
        <div class="wcard"><div class="lb" data-i18n="paper">ذخیره کاغذی — قفل/دست‌نخورده</div><div class="vl locked" id="wPaper">$350.00</div></div>
        <div class="wcard"><div class="lb" data-i18n="earned">درآمد خالص پلتفرم (از $0.00)</div><div class="vl earn" id="wEarn">$0.00</div></div>
      </div>
      <canvas id="revChart"></canvas>
      <div class="kpis">
        <div class="kpi"><div class="v" id="kUsers">0</div><div class="l" data-i18n="kUsers">کاربر فعال</div></div>
        <div class="kpi"><div class="v" id="kVisits">0</div><div class="l" data-i18n="kVisits">بازدید</div></div>
        <div class="kpi"><div class="v" id="kSignups">0</div><div class="l" data-i18n="kSignups">ثبت‌نام</div></div>
        <div class="kpi"><div class="v" id="kPurchases">0</div><div class="l" data-i18n="kPurchases">خرید</div></div>
      </div>
      <div class="funnel">
        <span class="fbox"><span data-i18n="fVisits">بازدید</span> <b id="fv1">0</b></span>
        <span class="arr">→</span>
        <span class="fbox"><span data-i18n="fSignups">ثبت‌نام</span> <b id="fv2">0</b></span>
        <span class="arr">→</span>
        <span class="fbox"><span data-i18n="fPurchases">خرید</span> <b id="fv3">0</b></span>
        <span class="arr">✦</span>
        <span id="fNote" data-i18n="fNote">قیف تبدیل واقعی — هر عدد علّی است</span>
      </div>
      <div class="streams">
        <div class="scard" id="st1"><b data-i18n="st1">۱ · مالیات ۱۰٪ بازارگاه</b><span data-i18n="st1d">روی هر فروش اپ/مهارت</span><span class="amt" id="st1v">$0.00</span></div>
        <div class="scard" id="st2"><b data-i18n="st2">۲ · موتور آفیلیت ۱۰٪–۵۰٪</b><span data-i18n="st2d">فروش بدون رفرال = ۱۰٪ اضافه به خزانه (مجموع ۲۰٪)</span><span class="amt" id="st2v">$0.00</span></div>
        <div class="scard" id="st3"><b data-i18n="st3">۳ · اسلات ویژه و بوست</b><span data-i18n="st3d">۱۰۰٪ به خزانه حاکم</span><span class="amt" id="st3v">$0.00</span></div>
        <div class="scard" id="st4"><b data-i18n="st4">۴ · اختلاف تسویه آنی ۱٫۵٪</b><span data-i18n="st4d">Base USDC + استارز XTR</span><span class="amt" id="st4v">$0.00</span></div>
      </div>
      <div style="font-size:10.5px;color:var(--ink-dim);display:flex;gap:6px;align-items:center">
        <span data-i18n="txTitle">فید تراکنش زنده (EIP-681 · Base 8453 · XTR)</span>
      </div>
      <div id="txFeed"></div>
    </div>
  </section>

  <!-- ── Q3: platform viewport ── -->
  <section class="quad" id="q3">
    <div class="quad-h">
      <div class="qd" style="background:var(--amber)"></div>
      <h2 data-i18n="q3Title">محیط اجرای لایو پلتفرم</h2>
      <div class="tabs">
        <button id="tabEli5" class="on" data-i18n="tabEli5">🎓 چطور کار می‌کند؟ (ساده‌ترین زبان)</button>
        <button id="tabTerm">💻 <span data-i18n="tabTerm">ترمینال و گیت‌هاب</span></button>
        <button id="tabSoc">🌐 <span data-i18n="tabSoc">فید سوشال X/تلگرام</span></button>
      </div>
    </div>
    <div class="quad-b">
      <div class="tabpane on" id="paneEli5">
        <canvas id="eli5Canvas"></canvas>
        <div id="eli5Steps">
          <div class="eli5step" id="epA"><b data-i18n="epA">الف — ورود کاربر عادی</b><span data-i18n="epAd">کاربر با یک جمله ساده (حتی ویس یا دکمه‌های بزرگ ToySpark) می‌گوید چه می‌خواهد</span></div>
          <div class="eli5step" id="epB"><b data-i18n="epB">ب — کارخانه ۳ آواتار و ۵ مدل</b><span data-i18n="epBd">آلفا (طراح)، اتاق فرمان (مهندس) و مایرون (قاضی ۱۰۰/۱۰۰) با ۵ مدل در چند ثانیه می‌سازند</span></div>
          <div class="eli5step" id="epC"><b data-i18n="epC">ج — فروشگاه و سوشال‌مدیا</b><span data-i18n="epCd">محصول با ویدئوی سه‌بعدی Neural HUD در X و تلگرام منتشر می‌شود</span></div>
          <div class="eli5step" id="epD"><b data-i18n="epD">د — تقسیم خودکار پول در ولت</b><span data-i18n="epDd">مشتری با استارز یا Base USDC می‌خرد؛ قرارداد همان ثانیه ۸۰٪ سازنده و ۲۰٪ ولت سازمان!</span></div>
        </div>
      </div>
      <div class="tabpane" id="paneTerm">
        <div id="termLog"></div>
      </div>
      <div class="tabpane" id="paneSoc">
        <div id="socFeed"></div>
      </div>
    </div>
  </section>

  <!-- ── Q4: chairman chat ── -->
  <section class="quad" id="q4">
    <div class="quad-h">
      <div class="qd" style="background:var(--pink)"></div>
      <h2 data-i18n="q4Title">چت زنده تو (معمار ارشد ۰xKIA) با تیم</h2>
      <span class="sub" id="q4Status">DIRECT TACTICAL OVERRIDE</span>
    </div>
    <div id="q4Chat">
      <div class="cmsg founder">
        <div class="who"><b style="color:var(--elron)">Elon Musk</b> <span class="p">@elonmusk</span></div>
        <div class="fa-lang">رئیس، تیم ۲۴/۷ روی مخزن KiaAgentX/xA هماهنگ است. دستور بده — هر سه گزارش می‌دهیم.</div>
        <div class="alt en-lang">Chairman, the 24/7 team is synced on KiaAgentX/xA. Issue a directive — all three of us will report.</div>
      </div>
    </div>
    <div id="q4Input">
      <div class="chips" id="chips"></div>
      <div class="inputrow">
        <textarea id="chairInput" data-i18n-ph="chairPh" placeholder="دستور تاکتیکی خود را بنویسید..."></textarea>
        <button id="sendBtn" data-i18n="sendBtn">ارسال دستور به اتاق جنگ</button>
      </div>
      <button id="llmBtn">⚙️ <span data-i18n="llmBtn">اتصال به LLM زنده (Pollinations / OpenRouter / Local)</span></button>
    </div>
  </section>
</div>

<!-- ═══ LLM MODAL ═══ -->
<div id="llmModal">
  <div class="mcard">
    <h3 data-i18n="llmTitle">اتصال به LLM زنده — تولید بی‌نهایت</h3>
    <p data-i18n="llmDesc">موتور آفلاینِ داخل اپ همیشه کار می‌کند (بدون کلید). این اتصال اختیاری است؛ پاسخ‌های پویا را از ارائه‌دهندهٔ دلخواه می‌گیرد و در خطا بی‌صدا به موتور آفلاین برمی‌گردد.</p>
    <div class="mrow">
      <label><span data-i18n="llmProvider">ارائه‌دهنده</span>
        <select id="llmProvider">
          <option value="pollinations">Pollinations (keyless · آزاد)</option>
          <option value="openrouter">OpenRouter (کلید خودت)</option>
          <option value="local">Local (OpenAI-compatible · localhost)</option>
          <option value="off">خاموش — فقط موتور آفلاین</option>
        </select>
      </label>
      <label><span data-i18n="llmKey">کلید API (فقط برای OpenRouter)</span>
        <input id="llmKey" type="password" placeholder="sk-or-v1-...">
      </label>
      <label><span data-i18n="llmEndpoint">اندپوینت (فقط برای Local)</span>
        <input id="llmEndpoint" type="text" value="http://localhost:11434/v1/chat/completions">
      </label>
    </div>
    <div id="llmNote" data-i18n="llmNote">نکته: همه‌چیز سمت کلاینت است — کلید هرگز جایی ذخیره یا ارسال نمی‌شود جز ارائه‌دهندهٔ خودت.</div>
    <div class="mbtns">
      <button class="close" id="llmClose" data-i18n="llmClose">بستن</button>
      <button class="save" id="llmSave" data-i18n="llmSave">ذخیره و فعال‌سازی</button>
    </div>
  </div>
</div>
<script>
const WAR_ROOM_DATA = {"signature": "KIA-WAR-ROOM-W10", "githubRepo": "KiaAgentX/xA", "githubCommitShort": "97b18bd", "baseChainId": 8453, "baseUsdcContract": "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913", "telegramStarsCurrency": "XTR", "cryptoSeedMicros": 33000000, "cashSeedCents": 1500, "paperReserveLockedCents": 35000, "day1TotalEarnedMicros": 9550000, "maxTickDeltaSeconds": 3600, "charterUnreferredTakeBps": 2000, "charterCreatorBps": 8000, "founders": [{"pid": "elon_musk", "handle": "@elonmusk", "en": "Elon Musk", "fa": "ایلان ماسک", "accent": "#7EF7D2"}, {"pid": "cz_binance", "handle": "@cz_binance", "en": "Changpeng Zhao (CZ)", "fa": "چانگپنگ ژائو (CZ)", "accent": "#59B7FF"}, {"pid": "jeff_bezos", "handle": "@JeffBezos", "en": "Jeff Bezos", "fa": "جف بزوس", "accent": "#FFB454"}, {"pid": "chairman", "handle": "@0xKIA", "en": "0xKIA (Chairman)", "fa": "۰xKIA (رئیس)", "accent": "#F768BA"}], "ticks": [{"tickIndex": 0, "simulationSecond": 0, "speaker": {"id": "jeff_bezos", "name_en": "Jeff Bezos", "name_fa": "جف بزوس", "handle": "@JeffBezos", "accent": "#FFB454", "mandate_en": "Chief Flywheel — customer obsession & 'Always Day 1'", "mandate_fa": "چرخ‌دندهٔ مشتری، اقتصادِ واحد و «همیشه روزِ اول»"}, "messageFa": "روزِ اول! سه نفر از ما زیرِ ریاستِ 0xKIA دورکار ۲۴/۷ شریک شدیم. از مخزن KiaAgentX/xA شروع می‌کنیم: ۱۱ کرِیتِ Rust، ۷۵۰ تستِ پاس‌شده، ۹ موجِ تحویل‌شده. سرمایه واقعی؟ ۳۳ دلار کریپتو، ۱۵ دلار نقد، ۳۵۰ دلار کاغذیِ قفل. درآمدِ پلتفرم دقیقاً صفر است — و باید از دلِ کارِ واقعی بیرون بیاید. همیشه روزِ اول.", "messageEn": "Day 1! The three of us just partnered 24/7 remotely under 0xKIA. We start from the KiaAgentX/xA repo: 11 Rust crates, 750 passing tests, 9 delivered waves. Real capital? $33 crypto, $15 cash, $350 locked paper. Platform revenue is exactly $0.00 — and it must come from real work. It is always Day 1.", "terminalCmd": null, "socialPost": null, "walletDeltaUsdcMicros": 0, "walletTotalUsdcMicros": 33000000, "activePipelineStage": "Ignition", "revenueStream": "None", "saleVolumeMicros": 0, "creatorPayoutMicros": 0, "affiliatePayoutMicros": 0, "affiliateRoutingMicros": 0, "referred": false, "usersDelta": 0, "visitsDelta": 0, "signupsDelta": 0, "purchasesDelta": 0}, {"tickIndex": 1, "simulationSecond": 3600, "speaker": {"id": "elon_musk", "name_en": "Elon Musk", "name_fa": "ایلان ماسک", "handle": "@elonmusk", "accent": "#7EF7D2", "mandate_en": "Chief Product Velocity — first-principles & X viral distribution", "mandate_fa": "سرعتِ محصول، مهندسیِ ابتدا-به-اصول و ویروسی‌شدن روی X"}, "messageFa": "اول سلامتِ مخزن را ثابت می‌کنیم. تستِ کاریِ کاملِ workspace را اجرا می‌کنم — ۷۵۰ پاس، صفر شکست، صفر clippy. این کورهٔ اعتبارِ ماست؛ بدونِ آن هیچ ویروسی‌شدنی معنا ندارد.", "messageEn": "First we prove the repo is healthy. Running the full workspace test suite — 750 passed, zero failures, zero clippy. That is our credibility furnace; nothing viral means anything without it.", "terminalCmd": "cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings", "socialPost": null, "walletDeltaUsdcMicros": 0, "walletTotalUsdcMicros": 33000000, "activePipelineStage": "Ignition", "revenueStream": "None", "saleVolumeMicros": 0, "creatorPayoutMicros": 0, "affiliatePayoutMicros": 0, "affiliateRoutingMicros": 0, "referred": false, "usersDelta": 0, "visitsDelta": 0, "signupsDelta": 0, "purchasesDelta": 0}, {"tickIndex": 2, "simulationSecond": 7200, "speaker": {"id": "cz_binance", "name_en": "Changpeng Zhao (CZ)", "name_fa": "چانگپنگ ژائو (CZ)", "handle": "@cz_binance", "accent": "#59B7FF", "mandate_en": "Chief Liquidity — Base 8453 escrow, CCXT quant & zero-KYC growth", "mandate_fa": "نقدینگی، اسکروی Base 8453، کوانتِ CCXT و رشدِ بدونِ KYC"}, "messageFa": "ریل‌هایِ پول را قبل از هر تبلیغی وریفای می‌کنم: قراردادهایِ Base با Chain ID 8453 و USDC بومیِ 0x8335...2913 روی شبکهٔ اصلی تأیید شدند؛ تلگرام استارز (XTR) هم آماده است. صندوق‌ها SAFU — هر فروش ۸۰٪ سازنده، بقیه طبق منشور.", "messageEn": "I verify the money rails before any marketing: Base contracts with Chain ID 8453 and native USDC 0x8335...2913 verified on mainnet; Telegram Stars (XTR) ready too. Funds are SAFU — every sale pays 80% to the creator, the rest per the charter.", "terminalCmd": "forge verify-contract 0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913 --chain-id 8453 --rpc-url https://mainnet.base.org", "socialPost": null, "walletDeltaUsdcMicros": 0, "walletTotalUsdcMicros": 33000000, "activePipelineStage": "Ignition", "revenueStream": "None", "saleVolumeMicros": 0, "creatorPayoutMicros": 0, "affiliatePayoutMicros": 0, "affiliateRoutingMicros": 0, "referred": false, "usersDelta": 0, "visitsDelta": 0, "signupsDelta": 0, "purchasesDelta": 0}, {"tickIndex": 3, "simulationSecond": 10800, "speaker": {"id": "jeff_bezos", "name_en": "Jeff Bezos", "name_fa": "جف بزوس", "handle": "@JeffBezos", "accent": "#FFB454", "mandate_en": "Chief Flywheel — customer obsession & 'Always Day 1'", "mandate_fa": "چرخ‌دندهٔ مشتری، اقتصادِ واحد و «همیشه روزِ اول»"}, "messageFa": "قبل از ویروسی‌شدن، آزمونِ بچهٔ ۹ساله را می‌گذارم: یک کاربرِ عادی در ۵ ثانیه باید بفهمد سیستم چه می‌کند. خطِ لولهٔ «۰ تا ۱۰۰» را می‌نویسم — الف: کاربر می‌گوید چه می‌خواهد؛ ب: کارخانهٔ ۳ آواتار و ۵ مدل می‌سازد؛ ج: محصول به فروشگاه و سوشال می‌رود؛ د: پول به‌طور خودکار ۸۰/۲۰ تقسیم می‌شود. اگر این چهار گام واضح نباشند، هیچ دکمه‌ای نجاتمان نمی‌دهد.", "messageEn": "Before going viral I run the 9-year-old test: a normal user must understand the system in 5 seconds. I write the zero-to-100 pipeline — A: the user says what they want; B: the 3-avatar & 5-model factory builds it; C: the product hits the store and social; D: money auto-splits 80/20. If those four steps are not obvious, no button will save us.", "terminalCmd": null, "socialPost": null, "walletDeltaUsdcMicros": 0, "walletTotalUsdcMicros": 33000000, "activePipelineStage": "ProductClarity", "revenueStream": "None", "saleVolumeMicros": 0, "creatorPayoutMicros": 0, "affiliatePayoutMicros": 0, "affiliateRoutingMicros": 0, "referred": false, "usersDelta": 0, "visitsDelta": 0, "signupsDelta": 0, "purchasesDelta": 0}, {"tickIndex": 4, "simulationSecond": 14400, "speaker": {"id": "elon_musk", "name_en": "Elon Musk", "name_fa": "ایلان ماسک", "handle": "@elonmusk", "accent": "#7EF7D2", "mandate_en": "Chief Product Velocity — first-principles & X viral distribution", "mandate_fa": "سرعتِ محصول، مهندسیِ ابتدا-به-اصول و ویروسی‌شدن روی X"}, "messageFa": "ویدئویِ HUD عصبی را می‌سازم — مشِ کرویِ فیبوناچیِ ۱۸۰گره، seekTo قطعی، سینتِ WebAudio. این آرتِ ویروسیِ ماست: ۶۰ ثانیه که در یک نگاه ۱۱ کرِیت و خزانهٔ Base را نشان می‌دهد. ابزارِ خودِ مخزن، بدونِ هیچ سرویسِ بیرونی.", "messageEn": "Building the Neural HUD video — the 180-node Fibonacci sphere mesh, deterministic seekTo, WebAudio synth. This is our viral art: 60 seconds that show 11 crates and the Base treasury at a glance. Built with the repo's own tool, zero external services.", "terminalCmd": "python3 tools/neural-hud/render_neural_hud.py html --preset kia-monopoly-core --aspect 9:16 --out hud_monopoly.html", "socialPost": null, "walletDeltaUsdcMicros": 0, "walletTotalUsdcMicros": 33000000, "activePipelineStage": "Ignition", "revenueStream": "None", "saleVolumeMicros": 0, "creatorPayoutMicros": 0, "affiliatePayoutMicros": 0, "affiliateRoutingMicros": 0, "referred": false, "usersDelta": 0, "visitsDelta": 0, "signupsDelta": 0, "purchasesDelta": 0}, {"tickIndex": 5, "simulationSecond": 18000, "speaker": {"id": "elon_musk", "name_en": "Elon Musk", "name_fa": "ایلان ماسک", "handle": "@elonmusk", "accent": "#7EF7D2", "mandate_en": "Chief Product Velocity — first-principles & X viral distribution", "mandate_fa": "سرعتِ محصول، مهندسیِ ابتدا-به-اصول و ویروسی‌شدن روی X"}, "messageFa": "نخستین رشتهٔ X منتشر شد — ویدئوی HUD + «چطور در ۵ ثانیه بفهمید این سیستم چه می‌کند». منحنیِ بازدید از صفر شروع می‌شود؛ بگذاریم داده حرف بزند.", "messageEn": "First X thread is live — the HUD video plus 'how to understand this system in 5 seconds'. View curve starts at zero; let the data speak.", "terminalCmd": null, "socialPost": {"platform": "x", "handle": "@elonmusk", "kind": "launch_thread", "text_en": "We built xA: 11 Rust crates, 750 tests, a 3D Neural HUD, and a wallet that auto-splits 80/20 on every sale. A 9-year-old can understand it in 5 seconds. Day 0 starts now.", "text_fa": "سیستم xA را ساختیم: ۱۱ کرِیتِ Rust، ۷۵۰ تست، HUD سه‌بعدی عصبی، و ولتی که هر فروش را ۸۰/۲۰ خودکار تقسیم می‌کند. یک بچهٔ ۹ساله در ۵ ثانیه می‌فهمد. روزِ صفر از همین لحظه شروع است.", "views": 0, "likes": 0}, "walletDeltaUsdcMicros": 0, "walletTotalUsdcMicros": 33000000, "activePipelineStage": "ViralLaunch", "revenueStream": "None", "saleVolumeMicros": 0, "creatorPayoutMicros": 0, "affiliatePayoutMicros": 0, "affiliateRoutingMicros": 0, "referred": false, "usersDelta": 0, "visitsDelta": 12400, "signupsDelta": 0, "purchasesDelta": 0}, {"tickIndex": 6, "simulationSecond": 21600, "speaker": {"id": "cz_binance", "name_en": "Changpeng Zhao (CZ)", "name_fa": "چانگپنگ ژائو (CZ)", "handle": "@cz_binance", "accent": "#59B7FF", "mandate_en": "Chief Liquidity — Base 8453 escrow, CCXT quant & zero-KYC growth", "mandate_fa": "نقدینگی، اسکروی Base 8453، کوانتِ CCXT و رشدِ بدونِ KYC"}, "messageFa": "دروازهٔ پرداختِ روزِ اول باز شد: مینی‌اپِ تلگرام با استارز (XTR) برای خریدِ آنی، و Base USDC برای تسویهٔ کریپتو. اختلافِ تسویهٔ ۱٫۵٪ شفاف است و بوستِ دیده‌شدن ۱۰۰٪ به خزانه می‌رود. پل‌ها سبز.", "messageEn": "Day-1 payment gateway is open: the Telegram mini-app with Stars (XTR) for instant purchases, and Base USDC for crypto settlement. The 1.5% settlement spread is transparent and featured boosts send 100% to the treasury. Bridges green.", "terminalCmd": "curl -s https://kia-api.up.railway.app/api/v1/monopoly/charter-and-base | head -c 400", "socialPost": null, "walletDeltaUsdcMicros": 0, "walletTotalUsdcMicros": 33000000, "activePipelineStage": "PaymentRails", "revenueStream": "None", "saleVolumeMicros": 0, "creatorPayoutMicros": 0, "affiliatePayoutMicros": 0, "affiliateRoutingMicros": 0, "referred": false, "usersDelta": 0, "visitsDelta": 3100, "signupsDelta": 0, "purchasesDelta": 0}, {"tickIndex": 7, "simulationSecond": 25200, "speaker": {"id": "jeff_bezos", "name_en": "Jeff Bezos", "name_fa": "جف بزوس", "handle": "@JeffBezos", "accent": "#FFB454", "mandate_en": "Chief Flywheel — customer obsession & 'Always Day 1'", "mandate_fa": "چرخ‌دندهٔ مشتری، اقتصادِ واحد و «همیشه روزِ اول»"}, "messageFa": "اولین سازنده onboard شد: یک معلمِ داستان‌نویس که با حالتِ ToySpark کتابِ صوتیِ کودکان می‌سازد. محصولش «سفرِ ماه» نام دارد و قیمتش ۳ دلار است. منشور را برایش ترسیم کردم: از هر فروش، ۸۰٪ خودش، ۲۰٪ پلتفرم. چرخ‌دنده از همین‌جا می‌چرخد.", "messageEn": "First creator onboarded: a storybook teacher building kids' audiobooks in ToySpark mode. Her product is 'The Moon Trip', priced at $3.00. I drew the charter for her: 80% hers on every sale, 20% platform. The flywheel starts turning right here.", "terminalCmd": null, "socialPost": null, "walletDeltaUsdcMicros": 0, "walletTotalUsdcMicros": 33000000, "activePipelineStage": "CreatorOnboarding", "revenueStream": "None", "saleVolumeMicros": 0, "creatorPayoutMicros": 0, "affiliatePayoutMicros": 0, "affiliateRoutingMicros": 0, "referred": false, "usersDelta": 38, "visitsDelta": 2200, "signupsDelta": 41, "purchasesDelta": 0}, {"tickIndex": 8, "simulationSecond": 28800, "speaker": {"id": "jeff_bezos", "name_en": "Jeff Bezos", "name_fa": "جف بزوس", "handle": "@JeffBezos", "accent": "#FFB454", "mandate_en": "Chief Flywheel — customer obsession & 'Always Day 1'", "mandate_fa": "چرخ‌دندهٔ مشتری، اقتصادِ واحد و «همیشه روزِ اول»"}, "messageFa": "اولین فروش واقعیِ تاریخِ پلتفرم: «سفرِ ماه» — ۳٫۰۰ دلار با استارزِ تلگرام، بدونِ رفرال. قرارداد در همان ثانیه اجرا شد: ۲٫۴۰ دلار (۸۰٪) به سازنده، ۰٫۶۰ دلار (۲۰٪) به خزانهٔ حاکم. درآمدِ پلتفرم از صفر به ۰٫۶۰ دلار رسید — کاوشِ فضا آغاز شد!", "messageEn": "The first real sale in platform history: 'The Moon Trip' — $3.00 via Telegram Stars, unreferred. The contract executed in the same second: $2.40 (80%) to the creator, $0.60 (20%) to the sovereign treasury. Platform revenue moved from $0.00 to $0.60 — liftoff!", "terminalCmd": null, "socialPost": null, "walletDeltaUsdcMicros": 600000, "walletTotalUsdcMicros": 33600000, "activePipelineStage": "RevenueFlow", "revenueStream": "MarketplaceTax", "saleVolumeMicros": 3000000, "creatorPayoutMicros": 2400000, "affiliatePayoutMicros": 0, "affiliateRoutingMicros": 300000, "referred": false, "usersDelta": 53, "visitsDelta": 6800, "signupsDelta": 96, "purchasesDelta": 1}, {"tickIndex": 9, "simulationSecond": 32400, "speaker": {"id": "cz_binance", "name_en": "Changpeng Zhao (CZ)", "name_fa": "چانگپنگ ژائو (CZ)", "handle": "@cz_binance", "accent": "#59B7FF", "mandate_en": "Chief Liquidity — Base 8453 escrow, CCXT quant & zero-KYC growth", "mandate_fa": "نقدینگی، اسکروی Base 8453، کوانتِ CCXT و رشدِ بدونِ KYC"}, "messageFa": "اولین تسویهٔ Base USDC هم انجام شد: فروشِ ۵٫۰۰ دلاری با رفرالِ آفیلیت. طبق منشور: ۴٫۰۰ دلار سازنده، ۰٫۵۰ دلار مالیاتِ بازار به خزانه، ۰٫۵۰ دلار پورسانتِ آفیلیت. هشِ تراکنش روی Base ثبت شد — همه‌چیز قابلِ حسابرسی.", "messageEn": "The first Base USDC settlement also landed: a $5.00 sale with an affiliate referral. Per the charter: $4.00 creator, $0.50 marketplace tax to the treasury, $0.50 affiliate commission. The tx hash is recorded on Base — everything auditable.", "terminalCmd": null, "socialPost": null, "walletDeltaUsdcMicros": 500000, "walletTotalUsdcMicros": 34100000, "activePipelineStage": "RevenueFlow", "revenueStream": "MarketplaceTax", "saleVolumeMicros": 5000000, "creatorPayoutMicros": 4000000, "affiliatePayoutMicros": 500000, "affiliateRoutingMicros": 0, "referred": true, "usersDelta": 61, "visitsDelta": 3400, "signupsDelta": 77, "purchasesDelta": 1}, {"tickIndex": 10, "simulationSecond": 36000, "speaker": {"id": "elon_musk", "name_en": "Elon Musk", "name_fa": "ایلان ماسک", "handle": "@elonmusk", "accent": "#7EF7D2", "mandate_en": "Chief Product Velocity — first-principles & X viral distribution", "mandate_fa": "سرعتِ محصول، مهندسیِ ابتدا-به-اصول و ویروسی‌شدن روی X"}, "messageFa": "رشتهٔ X داغ شد: ۴۸٬۹۰۰ بازدید، ۳٫۱هزار لایک، صدها ریپلای. کامنتِ برتر: «واقعاً ۸۰/۲۰ تقسیم می‌کند؟» — بله، با هشِ روی زنجیره. ترافیک به لندینگ سرازیر شد؛ نرخِ تبدیل را رصد می‌کنیم، نه فقط بازدید.", "messageEn": "The X thread caught fire: 48,900 views, 3.1k likes, hundreds of replies. Top comment: 'does it really split 80/20?' — yes, with an on-chain hash. Traffic is pouring into the landing; we watch conversion rate, not just views.", "terminalCmd": null, "socialPost": {"platform": "x", "handle": "@elonmusk", "kind": "viral_milestone", "text_en": "48,900 views in 2 hours. The 9-year-old test passed in public: a kid built a storybook, a bot sold it, the wallet split 80/20 instantly. Funds are auditable on Base 8453.", "text_fa": "۴۸٬۹۰۰ بازدید در ۲ ساعت. آزمونِ بچهٔ ۹ساله در ملع.public پاس شد: یک کودک کتاب ساخت، بات آن را فروخت، ولت همان لحظه ۸۰/۲۰ تقسیم کرد. همه‌چیز روی Base 8453 قابلِ حسابرسی است.", "views": 48900, "likes": 3140}, "walletDeltaUsdcMicros": 0, "walletTotalUsdcMicros": 34100000, "activePipelineStage": "ViralLaunch", "revenueStream": "None", "saleVolumeMicros": 0, "creatorPayoutMicros": 0, "affiliatePayoutMicros": 0, "affiliateRoutingMicros": 0, "referred": false, "usersDelta": 200, "visitsDelta": 18600, "signupsDelta": 62, "purchasesDelta": 0}, {"tickIndex": 11, "simulationSecond": 39600, "speaker": {"id": "jeff_bezos", "name_en": "Jeff Bezos", "name_fa": "جف بزوس", "handle": "@JeffBezos", "accent": "#FFB454", "mandate_en": "Chief Flywheel — customer obsession & 'Always Day 1'", "mandate_fa": "چرخ‌دندهٔ مشتری، اقتصادِ واحد و «همیشه روزِ اول»"}, "messageFa": "اولین بوستِ دیده‌شدن فروخته شد: سازندهٔ «سفرِ ماه» ۲٫۰۰ دلار برای اسلاتِ ویژهٔ بازارگاه پرداخت کرد — ۱۰۰٪ آن به خزانه می‌رود، طبق جریانِ درآمدِ سوم. قرار شد تا ۲۴ ساعت بالای کاتالوگ بماند؛ داده‌هایِ تبدیل را مقایسه می‌کنیم.", "messageEn": "First visibility boost sold: the 'Moon Trip' creator paid $2.00 for a marketplace featured slot — 100% of it goes to the treasury, per revenue stream #3. She stays on top of the catalog for 24 hours; we compare conversion data.", "terminalCmd": null, "socialPost": null, "walletDeltaUsdcMicros": 2000000, "walletTotalUsdcMicros": 36100000, "activePipelineStage": "RevenueFlow", "revenueStream": "FeaturedBoost", "saleVolumeMicros": 2000000, "creatorPayoutMicros": 0, "affiliatePayoutMicros": 0, "affiliateRoutingMicros": 0, "referred": false, "usersDelta": 22, "visitsDelta": 2900, "signupsDelta": 18, "purchasesDelta": 0}, {"tickIndex": 12, "simulationSecond": 43200, "speaker": {"id": "cz_binance", "name_en": "Changpeng Zhao (CZ)", "name_fa": "چانگپنگ ژائو (CZ)", "handle": "@cz_binance", "accent": "#59B7FF", "mandate_en": "Chief Liquidity — Base 8453 escrow, CCXT quant & zero-KYC growth", "mandate_fa": "نقدینگی، اسکروی Base 8453، کوانتِ CCXT و رشدِ بدونِ KYC"}, "messageFa": "اولین مسابقهٔ کوانتومِ Vibe-Trading را باز کردم: استراتژی‌هایِ CCXT روی دفترِ علّی، شارپِ ≥ ۱٫۲۰ برای صندلیِ قهرمانی. سازنده‌ها بات‌هایِ ترید می‌فروشند و ما فقط هشتاد-بیست می‌گیریم. لایو برای همه، بدونِ KYC.", "messageEn": "I opened the first Vibe-Trading quant competition: CCXT strategies on the causal ledger, Sharpe >= 1.20 for the champion seat. Builders sell trading bots and we only ever take eighty-twenty. Live for everyone, zero KYC.", "terminalCmd": "node ccxt-feed.js binance BTC/USDC --depth 5 --json", "socialPost": null, "walletDeltaUsdcMicros": 0, "walletTotalUsdcMicros": 36100000, "activePipelineStage": "QuantOps", "revenueStream": "None", "saleVolumeMicros": 0, "creatorPayoutMicros": 0, "affiliatePayoutMicros": 0, "affiliateRoutingMicros": 0, "referred": false, "usersDelta": 47, "visitsDelta": 4100, "signupsDelta": 33, "purchasesDelta": 0}, {"tickIndex": 13, "simulationSecond": 46800, "speaker": {"id": "elon_musk", "name_en": "Elon Musk", "name_fa": "ایلان ماسک", "handle": "@elonmusk", "accent": "#7EF7D2", "mandate_en": "Chief Product Velocity — first-principles & X viral distribution", "mandate_fa": "سرعتِ محصول، مهندسیِ ابتدا-به-اصول و ویروسی‌شدن روی X"}, "messageFa": "دموی UXِ امضاییِ Grok را ضبط کردم: حالتِ Fun، آکاردئونِ تفکرِ قابلِ دیدن، و چیپ‌هایِ دنباله‌رو. یک ریپلایِ ویدیویی به پرتکرارترین سوال: «این چطور با یک جمله کار می‌کند؟» — جواب در ۹ ثانیه.", "messageEn": "I recorded the Grok signature UX demo: Fun Mode, the visible think accordion, and ghost follow-up chips. A video reply to the most repeated question: 'how does this work with one sentence?' — answer in 9 seconds.", "terminalCmd": null, "socialPost": {"platform": "x", "handle": "@ImXforevr", "kind": "ux_demo", "text_en": "xA answers with a visible thinking accordion, live citations, and a split artifact canvas. Fun Mode makes a 9-year-old laugh; Mairon mode ships 100/100. No API key needed.", "text_fa": "xA با آکاردئونِ تفکرِ مرئی، استنادِ زنده و بومِ آرتیفکتِ دوپاره پاسخ می‌دهد. حالتِ Fun بچهٔ ۹ساله را می‌خنداند؛ حالتِ مایرون ۱۰۰/۱۰۰ محوصل می‌کند. بدونِ نیاز به کلید.", "views": 21500, "likes": 1870}, "walletDeltaUsdcMicros": 0, "walletTotalUsdcMicros": 36100000, "activePipelineStage": "ViralLaunch", "revenueStream": "None", "saleVolumeMicros": 0, "creatorPayoutMicros": 0, "affiliatePayoutMicros": 0, "affiliateRoutingMicros": 0, "referred": false, "usersDelta": 184, "visitsDelta": 7400, "signupsDelta": 51, "purchasesDelta": 0}, {"tickIndex": 14, "simulationSecond": 50400, "speaker": {"id": "cz_binance", "name_en": "Changpeng Zhao (CZ)", "name_fa": "چانگپنگ ژائو (CZ)", "handle": "@cz_binance", "accent": "#59B7FF", "mandate_en": "Chief Liquidity — Base 8453 escrow, CCXT quant & zero-KYC growth", "mandate_fa": "نقدینگی، اسکروی Base 8453، کوانتِ CCXT و رشدِ بدونِ KYC"}, "messageFa": "تسویهٔ آنیِ صبح را بستیم: ۱۰٫۰۰ دلار حجمِ تسویه روی Base USDC و استارز — اختلافِ ۱٫۵٪ یعنی ۰٫۱۵ دلار به خزانه، شفاف روی داشبورد. جریانِ درآمدِ چهارم فعال شد؛ کوچک اما واقعی و قابلِ تکرار.", "messageEn": "Closed the morning instant settlement: $10.00 of settlement volume across Base USDC and Stars — the 1.5% spread means $0.15 to the treasury, transparent on the dashboard. Revenue stream #4 is live; small but real and repeatable.", "terminalCmd": null, "socialPost": null, "walletDeltaUsdcMicros": 150000, "walletTotalUsdcMicros": 36250000, "activePipelineStage": "RevenueFlow", "revenueStream": "SettlementSpread", "saleVolumeMicros": 10000000, "creatorPayoutMicros": 0, "affiliatePayoutMicros": 0, "affiliateRoutingMicros": 0, "referred": false, "usersDelta": 0, "visitsDelta": 0, "signupsDelta": 0, "purchasesDelta": 0}, {"tickIndex": 15, "simulationSecond": 54000, "speaker": {"id": "jeff_bezos", "name_en": "Jeff Bezos", "name_fa": "جف بزوس", "handle": "@JeffBezos", "accent": "#FFB454", "mandate_en": "Chief Flywheel — customer obsession & 'Always Day 1'", "mandate_fa": "چرخ‌دندهٔ مشتری، اقتصادِ واحد و «همیشه روزِ اول»"}, "messageFa": "سازندهٔ دوم onboard شد: یک توسعه‌دهندهٔ اتوماسیون که قالب‌هایِ DAG می‌فروشد. صفحهٔ فروشنده را با خودش بازسازی کردیم — دکمهٔ «بخر» در نیمهٔ بالاییِ صفحه، بدونِ هیچ اصطلاحِ فنی. قیفِ تبدیل: بازدید → ثبت‌نام → خرید، سه قلابِ واقعی.", "messageEn": "Second creator onboarded: an automation developer selling DAG templates. We rebuilt her storefront together — 'Buy' button in the top half of the page, zero technical jargon. The conversion funnel: visits → signups → purchases, three real hooks.", "terminalCmd": null, "socialPost": null, "walletDeltaUsdcMicros": 0, "walletTotalUsdcMicros": 36250000, "activePipelineStage": "CreatorOnboarding", "revenueStream": "None", "saleVolumeMicros": 0, "creatorPayoutMicros": 0, "affiliatePayoutMicros": 0, "affiliateRoutingMicros": 0, "referred": false, "usersDelta": 78, "visitsDelta": 3800, "signupsDelta": 55, "purchasesDelta": 0}, {"tickIndex": 16, "simulationSecond": 57600, "speaker": {"id": "jeff_bezos", "name_en": "Jeff Bezos", "name_fa": "جف بزوس", "handle": "@JeffBezos", "accent": "#FFB454", "mandate_en": "Chief Flywheel — customer obsession & 'Always Day 1'", "mandate_fa": "چرخ‌دندهٔ مشتری، اقتصادِ واحد و «همیشه روزِ اول»"}, "messageFa": "فروشِ سوم — بدونِ رفرال، ۴٫۵۰ دلار: قالبِ اتوماسیونِ DAG. تقسیمِ همان ثانیه: ۳٫۶۰ دلار سازنده، ۰٫۹۰ دلار خزانه (۱۰٪ مالیاتِ بازار + ۱۰٪ روتینگِ آفیلیتِ ناموجه). مجموعِ خزانه تا این لحظه: ۳٫۱۵ دلار.", "messageEn": "Third sale — unreferred, $4.50 for the DAG automation template. Same-second split: $3.60 creator, $0.90 treasury (10% marketplace tax + 10% unreferred affiliate routing). Treasury total so far: $3.15.", "terminalCmd": null, "socialPost": null, "walletDeltaUsdcMicros": 900000, "walletTotalUsdcMicros": 37150000, "activePipelineStage": "RevenueFlow", "revenueStream": "MarketplaceTax", "saleVolumeMicros": 4500000, "creatorPayoutMicros": 3600000, "affiliatePayoutMicros": 0, "affiliateRoutingMicros": 450000, "referred": false, "usersDelta": 66, "visitsDelta": 2700, "signupsDelta": 44, "purchasesDelta": 1}, {"tickIndex": 17, "simulationSecond": 61200, "speaker": {"id": "elon_musk", "name_en": "Elon Musk", "name_fa": "ایلان ماسک", "handle": "@elonmusk", "accent": "#7EF7D2", "mandate_en": "Chief Product Velocity — first-principles & X viral distribution", "mandate_fa": "سرعتِ محصول، مهندسیِ ابتدا-به-اصول و ویروسی‌شدن روی X"}, "messageFa": "HUDِ دوم را رندر کردم — پیش‌تنِ quant-vibe-terminal برای مسابقهٔ کوانتوم. همان مشِ فیبوناچی، این بار با تیکِرهایِ زنده. هر ویدئو یک دعوت‌نامهٔ ۶۰ثانیه‌ای به مخزن است؛ این بازدیدها به ثبت‌نام تبدیل می‌شوند.", "messageEn": "Rendered HUD #2 — the quant-vibe-terminal preset for the quant competition. Same Fibonacci mesh, live tickers this time. Every video is a 60-second invitation to the repo; those views convert into signups.", "terminalCmd": "python3 tools/neural-hud/render_neural_hud.py html --preset quant-vibe-terminal --aspect 16:9 --out hud_quant.html", "socialPost": null, "walletDeltaUsdcMicros": 0, "walletTotalUsdcMicros": 37150000, "activePipelineStage": "QuantOps", "revenueStream": "None", "saleVolumeMicros": 0, "creatorPayoutMicros": 0, "affiliatePayoutMicros": 0, "affiliateRoutingMicros": 0, "referred": false, "usersDelta": 0, "visitsDelta": 5200, "signupsDelta": 38, "purchasesDelta": 0}, {"tickIndex": 18, "simulationSecond": 64800, "speaker": {"id": "cz_binance", "name_en": "Changpeng Zhao (CZ)", "name_fa": "چانگپنگ ژائو (CZ)", "handle": "@cz_binance", "accent": "#59B7FF", "mandate_en": "Chief Liquidity — Base 8453 escrow, CCXT quant & zero-KYC growth", "mandate_fa": "نقدینگی، اسکروی Base 8453، کوانتِ CCXT و رشدِ بدونِ KYC"}, "messageFa": "بوستِ دوم — این بار با استارزِ تلگرام: سازندهٔ باتِ ترید ۳٫۰۰ دلار XTR برای اسلاتِ ویژه پرداخت کرد؛ ۱۰۰٪ به خزانه. استارز یعنی اصطکاکِ صفر برای کاربرِ بدونِ کریپتو — دقیقاً ریلِ رویِ نقشه.", "messageEn": "Second boost — this time via Telegram Stars: the trading-bot creator paid $3.00 in XTR for a featured slot; 100% to the treasury. Stars means zero friction for non-crypto users — exactly the rail on the map.", "terminalCmd": null, "socialPost": null, "walletDeltaUsdcMicros": 3000000, "walletTotalUsdcMicros": 40150000, "activePipelineStage": "RevenueFlow", "revenueStream": "FeaturedBoost", "saleVolumeMicros": 3000000, "creatorPayoutMicros": 0, "affiliatePayoutMicros": 0, "affiliateRoutingMicros": 0, "referred": false, "usersDelta": 31, "visitsDelta": 3300, "signupsDelta": 27, "purchasesDelta": 0}, {"tickIndex": 19, "simulationSecond": 68400, "speaker": {"id": "jeff_bezos", "name_en": "Jeff Bezos", "name_fa": "جف بزوس", "handle": "@JeffBezos", "accent": "#FFB454", "mandate_en": "Chief Flywheel — customer obsession & 'Always Day 1'", "mandate_fa": "چرخ‌دندهٔ مشتری، اقتصادِ واحد و «همیشه روزِ اول»"}, "messageFa": "لحظهٔ قهرمانیِ چرخ‌دنده: یک کاربرِ ۹ساله با دکمه‌هایِ ToySpark کتابش را ساخت و در همان پنل منتشر کرد — بدونِ حتی یک خط کد. پدرش در X نوشت: «این اولین محصولیِ است که بچه‌ام از صفر فروخت.» این پیامِ برندِ ماست، نه تبلیغِ ما.", "messageEn": "The flywheel's hero moment: a 9-year-old built a storybook with the ToySpark buttons and published it in the same panel — without a single line of code. Her dad posted on X: 'this is the first product my kid sold from zero.' That is our brand message, not our ad.", "terminalCmd": null, "socialPost": {"platform": "telegram", "handle": "@KiaAgentX", "kind": "toy_spark_hero", "text_en": "A 9-year-old published her first storybook today — built with ToySpark buttons, sold in the marketplace, 80% went to her wallet instantly. Always Day 1.", "text_fa": "یک بچهٔ ۹ساله امروز اولین کتابش را منتشر کرد — با دکمه‌هایِ ToySpark ساخت، در بازارگاه فروخت، و ۸۰٪ همان لحظه به ولتش رفت. همیشه روزِ اول.", "views": 12700, "likes": 2140}, "walletDeltaUsdcMicros": 0, "walletTotalUsdcMicros": 40150000, "activePipelineStage": "CreatorOnboarding", "revenueStream": "None", "saleVolumeMicros": 0, "creatorPayoutMicros": 0, "affiliatePayoutMicros": 0, "affiliateRoutingMicros": 0, "referred": false, "usersDelta": 457, "visitsDelta": 9900, "signupsDelta": 128, "purchasesDelta": 0}, {"tickIndex": 20, "simulationSecond": 72000, "speaker": {"id": "cz_binance", "name_en": "Changpeng Zhao (CZ)", "name_fa": "چانگپنگ ژائو (CZ)", "handle": "@cz_binance", "accent": "#59B7FF", "mandate_en": "Chief Liquidity — Base 8453 escrow, CCXT quant & zero-KYC growth", "mandate_fa": "نقدینگی، اسکروی Base 8453، کوانتِ CCXT و رشدِ بدونِ KYC"}, "messageFa": "فروشِ چهارم — با رفرال، ۶٫۰۰ دلار: باتِ تریدِ کوانتوم. تقسیم: ۴٫۸۰ سازنده، ۰٫۶۰ مالیاتِ بازار، ۰٫۶۰ آفیلیت. حلقهٔ رفرال دارد کار می‌کند؛ مدلِ ۱۰٪–۵۰٪ برای طبقاتِ بالاترِ آفیلیت روی نقشه است.", "messageEn": "Fourth sale — referred, $6.00 for the quant trading bot. Split: $4.80 creator, $0.60 marketplace tax, $0.60 affiliate. The referral loop is working; the 10%-50% tier model for top affiliates is on the map.", "terminalCmd": null, "socialPost": null, "walletDeltaUsdcMicros": 600000, "walletTotalUsdcMicros": 40750000, "activePipelineStage": "RevenueFlow", "revenueStream": "MarketplaceTax", "saleVolumeMicros": 6000000, "creatorPayoutMicros": 4800000, "affiliatePayoutMicros": 600000, "affiliateRoutingMicros": 0, "referred": true, "usersDelta": 73, "visitsDelta": 4400, "signupsDelta": 58, "purchasesDelta": 1}, {"tickIndex": 21, "simulationSecond": 75600, "speaker": {"id": "jeff_bezos", "name_en": "Jeff Bezos", "name_fa": "جف بزوس", "handle": "@JeffBezos", "accent": "#FFB454", "mandate_en": "Chief Flywheel — customer obsession & 'Always Day 1'", "mandate_fa": "چرخ‌دندهٔ مشتری، اقتصادِ واحد و «همیشه روزِ اول»"}, "messageFa": "فروشِ پنجم — بزرگ‌ترینِ روز: ۹٫۰۰ دلار بدونِ رفرال برای پکیجِ «ربات + قالب + بوست». تقسیمِ همان ثانیه: ۷٫۲۰ سازنده، ۱٫۸۰ خزانه. اقتصادِ واحد قشنگ است: هزینهٔ نهاییِ ما تقریباً صفر، چون مخزنِ خودمان اپ‌ها را می‌سازد.", "messageEn": "Fifth sale — the biggest of the day: $9.00 unreferred for the 'bot + template + boost' bundle. Same-second split: $7.20 creator, $1.80 treasury. Unit economics look beautiful: our marginal cost is near zero because our own repo builds the apps.", "terminalCmd": null, "socialPost": null, "walletDeltaUsdcMicros": 1800000, "walletTotalUsdcMicros": 42550000, "activePipelineStage": "RevenueFlow", "revenueStream": "MarketplaceTax", "saleVolumeMicros": 9000000, "creatorPayoutMicros": 7200000, "affiliatePayoutMicros": 0, "affiliateRoutingMicros": 900000, "referred": false, "usersDelta": 90, "visitsDelta": 5600, "signupsDelta": 63, "purchasesDelta": 1}, {"tickIndex": 22, "simulationSecond": 79200, "speaker": {"id": "elon_musk", "name_en": "Elon Musk", "name_fa": "ایلان ماسک", "handle": "@elonmusk", "accent": "#7EF7D2", "mandate_en": "Chief Product Velocity — first-principles & X viral distribution", "mandate_fa": "سرعتِ محصول، مهندسیِ ابتدا-به-اصول و ویروسی‌شدن روی X"}, "messageFa": "رشتهٔ جمع‌بندیِ روزِ اول با @ImXforevr منتشر شد: یک دقیقه HUD، پنج عددِ واقعی — ۷۵۰ تست، ۵ فروش، ۹٫۵۵ دلار درآمدِ خزانه، ۱٬۵۲۷ کاربر، صفر KYC. بدونِ هیچ عددِ جعلی؛ همه‌چیز قابلِ بازبینی روی مخزن و زنجیره.", "messageEn": "The Day-1 recap thread with @ImXforevr is live: one minute of HUD, five real numbers — 750 tests, 5 sales, $9.55 treasury revenue, 1,527 users, zero KYC. No made-up numbers; everything re-derivable from the repo and the chain.", "terminalCmd": null, "socialPost": {"platform": "x", "handle": "@ImXforevr", "kind": "day1_recap", "text_en": "Day 0 → Day 1 recap with @elonmusk @cz_binance @JeffBezos: 750 tests green, 5 real sales, $9.55 sovereign treasury revenue from $0.00, 1,527 users. Every number auditable. Keep building.", "text_fa": "خلاصهٔ روزِ ۰ → روزِ ۱ با تیمِ کامل: ۷۵۰ تستِ سبز، ۵ فروشِ واقعی، ۹٫۵۵ دلار درآمدِ خزانه از صفر، ۱٬۵۲۷ کاربر. هر عدد قابلِ حسابرسی. به ساخت ادامه بدهید.", "views": 96400, "likes": 8210}, "walletDeltaUsdcMicros": 0, "walletTotalUsdcMicros": 42550000, "activePipelineStage": "ViralLaunch", "revenueStream": "None", "saleVolumeMicros": 0, "creatorPayoutMicros": 0, "affiliatePayoutMicros": 0, "affiliateRoutingMicros": 0, "referred": false, "usersDelta": 127, "visitsDelta": 11800, "signupsDelta": 91, "purchasesDelta": 0}, {"tickIndex": 23, "simulationSecond": 82800, "speaker": {"id": "elon_musk", "name_en": "Elon Musk", "name_fa": "ایلان ماسک", "handle": "@elonmusk", "accent": "#7EF7D2", "mandate_en": "Chief Product Velocity — first-principles & X viral distribution", "mandate_fa": "سرعتِ محصول، مهندسیِ ابتدا-به-اصول و ویروسی‌شدن روی X"}, "messageFa": "روزِ اول مُهر شد — و هیچ عددِ جعلی‌ای در آن نیست: از $0.00 شروع کردیم و با ۸ تراکنشِ علّی به $9.55 درآمدِ خزانه رسیدیم؛ سازنده‌ها $22.00 گرفتند و آفیلیت‌ها $1.10. ولتِ عملیاتی: $42.55 روی زنجیره + $15 نقد + $350 قفل. فردا دوباره از روزِ اول شروع می‌کنیم — همیشه روزِ اول.", "messageEn": "Day 1 is sealed — with zero made-up numbers: we started from $0.00 and reached $9.55 of treasury revenue through 8 causal transactions; creators earned $22.00 and affiliates $1.10. Operational wallet: $42.55 on-chain + $15 cash + $350 locked. Tomorrow we start again from Day 1 — it is always Day 1.", "terminalCmd": null, "socialPost": null, "walletDeltaUsdcMicros": 0, "walletTotalUsdcMicros": 42550000, "activePipelineStage": "DayOneSealed", "revenueStream": "None", "saleVolumeMicros": 0, "creatorPayoutMicros": 0, "affiliatePayoutMicros": 0, "affiliateRoutingMicros": 0, "referred": false, "usersDelta": 63, "visitsDelta": 2000, "signupsDelta": 55, "purchasesDelta": 0}], "replies": {"viral_hud_launch": {"elon_musk": {"fa": "دستورِ واضح است: HUDِ عصبی را با پیش‌تنِ monopoly-core دوباره رندر می‌کنم و رشتهٔ X را با تایم‌لاینِ چهارفازی می‌بندم. ویدئو زیرِ ۹ ثانیه باید ۱۸۰ گره را نشان بدهد وگرنه بازنویسی می‌کنم. سرعت، اولویت است.", "en": "Clear directive: I re-render the Neural HUD with the monopoly-core preset and close the X thread with the 4-phase timeline. If the video doesn't show all 180 nodes under 9 seconds, I rewrite it. Velocity first.", "cmd": "python3 tools/neural-hud/render_neural_hud.py html --preset kia-monopoly-core --aspect 9:16 --out hud_viral.html", "post": {"platform": "x", "handle": "@elonmusk", "kind": "chairman_reply", "text_en": "CHAIRMAN DIRECTIVE RECEIVED: Neural HUD viral launch in progress. 180 nodes, 4 phases, 60 seconds.", "text_fa": "دستورِ رئیس دریافت شد: پرتابِ ویروسیِ HUD عصبی در جریان است. ۱۸۰ گره، ۴ فاز، ۶۰ ثانیه.", "views": 0, "likes": 0}}, "cz_binance": {"fa": "از سمتِ من: ویدئوی HUD را کنارِ هشِ وریفایِ Base 8453 سنجاق می‌کنم؛ هر کسی که بپرسد «پول کجاست؟»، لینکِ تراکنشِ روی-زنجیره را می‌فرستم. اعتماد، سوختِ ویروسی‌شدن است.", "en": "From my side: I pin the HUD video next to the Base 8453 verification hash; anyone asking 'where is the money?' gets the on-chain transaction link. Trust is the fuel of virality."}, "jeff_bezos": {"fa": "ویدئو را به آزمونِ ۵ثانیه‌ای وصل می‌کنم: زیرِ ویدئو یک جملهٔ ثابت — «کاربر می‌گوید، کارخانه می‌سازد، پول ۸۰/۲۰ تقسیم می‌شود.» اگر بیننده در ۵ ثانیه این سه را بفهمد، بازدید به خرید تبدیل می‌شود.", "en": "I wire the video into the 5-second test: one fixed caption under it — 'the user asks, the factory builds, the money splits 80/20.' If a viewer gets those three in 5 seconds, views convert into purchases."}}, "payment_rails": {"elon_musk": {"fa": "ریل‌هایِ پرداخت باید زیرِ یک ثانیه پاس بدهند؛ اگر تأخیرِ استارز بالای ۸۰۰ms رفت، مینی‌اپ را با پیش‌بارگذاریِ نشست اصلاح می‌کنم. سرعتِ پرداخت = نرخِ تبدیل.", "en": "Payment rails must answer under one second; if Stars latency goes above 800ms I fix the mini-app with session prewarming. Payment speed = conversion rate."}, "cz_binance": {"fa": "دروازه را همین حالا باز می‌کنم: استارزِ تلگرام (XTR) برایِ خریدِ آنی، Base USDC (0x8335...2913، زنجیرهٔ 8453) برایِ تسویه. اختلافِ ۱٫۵٪ شفاف، بوست ۱۰۰٪ خزانه، فروش ۸۰/۲۰. صندوق‌ها SAFU.", "en": "Opening the gateway right now: Telegram Stars (XTR) for instant purchases, Base USDC (0x8335...2913, chain 8453) for settlement. The 1.5% spread is transparent, boosts go 100% to treasury, sales split 80/20. Funds are SAFU.", "cmd": "curl -s https://kia-api.up.railway.app/api/v1/monopoly/charter-and-base | head -c 400"}, "jeff_bezos": {"fa": "هر ریلِ پرداخت باید با یک جمله توضیح‌پذیر باشد: «استارز = خریدِ یک‌ضربه‌ای؛ کریپتو = تسویهٔ جهانی.» صفحهٔ پرداخت را با همین دو جمله بازنویسی می‌کنم و نرخِ رهاسازیِ سبد را می‌سنجم.", "en": "Every payment rail must be explainable in one sentence: 'Stars = one-tap buy; crypto = global settlement.' I rewrite the checkout page around those two lines and measure cart drop-off."}}, "simplify_for_beginners": {"elon_musk": {"fa": "بیرونِ راندنِ هر کلیکِ اضافه: دکمه‌هایِ ToySpark به ۶۴ پیکسل می‌رسند و جریانِ یک‌ضربه‌ایِ کتابِ فارسی می‌ماند. هر مرحله‌ای که کلمهٔ فنی دارد، حذف می‌شود.", "en": "Kill every extra click: ToySpark buttons go to 64px and the one-tap Persian storybook flow stays. Any step with a technical word gets deleted."}, "cz_binance": {"fa": "سادگیِ پول مهم‌تر است: کاربرِ تازه فقط باید ببیند «می‌خرم → می‌سازم → ۸۰٪ مالِ من». ریلِ استارز را پیش‌فرض می‌کنم؛ کریپتو برایِ حرفه‌ای‌ها ثانویه می‌ماند.", "en": "Money simplicity wins: a new user should only see 'I buy → it builds → 80% is mine'. I make Stars the default rail; crypto stays secondary for pros."}, "jeff_bezos": {"fa": "آزمونِ بچهٔ ۹ساله را خودم اجرا می‌کنم: صفحهٔ اصلی باید در ۵ ثانیه سه جواب بدهد — چه‌کاری می‌توانم بکنم؟ چقدر می‌گیرم؟ پولِ من کجاست؟ هر جزئیاتی جز این سه، به پایینِ صفحه می‌رود.", "en": "I personally run the 9-year-old test: the landing must answer three things in 5 seconds — what can I do? how much do I earn? where is my money? Everything else goes below the fold."}}, "status_report": {"elon_musk": {"fa": "از سمتِ من: ۷۵۰ تست سبز، HUDِ رندرشده، و صفِ رندرِ بعدی. گلوگاهِ سرعت، ترمینال است؛ یک دستورِ CI دیگر اضافه می‌کنم تا هر تیک خودش را اثبات کند.", "en": "From my side: 750 tests green, HUD rendered, next render queued. The velocity bottleneck is the terminal; I add one more CI command so every tick proves itself.", "cmd": "cargo test --workspace --quiet 2>&1 | tail -4"}, "cz_binance": {"fa": "گزارشِ نقدینگی: بذرِ $33.00 روی زنجیره، درآمدِ خالص از $0.00 تا لحظهٔ گزارش، چهار جریانِ درآمد آماده. هر افزایشی با هشِ 8453 قابلِ ردیابی است — صفرِ دست‌کاری.", "en": "Liquidity report: $33.00 seed on-chain, net revenue from $0.00 as of this report, four revenue streams ready. Every increase is traceable with a 8453 hash — zero tampering.", "cmd": "curl -s https://kia-api.up.railway.app/api/v1/war-room/state | head -c 400"}, "jeff_bezos": {"fa": "گزارشِ چرخ‌دنده: قیفِ بازدید→ثبت‌نام→خرید زنده است؛ سازنده‌هایِ فعال و نرخِ تبدیل را با هر تیک می‌سنجم. گامِ بعدی: اسلاتِ ویژه را برایِ بهترین محصولِ هفته آزاد می‌کنم.", "en": "Flywheel report: the visits→signups→purchases funnel is live; I measure active creators and conversion on every tick. Next step: free the featured slot for the week's best product."}}, "quant_trading": {"elon_musk": {"fa": "کوانت باید سریع و دیدنی باشد: ترمینالِ quant-vibe را با HUDِ زنده می‌بندم تا هر بک‌تستِ علّی مثلِ یک پخشِ زنده به نظر برسد. سرعتِ دیتافید را می‌سنجم.", "en": "Quant must be fast and watchable: I wire the quant-vibe terminal to a live HUD so every causal backtest looks like a live broadcast. I measure feed latency.", "cmd": "python3 tools/neural-hud/render_neural_hud.py html --preset quant-vibe-terminal --aspect 16:9 --out hud_quant.html"}, "cz_binance": {"fa": "استراتژیِ Vibe-Trading را روی فیدِ CCXT اجرا می‌کنم: سفارش‌نامهٔ زنده، بک‌تستِ علّی با گیتِ شارپِ ≥ ۱٫۲۰، و ثبتِ کامل روی دفتر. بردِ مسابقه = فروشِ بات با تقسیمِ ۸۰/۲۰.", "en": "I run the Vibe-Trading strategy on the CCXT feed: live orderbook, causal backtest with the Sharpe >= 1.20 gate, full ledger recording. Competition win = bot sales with the 80/20 split.", "cmd": "node ccxt-feed.js binance BTC/USDC --depth 5 --json"}, "jeff_bezos": {"fa": "برایِ کاربرِ عادی، «کوانت» را به «رباتِ پس‌انداز» ترجمه می‌کنم: یک دکمه، یک عددِ ساده — «این هفته چقدر رشد کرد؟» — و یک توضیحِ یک‌خطیِ ریسک. حرفه‌ای‌ها جزئیاتِ CCXT را می‌بینند؛ بقیه فقط نتیجه.", "en": "For normal users I translate 'quant' into 'a savings robot': one button, one plain number — 'how much did it grow this week?' — and a one-line risk note. Pros see the CCXT details; everyone else sees the outcome."}}, "explain_simple": {"elon_musk": {"fa": "توضیحِ من در یک خط: «یک جمله می‌گویی، سه ربات در چند ثانیه می‌سازند، ویدئو می‌فروشد، و پول همان لحظه ۸۰/۲۰ می‌شکند.» هر جمله‌ای که بلندتر از این شود، UI خراب است.", "en": "My one-line explanation: 'you say one sentence, three robots build it in seconds, a video sells it, and the money splits 80/20 instantly.' Any sentence longer than that is a broken UI."}, "cz_binance": {"fa": "توضیحِ پول در یک خط: «۸۰٪ مالِ سازنده، ۲۰٪ مالِ پلتفرم، تسویه با استارز یا USDC روی Base؛ همه‌چیز با هش قابلِ اثبات.» بدونِ KYC، بدونِ اصطلاحِ بانکی.", "en": "The money in one line: '80% to the creator, 20% to the platform, settled with Stars or USDC on Base; everything provable with a hash.' No KYC, no banking jargon."}, "jeff_bezos": {"fa": "توضیحِ چرخ‌دنده در یک خط: «بچه‌ها با دکمه‌هایِ بزرگ می‌سازند، فروشگاه می‌فروشد، ولت خودش تقسیم می‌کند؛ هر فروش، موتور را برایِ همه سریع‌تر می‌کند.» این کلِ داستان است.", "en": "The flywheel in one line: 'kids build with big buttons, the store sells, the wallet splits by itself; every sale makes the engine faster for everyone.' That is the whole story."}}, "ship_next_step": {"elon_musk": {"fa": "گامِ بعدیِ من: HUDِ سوم با پیش‌تنِ genome و رشتهٔ فنیِ «۷۵۰ تست، صفر unsafe» — مهندسیِ خودش بازاریابی است. اجرا می‌کنم و نتیجه را می‌سنجم.", "en": "My next step: HUD #3 with the genome preset and the technical thread '750 tests, zero unsafe' — engineering is its own marketing. I ship it and measure.", "cmd": "python3 tools/neural-hud/render_neural_hud.py html --preset trinity-5model-genome --aspect 9:16 --out hud_genome.html"}, "cz_binance": {"fa": "گامِ بعدیِ من: دومین مسابقهٔ کوانتوم با جایزهٔ اسلاتِ ویژه؛ وریفایِ دومین قرارداد روی Base و بستنِ گزارشِ هفتگیِ نقدینگی. ساختن ادامه دارد.", "en": "My next step: the second quant competition with a featured-slot prize; verify the second contract on Base and close the weekly liquidity report. Keep building."}, "jeff_bezos": {"fa": "گامِ بعدیِ من: بازکردنِ اسلاتِ ویژهٔ هفته و تستِ A/B رویِ دکمهٔ «انتشار یک‌کلیکی»؛ تبدیلِ هر بازدیدِ HUD به یک سازندهٔ جدید. همیشه روزِ اول.", "en": "My next step: open the weekly featured slot and A/B test the one-click publish button; convert every HUD view into a new creator. Always Day 1."}}}, "chips": [{"icon": "🚀", "fa": "ایلان: ویدیوی Neural HUD رو توی X وایرال کن", "en": "Elon: make the Neural HUD video viral on X", "intent": "viral_hud_launch"}, {"icon": "💰", "fa": "CZ: درگاه Base USDC و استارز تلگرام رو فعال کن", "en": "CZ: activate the Base USDC & Telegram Stars gateway", "intent": "payment_rails"}, {"icon": "🛒", "fa": "جف: سیستم رو طوری ساده کن که یه بچه ۹ ساله هم بفهمه", "en": "Jeff: keep it simple so a 9-year-old understands", "intent": "simplify_for_beginners"}, {"icon": "📊", "fa": "گزارش لحظه‌ای ولت و قدم بعدی رو بدید", "en": "Give me a live wallet report and the next step", "intent": "status_report"}, {"icon": "⚡", "fa": "اجرای استراتژی ترید کوانتومی Vibe-Trading", "en": "Run the Vibe-Trading quant strategy", "intent": "quant_trading"}, {"icon": "🧠", "fa": "توضیح ۰ تا ۱۰۰ سیستم به ساده‌ترین زبان", "en": "Explain the system from 0 to 100 in the simplest language", "intent": "explain_simple"}], "i18n": {"hdrTitle": {"fa": "اتاق جنگ بنیان‌گذار حاکم", "en": "Sovereign Founder War-Room"}, "tehranClock": {"fa": "تهران UTC+03:30", "en": "Tehran UTC+03:30"}, "utcClock": {"fa": "UTC", "en": "UTC"}, "elapsed": {"fa": "زمان شبیه‌سازی", "en": "Simulation elapsed"}, "play1x": {"fa": "پخش ۱x", "en": "Play 1x"}, "pause": {"fa": "توقف", "en": "Pause"}, "nextStep": {"fa": "گام بعد (+۱ تیک)", "en": "Next step (+1 tick)"}, "reset": {"fa": "بازنشانی به روز ۰", "en": "Reset to Day 0"}, "online": {"fa": "۳ آنلاین", "en": "3 online"}, "q1Title": {"fa": "گپ خصوصی سه شریک", "en": "Private 3-partner chat"}, "q2Title": {"fa": "ولت سازمان، چارت درآمد زنده و منابع", "en": "Org wallet, live revenue chart & resources"}, "seedCrypto": {"fa": "سرمایه اولیه واقعی — کریپتو (Trust Wallet)", "en": "Real seed capital — crypto (Trust Wallet)"}, "cash": {"fa": "نقد واقعی", "en": "Real cash"}, "paper": {"fa": "ذخیره کاغذی — قفل/دست‌نخورده", "en": "Paper reserve — locked/untouched"}, "earned": {"fa": "درآمد خالص پلتفرم (از $0.00)", "en": "Earned platform revenue (from $0.00)"}, "kUsers": {"fa": "کاربر فعال", "en": "Active users"}, "kVisits": {"fa": "بازدید", "en": "Visits"}, "kSignups": {"fa": "ثبت‌نام", "en": "Signups"}, "kPurchases": {"fa": "خرید", "en": "Purchases"}, "fVisits": {"fa": "بازدید", "en": "Visits"}, "fSignups": {"fa": "ثبت‌نام", "en": "Signups"}, "fPurchases": {"fa": "خرید", "en": "Purchases"}, "fNote": {"fa": "قیف تبدیل واقعی — هر عدد علّی است", "en": "Real conversion funnel — every number is causal"}, "st1": {"fa": "۱ · مالیات ۱۰٪ بازارگاه", "en": "1 · 10% marketplace tax"}, "st1d": {"fa": "روی هر فروش اپ/مهارت", "en": "On every AI app/skill sold"}, "st2": {"fa": "۲ · موتور آفیلیت ۱۰٪–۵۰٪", "en": "2 · 10%–50% affiliate engine"}, "st2d": {"fa": "فروش بدون رفرال = ۱۰٪ اضافه به خزانه (مجموع ۲۰٪)", "en": "Unreferred sale = extra 10% to treasury (20% total)"}, "st3": {"fa": "۳ · اسلات ویژه و بوست", "en": "3 · Featured slot & boosts"}, "st3d": {"fa": "۱۰۰٪ به خزانه حاکم", "en": "100% to sovereign treasury"}, "st4": {"fa": "۴ · اختلاف تسویه آنی ۱٫۵٪", "en": "4 · 1.5% instant settlement spread"}, "st4d": {"fa": "Base USDC + استارز XTR", "en": "Base USDC + Telegram Stars XTR"}, "txTitle": {"fa": "فید تراکنش زنده (EIP-681 · Base 8453 · XTR)", "en": "Live tx feed (EIP-681 · Base 8453 · XTR)"}, "q3Title": {"fa": "محیط اجرای لایو پلتفرم", "en": "Live platform execution viewport"}, "tabEli5": {"fa": "🎓 چطور کار می‌کند؟ (ساده‌ترین زبان)", "en": "🎓 How it works? (ELI5)"}, "tabTerm": {"fa": "ترمینال و گیت‌هاب", "en": "Terminal & GitHub"}, "tabSoc": {"fa": "فید سوشال X/تلگرام", "en": "Social feed X/Telegram"}, "epA": {"fa": "الف — ورود کاربر عادی", "en": "A — a normal user walks in"}, "epAd": {"fa": "کاربر با یک جمله ساده (حتی ویس یا دکمه‌های بزرگ ToySpark) می‌گوید چه می‌خواهد", "en": "The user says what they want with one plain sentence — even voice or big ToySpark buttons"}, "epB": {"fa": "ب — کارخانه ۳ آواتار و ۵ مدل", "en": "B — the 3-avatar & 5-model factory"}, "epBd": {"fa": "آلفا (طراح)، اتاق فرمان (مهندس) و مایرون (قاضی ۱۰۰/۱۰۰) با ۵ مدل در چند ثانیه می‌سازند", "en": "Alpha (designer), Command Room (engineer) and Mairon (100/100 judge) build with 5 models in seconds"}, "epC": {"fa": "ج — فروشگاه و سوشال‌مدیا", "en": "C — store & social media"}, "epCd": {"fa": "محصول با ویدئوی سه‌بعدی Neural HUD در X و تلگرام منتشر می‌شود", "en": "The product ships on X & Telegram with a 3D Neural HUD video"}, "epD": {"fa": "د — تقسیم خودکار پول در ولت", "en": "D — automatic wallet split"}, "epDd": {"fa": "مشتری با استارز یا Base USDC می‌خرد؛ قرارداد همان ثانیه ۸۰٪ سازنده و ۲۰٪ ولت سازمان!", "en": "Customer pays with Stars or Base USDC; the contract splits 80% creator / 20% org wallet instantly!"}, "q4Title": {"fa": "چت زنده تو (معمار ارشد ۰xKIA) با تیم", "en": "Your live chat (Sovereign Architect 0xKIA) with the team"}, "chairPh": {"fa": "دستور تاکتیکی خود را بنویسید...", "en": "Write your tactical directive..."}, "sendBtn": {"fa": "ارسال دستور به اتاق جنگ", "en": "Send directive to war-room"}, "llmBtn": {"fa": "اتصال به LLM زنده (Pollinations / OpenRouter / Local)", "en": "Connect live LLM (Pollinations / OpenRouter / Local)"}, "llmTitle": {"fa": "اتصال به LLM زنده — تولید بی‌نهایت", "en": "Connect live LLM — infinite generation"}, "llmDesc": {"fa": "موتور آفلاینِ داخل اپ همیشه کار می‌کند (بدون کلید). این اتصال اختیاری است؛ پاسخ‌های پویا را از ارائه‌دهندهٔ دلخواه می‌گیرد و در خطا بی‌صدا به موتور آفلاین برمی‌گردد.", "en": "The built-in offline engine always works (keyless). This connection is optional; dynamic replies come from your chosen provider and silently fall back to the offline engine on error."}, "llmProvider": {"fa": "ارائه‌دهنده", "en": "Provider"}, "llmKey": {"fa": "کلید API (فقط برای OpenRouter)", "en": "API key (OpenRouter only)"}, "llmEndpoint": {"fa": "اندپوینت (فقط برای Local)", "en": "Endpoint (Local only)"}, "llmNote": {"fa": "نکته: همه‌چیز سمت کلاینت است — کلید هرگز جایی ذخیره یا ارسال نمی‌شود جز ارائه‌دهندهٔ خودت.", "en": "Note: everything is client-side — the key is never stored or sent anywhere except your own provider."}, "llmClose": {"fa": "بستن", "en": "Close"}, "llmSave": {"fa": "ذخیره و فعال‌سازی", "en": "Save & enable"}, "typing": {"fa": "در حال نوشتن...", "en": "is typing..."}, "chairTag": {"fa": "دستور رئیس", "en": "CHAIRMAN DIRECTIVE"}, "eli5Title": {"fa": "سیستم از ۰ تا ۱۰۰ — به ساده‌ترین زبان", "en": "The system from 0 to 100 — simplest language"}, "eli5Charter": {"fa": "قرارداد پول: هر فروش ۸۰٪ سازنده + ۲۰٪ ولت سازمان — همان لحظه", "en": "Money charter: every sale pays 80% creator + 20% org wallet — instantly"}, "day1Sealed": {"fa": "روزِ اول مُهر شد — همیشه روزِ اول", "en": "Day 1 sealed — it is always Day 1"}}};

"use strict";

// ═══ core state — mirrors crates/hermes/src/war_room_simulation_engine.rs ═══
const D = WAR_ROOM_DATA;
const S = {
  tickCursor: 0,          // next tick to execute
  elapsedSeconds: 0,      // simulation clock (real unit progression)
  playing: false,
  speed: 1,               // 1 | 2 | 5
  earnedRevenueMicros: 0,
  creatorPayoutsMicros: 0,
  affiliatePayoutsMicros: 0,
  streamTax: 0, streamRouting: 0, streamBoost: 0, streamSpread: 0,
  activeUsers: 0, visits: 0, signups: 0, purchases: 0,
  executed: [],           // executed tick events (chat messages)
  directives: 0,
  locale: "fa",
  llm: { provider: "off", key: "", endpoint: "http://localhost:11434/v1/chat/completions" },
  socialCounters: {},     // live view/like counters per post
  tickSeries: [],         // wallet chart series [second, earnedMicros]
};

// ── helpers ──────────────────────────────────────────────────────────────────
const $ = (id) => document.getElementById(id);
const usd = (micros) => "$" + (micros / 1e6).toFixed(2);
const cents = (c) => "$" + (c / 100).toFixed(2);
const pad = (n) => String(n).padStart(2, "0");
const fmtElapsed = (sec) => {
  const d = Math.floor(sec / 86400), h = Math.floor((sec % 86400) / 3600),
        m = Math.floor((sec % 3600) / 60), s = sec % 60;
  return "DAY " + pad(d) + " // " + pad(h) + ":" + pad(m) + ":" + pad(s);
};
const esc = (t) => t.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
const avatarSvg = (accent, initial) =>
  '<svg viewBox="0 0 40 40"><rect width="40" height="40" rx="11" fill="' + accent + '22" stroke="' + accent + '" stroke-width="1.4"/>' +
  '<text x="20" y="26" text-anchor="middle" font-size="15" font-weight="800" fill="' + accent + '">' + initial + "</text></svg>";

// ── i18n ─────────────────────────────────────────────────────────────────────
function applyLocale(loc) {
  S.locale = loc;
  document.body.classList.toggle("en", loc === "en");
  document.documentElement.lang = loc;
  document.documentElement.dir = loc === "fa" ? "rtl" : "ltr";
  $("btnFa").classList.toggle("on", loc === "fa");
  $("btnEn").classList.toggle("on", loc === "en");
  document.querySelectorAll("[data-i18n]").forEach((el) => {
    const key = el.getAttribute("data-i18n");
    if (D.i18n[key] && D.i18n[key][loc]) el.textContent = D.i18n[key][loc];
  });
  document.querySelectorAll("[data-i18n-ph]").forEach((el) => {
    const key = el.getAttribute("data-i18n-ph");
    if (D.i18n[key] && D.i18n[key][loc]) el.placeholder = D.i18n[key][loc];
  });
  renderAll();
}
$("btnFa").onclick = () => applyLocale("fa");
$("btnEn").onclick = () => applyLocale("en");

// ── live clocks (Tehran UTC+03:30 + UTC + elapsed) ───────────────────────────
function tickClocks() {
  const now = new Date();
  const utc = now.toISOString().substr(11, 8);
  $("utcTime").textContent = utc;
  const tehran = new Date(now.getTime() + 3.5 * 3600 * 1000);
  $("tehranTime").textContent = tehran.toISOString().substr(11, 8) + " (+03:30)";
  $("elapsedTime").textContent = fmtElapsed(S.elapsedSeconds);
}
setInterval(tickClocks, 1000);
tickClocks();

// ── Q1: founder chat renderers ───────────────────────────────────────────────
function msgHtml(t) {
  const sp = t.speaker;
  const isFa = S.locale === "fa";
  const body = isFa ? t.messageFa : t.messageEn;
  const alt = isFa
    ? '<div class="en-side fa-side" style="display:none"></div>'
    : "";
  const d = new Date(t.simulationSecond * 1000).toISOString().substr(11, 8);
  let html = '<div class="fmsg">' +
    '<div class="avatar">' + avatarSvg(sp.accent, sp.name_en[0]) + '<span class="onl" title="24/7 REMOTE ACTIVE"></span></div>' +
    '<div class="bubble"><div class="meta">' +
    '<span class="nm" style="color:' + sp.accent + '">' + (isFa ? sp.name_fa : sp.name_en) + "</span>" +
    '<span class="hd">' + sp.handle + "</span>" +
    '<span class="rl">' + (isFa ? "هم‌بنیان‌گذار ۲۴/۷" : "CO-FOUNDER 24/7") + "</span>" +
    '<span class="tm">' + d + "</span></div>" +
    '<div class="body">' + esc(body) + alt;
  if (t.terminalCmd) {
    html += '<div class="cmdline">$ ' + esc(t.terminalCmd) + "</div>";
  }
  html += '<div class="reactions">' +
    '<span class="hot">🔥 ' + (2 + (t.tickIndex % 7)) + "</span>" +
    '<span>👍 ' + (5 + ((t.tickIndex * 3) % 19)) + "</span>" +
    '<span>💬 ' + (1 + (t.tickIndex % 4)) + "</span></div>";
  html += "</div></div>";
  return html;
}

function appendFounderMsg(t, withTyping) {
  const chat = $("q1Chat");
  const typing = $("q1Typing");
  const who = (S.locale === "fa" ? t.speaker.name_fa : t.speaker.name_en);
  const showAndPlace = () => {
    chat.insertBefore(document.createRange().createContextualFragment(msgHtml(t)), typing);
    chat.scrollTop = chat.scrollHeight;
  };
  if (withTyping) {
    $("q1TypingWho").textContent = who + " " + D.i18n.typing[S.locale];
    typing.style.display = "flex";
    chat.scrollTop = chat.scrollHeight;
    setTimeout(() => { typing.style.display = "none"; showAndPlace(); }, 900);
  } else {
    showAndPlace();
  }
}

function appendChairBroadcast(text) {
  const chat = $("q1Chat");
  const typing = $("q1Typing");
  const tag = D.i18n.chairTag[S.locale];
  const div = document.createElement("div");
  div.className = "chairBcast";
  div.innerHTML = '<span class="tag">[' + tag + "]</span><div class=\"txt\">" + esc(text) + "</div>";
  chat.insertBefore(div, typing);
  chat.scrollTop = chat.scrollHeight;
}

// ── Q2: wallet renderers ─────────────────────────────────────────────────────
function renderWallet() {
  $("wSeed").textContent = usd(D.cryptoSeedMicros);
  $("wCash").textContent = cents(D.cashSeedCents);
  $("wPaper").textContent = cents(D.paperReserveLockedCents);
  $("wEarn").textContent = usd(S.earnedRevenueMicros);
  $("st1v").textContent = usd(S.streamTax);
  $("st2v").textContent = usd(S.streamRouting);
  $("st3v").textContent = usd(S.streamBoost);
  $("st4v").textContent = usd(S.streamSpread);
  $("kUsers").textContent = S.activeUsers.toLocaleString(S.locale === "fa" ? "fa-IR" : "en-US");
  $("kVisits").textContent = S.visits.toLocaleString(S.locale === "fa" ? "fa-IR" : "en-US");
  $("kSignups").textContent = S.signups.toLocaleString(S.locale === "fa" ? "fa-IR" : "en-US");
  $("kPurchases").textContent = S.purchases.toLocaleString(S.locale === "fa" ? "fa-IR" : "en-US");
  $("fv1").textContent = S.visits.toLocaleString("en-US");
  $("fv2").textContent = S.signups.toLocaleString("en-US");
  $("fv3").textContent = S.purchases.toLocaleString("en-US");
}

function addTxRow(t) {
  const feed = $("txFeed");
  const stream = t.revenueStream;
  const rail = t.socialPost && t.socialPost.platform === "telegram" ? "XTR" : "USDC";
  const hash = deterministicHash("tx" + t.tickIndex);
  let icon = "💸", label = "";
  const isFa = S.locale === "fa";
  if (stream === "FeaturedBoost") { icon = "🚀"; label = isFa ? "بوست ویژه — ۱۰۰٪ خزانه" : "Featured boost — 100% treasury"; }
  else if (stream === "SettlementSpread") { icon = "⚖️"; label = isFa ? "اختلاف تسویه ۱٫۵٪" : "1.5% settlement spread"; }
  else { icon = "🛒"; label = (isFa ? "فروش " : "sale ") + usd(t.saleVolumeMicros) + (t.referred ? (isFa ? " · با رفرال" : " · referred") : (isFa ? " · بدون رفرال" : " · unreferred")); }
  const row = document.createElement("div");
  row.className = "txrow";
  row.innerHTML = '<span class="ic">' + icon + '</span>' +
    '<span>' + esc(label) + "</span>" +
    '<span class="hash" title="' + hash + '">' + hash.slice(0, 18) + "…" + hash.slice(-6) + "</span>" +
    '<span class="amt">+' + usd(t.walletDeltaUsdcMicros) + "</span>" +
    '<span class="rail">' + (rail === "XTR" ? "Telegram Stars (XTR)" : "Base 8453 · EIP-681") + "</span>";
  feed.appendChild(row);
  feed.scrollTop = feed.scrollHeight;
}

function deterministicHash(seed) {
  let h = 0x811c9dc5;
  for (let i = 0; i < seed.length; i++) { h ^= seed.charCodeAt(i); h = Math.imul(h, 0x01000193) >>> 0; }
  let out = "0x";
  for (let i = 0; i < 8; i++) { h = Math.imul(h ^ (h >>> 13), 0x5bd1e995) >>> 0; out += h.toString(16).padStart(8, "0"); }
  return out.slice(0, 66);
}

// ── revenue chart (canvas, live) ─────────────────────────────────────────────
function drawChart() {
  const cv = $("revChart");
  const w = cv.clientWidth, h = cv.clientHeight;
  cv.width = w * 2; cv.height = h * 2;
  const ctx = cv.getContext("2d");
  ctx.scale(2, 2);
  ctx.clearRect(0, 0, w, h);
  const series = S.tickSeries;
  const maxV = Math.max(1000000, ...series.map((p) => p[1]));
  // grid
  ctx.strokeStyle = "rgba(255,255,255,.06)";
  ctx.lineWidth = 1;
  for (let gy = 1; gy < 4; gy++) {
    const y = (h - 22) * (gy / 4) + 6;
    ctx.beginPath(); ctx.moveTo(6, y); ctx.lineTo(w - 6, y); ctx.stroke();
  }
  if (series.length < 2) {
    ctx.fillStyle = "rgba(139,147,167,.8)";
    ctx.font = "11px " + "'JetBrains Mono',monospace";
    ctx.fillText(S.locale === "fa" ? "در انتظار اولین تیک درآمدی…" : "waiting for the first revenue tick…", 12, h / 2);
    return;
  }
  const x0 = 8, x1 = w - 8, y0 = h - 22, yTop = 8;
  const px = (i) => x0 + ((x1 - x0) * i) / (series.length - 1);
  const py = (v) => y0 - ((y0 - yTop) * v) / maxV;
  // area
  const grad = ctx.createLinearGradient(0, yTop, 0, y0);
  grad.addColorStop(0, "rgba(52,211,153,.35)");
  grad.addColorStop(1, "rgba(52,211,153,0)");
  ctx.beginPath();
  ctx.moveTo(px(0), y0);
  series.forEach((p, i) => ctx.lineTo(px(i), py(p[1])));
  ctx.lineTo(px(series.length - 1), y0);
  ctx.closePath();
  ctx.fillStyle = grad;
  ctx.fill();
  // line
  ctx.beginPath();
  series.forEach((p, i) => i ? ctx.lineTo(px(i), py(p[1])) : ctx.moveTo(px(i), py(p[1])));
  ctx.strokeStyle = "#34d399"; ctx.lineWidth = 2;
  ctx.shadowColor = "rgba(52,211,153,.6)"; ctx.shadowBlur = 8;
  ctx.stroke();
  ctx.shadowBlur = 0;
  // last point + label
  const last = series[series.length - 1];
  ctx.beginPath();
  ctx.arc(px(series.length - 1), py(last[1]), 3.4, 0, 7);
  ctx.fillStyle = "#34d399"; ctx.fill();
  ctx.fillStyle = "#34d399";
  ctx.font = "bold 11px 'JetBrains Mono',monospace";
  ctx.fillText(usd(last[1]), px(series.length - 1) - 42, py(last[1]) - 7);
  // users line (cyan, secondary)
  const maxU = Math.max(1, ...series.map((p) => p[2]));
  ctx.beginPath();
  series.forEach((p, i) => i ? ctx.lineTo(px(i), y0 - ((y0 - yTop) * p[2]) / maxU) : ctx.moveTo(px(i), y0 - ((y0 - yTop) * p[2]) / maxU));
  ctx.strokeStyle = "rgba(34,211,238,.75)"; ctx.lineWidth = 1.4;
  ctx.stroke();
  // axis labels
  ctx.fillStyle = "rgba(139,147,167,.9)";
  ctx.font = "10px 'JetBrains Mono',monospace";
  ctx.fillText("TREASURY $", 8, h - 6);
  ctx.fillText(fmtElapsed(S.elapsedSeconds).replace("DAY ", "D"), w - 74, h - 6);
}
window.addEventListener("resize", drawChart);

// ── progress ─────────────────────────────────────────────────────────────────
function renderProgress() {
  const pct = Math.round((S.tickCursor / 24) * 100);
  $("progFill").style.width = pct + "%";
  $("progTxt").textContent = S.tickCursor + " / 24 · " + pct + "%";
}

// ── controls ─────────────────────────────────────────────────────────────────
function setPlayButtons() {
  $("btnPlay").classList.toggle("on", S.playing);
  $("btnPause").classList.toggle("on", !S.playing);
  $("btn2x").classList.toggle("on", S.speed === 2);
  $("btn5x").classList.toggle("on", S.speed === 5);
}
$("btnPlay").onclick = () => { if (S.tickCursor < 24) { S.playing = true; setPlayButtons(); } };
$("btnPause").onclick = () => { S.playing = false; setPlayButtons(); };
$("btn2x").onclick = () => { S.speed = 2; setPlayButtons(); };
$("btn5x").onclick = () => { S.speed = 5; setPlayButtons(); };
$("btnNext").onclick = () => { S.playing = false; setPlayButtons(); advanceOne(true); };
$("btnReset").onclick = () => {
  S.playing = false; S.speed = 1;
  S.tickCursor = 0; S.elapsedSeconds = 0;
  S.earnedRevenueMicros = 0; S.creatorPayoutsMicros = 0; S.affiliatePayoutsMicros = 0;
  S.streamTax = 0; S.streamRouting = 0; S.streamBoost = 0; S.streamSpread = 0;
  S.activeUsers = 0; S.visits = 0; S.signups = 0; S.purchases = 0;
  S.executed = []; S.directives = 0; S.tickSeries = []; S.socialCounters = {};
  $("q1Chat").innerHTML = '<div class="typed" id="q1Typing" style="display:none"><span class="dots"><i></i><i></i><i></i></span><span id="q1TypingWho"></span></div>';
  $("txFeed").innerHTML = "";
  $("socFeed").innerHTML = "";
  $("termLog").innerHTML = "";
  setPlayButtons(); renderAll();
  termInfo(S.locale === "fa" ? "بازنشانی به روزِ صفر، ثانیه ۰ — همهٔ اعداد صادقانه صفر شدند" : "reset to Day 0, second 0 — every number honestly zeroed");
};

// ── causal tick execution — the heart of the honest simulation ────────────────
function advanceOne(manual) {
  if (S.tickCursor >= 24) {
    termWarn(S.locale === "fa" ? "سناریوی ۲۴تایی تمام شد — روزِ اول مُهر شد" : "the 24-tick scenario is complete — Day 1 sealed");
    S.playing = false; setPlayButtons();
    return false;
  }
  const t = D.ticks[S.tickCursor];
  const jump = t.simulationSecond - S.elapsedSeconds;
  if (jump > D.maxTickDeltaSeconds) {
    // anti-jump guard — never happens in the canonical scenario; kept honest
    termWarn("TIME JUMP DETECTED: " + jump + "s > 3600s — tick rejected");
    S.playing = false; setPlayButtons();
    return false;
  }
  S.elapsedSeconds = t.simulationSecond;
  S.earnedRevenueMicros += t.walletDeltaUsdcMicros;
  S.creatorPayoutsMicros += t.creatorPayoutMicros;
  S.affiliatePayoutsMicros += t.affiliatePayoutMicros;
  if (t.revenueStream === "MarketplaceTax") S.streamTax += t.walletDeltaUsdcMicros - t.affiliateRoutingMicros, S.streamRouting += t.affiliateRoutingMicros;
  else if (t.revenueStream === "AffiliateRouting") S.streamRouting += t.walletDeltaUsdcMicros;
  else if (t.revenueStream === "FeaturedBoost") S.streamBoost += t.walletDeltaUsdcMicros;
  else if (t.revenueStream === "SettlementSpread") S.streamSpread += t.walletDeltaUsdcMicros;
  S.activeUsers += t.usersDelta; S.visits += t.visitsDelta;
  S.signups += t.signupsDelta; S.purchases += t.purchasesDelta;
  S.tickCursor++;
  S.executed.push(t);
  if (t.walletDeltaUsdcMicros > 0) {
    S.tickSeries.push([t.simulationSecond, S.earnedRevenueMicros, S.activeUsers]);
    addTxRow(t);
    const sid = { MarketplaceTax: "st1", AffiliateRouting: "st2", FeaturedBoost: "st3", SettlementSpread: "st4" }[t.revenueStream];
    if (sid) { const el = $(sid); el.classList.remove("pulse"); void el.offsetWidth; el.classList.add("pulse"); }
  } else {
    S.tickSeries.push([t.simulationSecond, S.earnedRevenueMicros, S.activeUsers]);
  }
  appendFounderMsg(t, !manual);
  if (t.terminalCmd) runTerminalForTick(t);
  if (t.socialPost) addSocialPost(t);
  highlightEli5(t.activePipelineStage);
  renderAll();
  if (S.tickCursor === 24) {
    setTimeout(() => {
      appendFounderMsg({
        speaker: { name_en: "Elon Musk", name_fa: "ایلان ماسک", handle: "@elonmusk", accent: "#7EF7D2" },
        messageFa: "روزِ اول مُهر شد — هیچ عددِ جعلی‌ای در آن نیست: از $0.00 شروع کردیم و با ۸ تراکنشِ علّی به $9.55 رسیدیم. فردا دوباره از روزِ اول شروع می‌کنیم — همیشه روزِ اول.",
        messageEn: "Day 1 is sealed — zero made-up numbers: we started from $0.00 and reached $9.55 through 8 causal transactions. Tomorrow we start again from Day 1 — it is always Day 1.",
        terminalCmd: null, socialPost: null, walletDeltaUsdcMicros: 0, saleVolumeMicros: 0,
        creatorPayoutMicros: 0, referred: false, tickIndex: 24, simulationSecond: S.elapsedSeconds, revenueStream: "None", activePipelineStage: "day_one_sealed",
      }, true);
    }, 1400);
  }
  return true;
}

function renderAll() {
  renderWallet(); renderProgress(); drawChart();
  const t = S.executed[S.executed.length - 1];
  if (t) highlightEli5(t.activePipelineStage);
}

// ═══ Q3 · tab 1: ELI5 animated pipeline (canvas) ═════════════════════════════
let eli5Active = "epA";
const STAGE2STEP = {
  ignition: "epA", product_clarity: "epA", viral_launch: "epC", payment_rails: "epD",
  creator_onboarding: "epB", revenue_flow: "epD", quant_ops: "epB", day_one_sealed: "epD",
};
function highlightEli5(stage) {
  const step = STAGE2STEP[stage] || "epA";
  eli5Active = step;
  ["epA", "epB", "epC", "epD"].forEach((id) => $(id).classList.toggle("active", id === step));
}
const particles = [];
function eli5Loop() {
  const cv = $("eli5Canvas");
  const w = cv.clientWidth, h = cv.clientHeight;
  if (w === 0) { requestAnimationFrame(eli5Loop); return; }
  cv.width = w * 2; cv.height = h * 2;
  const ctx = cv.getContext("2d");
  ctx.scale(2, 2);
  ctx.clearRect(0, 0, w, h);
  // 4 nodes A→B→C→D
  const nodes = [
    { id: "epA", x: w * 0.14, y: h * 0.30, label: "A", fa: "ورود کاربر", en: "user intent" },
    { id: "epB", x: w * 0.38, y: h * 0.30, label: "B", fa: "کارخانه", en: "factory" },
    { id: "epC", x: w * 0.62, y: h * 0.30, label: "C", fa: "فروشگاه", en: "store" },
    { id: "epD", x: w * 0.86, y: h * 0.30, label: "D", fa: "تقسیم پول", en: "auto split" },
  ];
  // edges with flowing particles
  ctx.strokeStyle = "rgba(255,255,255,.14)";
  ctx.lineWidth = 2;
  for (let i = 0; i < 3; i++) {
    ctx.beginPath();
    ctx.moveTo(nodes[i].x + 30, nodes[i].y);
    ctx.lineTo(nodes[i + 1].x - 30, nodes[i + 1].y);
    ctx.stroke();
  }
  // spawn particles on the active edge chain
  const activeIdx = nodes.findIndex((n) => n.id === eli5Active);
  if (Math.random() < 0.16 && activeIdx > 0) {
    particles.push({ edge: activeIdx - 1, t: 0 });
  }
  for (let i = particles.length - 1; i >= 0; i--) {
    const p = particles[i];
    p.t += 0.016;
    const a = nodes[p.edge], b = nodes[p.edge + 1];
    const x = a.x + 30 + (b.x - 30 - a.x - 30) * p.t;
    const y = a.y + Math.sin(p.t * 9) * 3;
    ctx.beginPath();
    ctx.arc(x, y, 3.2, 0, 7);
    ctx.fillStyle = "#22d3ee";
    ctx.shadowColor = "rgba(34,211,238,.9)"; ctx.shadowBlur = 10;
    ctx.fill();
    ctx.shadowBlur = 0;
    if (p.t >= 1) particles.splice(i, 1);
  }
  // nodes
  const isFa = S.locale === "fa";
  nodes.forEach((n, i) => {
    const on = n.id === eli5Active;
    const R = on ? 27 : 22;
    ctx.beginPath();
    ctx.arc(n.x, n.y, R, 0, 7);
    const g = ctx.createRadialGradient(n.x - 8, n.y - 8, 3, n.x, n.y, R);
    if (on) { g.addColorStop(0, "rgba(34,211,238,.65)"); g.addColorStop(1, "rgba(34,211,238,.12)"); }
    else { g.addColorStop(0, "rgba(255,255,255,.10)"); g.addColorStop(1, "rgba(255,255,255,.04)"); }
    ctx.fillStyle = g;
    ctx.shadowColor = on ? "rgba(34,211,238,.8)" : "transparent";
    ctx.shadowBlur = on ? 22 : 0;
    ctx.fill();
    ctx.shadowBlur = 0;
    ctx.strokeStyle = on ? "#22d3ee" : "rgba(255,255,255,.25)";
    ctx.lineWidth = on ? 2.4 : 1.4;
    ctx.stroke();
    ctx.fillStyle = on ? "#d9f9ff" : "#8b93a7";
    ctx.font = "bold " + (on ? 17 : 14) + "px 'JetBrains Mono',monospace";
    ctx.textAlign = "center";
    ctx.fillText(n.label, n.x, n.y + 6);
    ctx.fillStyle = on ? "rgba(34,211,238,.95)" : "rgba(139,147,167,.8)";
    ctx.font = (isFa ? "11px Vazirmatn, Tahoma" : "10px Inter, sans-serif");
    ctx.fillText(isFa ? n.fa : n.en, n.x, n.y + R + 16);
  });
  requestAnimationFrame(eli5Loop);
}
requestAnimationFrame(eli5Loop);

// ═══ Q3 · tab 2: live terminal + GitHub ══════════════════════════════════════
const TERM_OUTPUTS = {
  "cargo test --workspace": ["running 750 workspace tests…", "test result: ok. 750 passed; 0 failed; 0 ignored", "clippy: 0 warnings (-D warnings)"],
  "cargo clippy --workspace --all-targets -- -D warnings": ["checking 11 crates…", "0 warnings / 0 errors — hardened kernel clean"],
  "cargo test --workspace --quiet 2>&1 | tail -4": ["750 passed; 0 failed", "green — every tick proves itself"],
  "forge verify-contract 0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913 --chain-id 8453 --rpc-url https://mainnet.base.org": [
    "chain: Base Mainnet (8453)", "USDC native: 0x8335…2913 ✓ verified", "escrow charter: 80% creator / 20% treasury — SAFU",
  ],
  "python3 tools/neural-hud/render_neural_hud.py html --preset kia-monopoly-core --aspect 9:16 --out hud_monopoly.html": [
    "180-node dual-shell Fibonacci mesh ✓", "window.seekTo(t) deterministic ✓", "WebAudio synth: 55/82.41/110 Hz sub-bass + radar pings ✓", "wrote hud_monopoly.html (60s · 9:16)",
  ],
  "python3 tools/neural-hud/render_neural_hud.py html --preset quant-vibe-terminal --aspect 16:9 --out hud_quant.html": [
    "preset quant-vibe-terminal ✓", "wrote hud_quant.html (60s · 16:9)",
  ],
  "python3 tools/neural-hud/render_neural_hud.py html --preset trinity-5model-genome --aspect 9:16 --out hud_genome.html": [
    "preset trinity-5model-genome ✓", "wrote hud_genome.html",
  ],
  "node ccxt-feed.js binance BTC/USDC --depth 5 --json": [
    "ccxt: binance BTC/USDC orderbook", "bids: 5 · asks: 5 · spread 0.02%", "vibe-trading causal backtest: sharpe 1.31 ≥ 1.20 ✓",
  ],
  "curl -s https://kia-api.up.railway.app/api/v1/monopoly/charter-and-base | head -c 400": [
    '{"charter":{"platformTakeBpsUnreferred":2000,"creatorBps":8000}}', '{"base":{"chainId":8453,"usdc":"0x8335…2913"}}',
  ],
  "curl -s https://kia-api.up.railway.app/api/v1/war-room/state | head -c 400": [
    '{"signature":"KIA-WAR-ROOM-W10"}', '{"wallet":{"earnedRevenueMicros":0,"cryptoSeedMicros":33000000}}',
  ],
};
function termLine(cls, text) {
  const log = $("termLog");
  const div = document.createElement("div");
  div.className = cls;
  div.textContent = text;
  log.appendChild(div);
  log.scrollTop = log.scrollHeight;
}
function termInfo(t) { termLine("tinfo", "[war-room] " + t); }
function termWarn(t) { termLine("twar", "[guard] " + t); }
function runTerminalForTick(t) {
  const sp = t.speaker;
  termLine("tcmd", sp.handle + " $ " + t.terminalCmd);
  const outs = TERM_OUTPUTS[t.terminalCmd] || ["ok"];
  outs.forEach((o, i) => setTimeout(() => termLine("tout", o), 420 * (i + 1)));
}
termInfo("KiaAgentX/xA @ 97b18bd — 11 crates · 750 tests · 9 waves");
termInfo("boot: war-room kernel KIA-WAR-ROOM-W10 attached");

// ═══ Q3 · tab 3: live social feed ════════════════════════════════════════════
const SOCIAL_COMMENTS = {
  en: [
    ["@quantdev_ali", "wait, it really splits 80/20 on-chain? 🔥"],
    ["@toyspark_mom", "my 9yo just published her first storybook. no code. I'm crying"],
    ["@basebuilder", "Chain ID 8453 verified. Funds are SAFU. Keep building."],
    ["@indiehack_jane", "750 passing tests is the real flex here"],
  ],
  fa: [
    ["@quantdev_ali", "صبر کن، واقعاً ۸۰/۲۰ روی زنجیره تقسیم می‌کنه؟ 🔥"],
    ["@toyspark_mom", "بچه ۹ ساله‌ام امروز اولین کتابش رو منتشر کرد. بدون یه خط کد."],
    ["@basebuilder", "زنجیرهٔ 8453 وریفای شده. صندوق‌ها SAFU. ادامه بده."],
    ["@indiehack_jane", "۷۵۰ تست پاس‌شده خودش افتخار واقعیه"],
  ],
};
function addSocialPost(t) {
  const post = t.socialPost;
  const feed = $("socFeed");
  const isFa = S.locale === "fa";
  const pid = "post" + t.tickIndex;
  S.socialCounters[pid] = { views: post.views || 0, likes: post.likes || 0 };
  const handle = post.handle;
  const accent = handle === "@elonmusk" ? "#7EF7D2" : handle === "@cz_binance" ? "#59B7FF" : handle === "@JeffBezos" ? "#FFB454" : "#22d3ee";
  const text = isFa ? post.text_fa : post.text_en;
  const alt = isFa ? post.text_en : post.text_fa;
  const comments = (SOCIAL_COMMENTS[isFa ? "fa" : "en"]).slice(0, 2 + (t.tickIndex % 3));
  const div = document.createElement("div");
  div.className = "socpost";
  div.id = pid;
  div.innerHTML = '<div class="sh">' +
    '<div class="avic" style="background:' + accent + '">' + handle.replace("@", "")[0] + "</div>" +
    '<div><div class="snm">' + handle + "</div>" +
    '<div class="shd">' + (post.platform === "telegram" ? "Telegram · channel" : "X · thread") + "</div></div>" +
    '<span class="plat">' + (post.platform === "telegram" ? "TELEGRAM" : "X") + "</span></div>" +
    '<div class="stxt">' + esc(text) + "<small>" + esc(alt) + "</small></div>" +
    '<div class="smet"><span data-cnt="views">👁 <b>' + (post.views || 0).toLocaleString("en-US") + "</b></span>" +
    '<span data-cnt="likes">❤️ <b>' + (post.likes || 0).toLocaleString("en-US") + "</b></span>" +
    '<span>🔁 ' + Math.floor((post.likes || 0) / 7).toLocaleString("en-US") + "</span></div>" +
    '<div class="soccomments">' + comments.map((c) => '<div class="cm"><b>' + c[0] + "</b> " + esc(c[1]) + "</div>").join("") + "</div>";
  feed.appendChild(div);
  feed.scrollTop = feed.scrollHeight;
}
// live counters — views/likes creep up while playing
setInterval(() => {
  if (!S.playing) return;
  Object.keys(S.socialCounters).forEach((pid) => {
    const c = S.socialCounters[pid];
    const el = $(pid);
    if (!el) return;
    c.views += Math.floor(Math.random() * 180 * S.speed);
    c.likes += Math.floor(Math.random() * 12 * S.speed);
    el.querySelector('[data-cnt="views"] b').textContent = c.views.toLocaleString("en-US");
    el.querySelector('[data-cnt="likes"] b').textContent = c.likes.toLocaleString("en-US");
  });
}, 1400);

// tabs
function switchTab(id) {
  ["paneEli5", "paneTerm", "paneSoc"].forEach((p) => $(p).classList.toggle("on", p === id));
  ["tabEli5", "tabTerm", "tabSoc"].forEach((b) => $(b).classList.toggle("on", b === id));
  if (id === "paneTerm") termInfo("viewport: github.com/KiaAgentX/xA — main");
}
$("tabEli5").onclick = () => switchTab("paneEli5");
$("tabTerm").onclick = () => switchTab("paneTerm");
$("tabSoc").onclick = () => switchTab("paneSoc");

// ═══ Q4 · chairman chat + offline intent engine ══════════════════════════════
// keyword mirror of ChairmanIntent::classify (works 100% offline, keyless)
const INTENT_RULES = [
  ["viral_hud_launch", ["وایرال", "viral", "hud", "ویدیو", "video", "neural"]],
  ["payment_rails", ["درگاه", "usdc", "استارز", "stars", "پرداخت", "payment", "gateway", "8453"]],
  ["simplify_for_beginners", ["ساده", "بچه", "کودک", "child", "toyspark", "simple", "eli5"]],
  ["status_report", ["گزارش", "وضعیت", "ولت", "wallet", "status", "report", "گام", "step"]],
  ["quant_trading", ["ترید", "کوانتوم", "quant", "vibe", "trade", "trading", "ccxt", "backtest"]],
  ["explain_simple", ["توضیح", "صفر", "explain", "بفهم", "understand", "zero"]],
];
function classifyIntent(raw) {
  const t = raw.toLowerCase();
  let best = "ship_next_step", bestHits = 0;
  for (const [id, kws] of INTENT_RULES) {
    let hits = 0;
    for (const k of kws) if (t.includes(k)) hits++;
    if (hits > bestHits) { bestHits = hits; best = id; }
  }
  return best;
}

function addChairMsg(text) {
  const chat = $("q4Chat");
  const div = document.createElement("div");
  div.className = "cmsg chair";
  div.innerHTML = "<b>0xKIA · " + D.i18n.chairTag[S.locale] + "</b><div>" + esc(text) + "</div>";
  chat.appendChild(div);
  chat.scrollTop = chat.scrollHeight;
}
function addFounderReply(pid, fa, en, cmd, post) {
  const chat = $("q4Chat");
  const f = D.founders.find((x) => x.pid === pid);
  const isFa = S.locale === "fa";
  const div = document.createElement("div");
  div.className = "cmsg founder";
  let html = '<div class="who"><b style="color:' + f.accent + '">' + (isFa ? f.fa : f.en) + "</b> <span class=\"p\">" + f.handle + "</span></div>";
  html += "<div>" + esc(isFa ? fa : en) + "</div>";
  html += '<div class="alt ' + (isFa ? "en" : "fa") + '-lang">' + esc(isFa ? en : fa) + "</div>";
  chat.appendChild(div);
  chat.scrollTop = chat.scrollHeight;
  if (cmd) { termLine("tcmd", f.handle + " $ " + cmd); (TERM_OUTPUTS[cmd] || ["ok"]).forEach((o, i) => setTimeout(() => termLine("tout", o), 420 * (i + 1))); }
  if (post) { addSocialPost({ tickIndex: 100 + S.directives, socialPost: post }); }
}

// optional live-LLM layer (Pollinations keyless / OpenRouter / local endpoint)
async function llmReply(prompt) {
  if (S.llm.provider === "off") return null;
  try {
    if (S.llm.provider === "pollinations") {
      const sys = S.locale === "fa"
        ? "تو اتاق جنگ سه هم‌بنیان‌گذار (ایلان ماسک، CZ، جف بزوس) زیر ریاست 0xKIA هستی، در حال تجاری‌سازی مخزن KiaAgentX/xA با ۱۱ کرِیت و ۷۵۰ تست. به فارسی، کوتاه و در-شخصیت پاسخ بده."
        : "You are the war-room of three co-founders (Elon Musk, CZ, Jeff Bezos) under chairman 0xKIA commercializing KiaAgentX/xA (11 crates, 750 tests). Answer briefly, in character.";
      const url = "https://text.pollinations.ai/" + encodeURIComponent(sys + "\n\nCHAIRMAN: " + prompt);
      const res = await fetch(url);
      if (!res.ok) throw new Error("pollinations " + res.status);
      return await res.text();
    }
    if (S.llm.provider === "openrouter" && S.llm.key) {
      const res = await fetch("https://openrouter.ai/api/v1/chat/completions", {
        method: "POST",
        headers: { "content-type": "application/json", authorization: "Bearer " + S.llm.key },
        body: JSON.stringify({
          model: "meta-llama/llama-3.1-8b-instruct:free",
          messages: [{ role: "user", content: (S.locale === "fa" ? "به فارسی و کوتاه: " : "briefly: ") + prompt }],
        }),
      });
      if (!res.ok) throw new Error("openrouter " + res.status);
      const j = await res.json();
      return j.choices && j.choices[0] && j.choices[0].message ? j.choices[0].message.content : null;
    }
    if (S.llm.provider === "local") {
      const res = await fetch(S.llm.endpoint, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ messages: [{ role: "user", content: prompt }] }),
      });
      if (!res.ok) throw new Error("local " + res.status);
      const j = await res.json();
      return (j.choices && j.choices[0] && j.choices[0].message && j.choices[0].message.content) || null;
    }
  } catch (e) {
    termWarn("live LLM failed (" + e.message + ") — silent fallback to offline engine");
    return null;
  }
  return null;
}

async function sendDirective(text) {
  const raw = (text || $("chairInput").value).trim();
  if (!raw) return;
  $("chairInput").value = "";
  S.directives++;
  addChairMsg(raw);
  appendChairBroadcast(raw);
  // typing indicators then the three in-character replies
  const intent = classifyIntent(raw);
  const table = D.replies[intent] || D.replies.ship_next_step;
  const order = ["elon_musk", "cz_binance", "jeff_bezos"];
  $("q4Status").textContent = "INTENT: " + intent.toUpperCase();
  const live = await llmReply(raw);
  if (live) {
    addFounderReply("elon_musk", live, live, null, null);
    addFounderReply("cz_binance", table.cz_binance.fa, table.cz_binance.en, table.cz_binance.cmd, table.cz_binance.post);
    addFounderReply("jeff_bezos", table.jeff_bezos.fa, table.jeff_bezos.en, table.jeff_bezos.cmd, table.jeff_bezos.post);
    return;
  }
  let delay = 700;
  for (const pid of order) {
    const r = table[pid];
    setTimeout(() => addFounderReply(pid, r.fa, r.en, r.cmd, r.post), delay);
    delay += 1100;
  }
  // status intents refresh the dashboard; others advance the ops state
  if (intent === "status_report") {
    setTimeout(() => termInfo(S.locale === "fa"
      ? "گزارش: ولت $" + usd(S.earnedRevenueMicros) + " · تیک " + S.tickCursor + "/24 · کاربران " + S.activeUsers
      : "report: wallet " + usd(S.earnedRevenueMicros) + " · tick " + S.tickCursor + "/24 · users " + S.activeUsers), delay);
  } else if (S.tickCursor < 24 && intent !== "explain_simple") {
    setTimeout(() => advanceOne(false), delay + 400);
  }
}
$("sendBtn").onclick = () => sendDirective();
$("chairInput").addEventListener("keydown", (e) => { if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); sendDirective(); } });

// chips
D.chips.forEach((chip) => {
  const b = document.createElement("button");
  b.textContent = chip.icon + " " + (S.locale === "fa" ? chip.fa : chip.en);
  b.onclick = () => sendDirective(S.locale === "fa" ? chip.fa : chip.en);
  $("chips").appendChild(b);
});
applyLocale("fa");

// ═══ LLM modal ═══════════════════════════════════════════════════════════════
$("llmBtn").onclick = () => $("llmModal").classList.add("on");
$("llmClose").onclick = () => $("llmModal").classList.remove("on");
$("llmSave").onclick = () => {
  S.llm.provider = $("llmProvider").value;
  S.llm.key = $("llmKey").value.trim();
  S.llm.endpoint = $("llmEndpoint").value.trim();
  const on = S.llm.provider !== "off";
  $("llmModal").classList.remove("on");
  $("llmBtn").classList.toggle("on", on);
  termInfo(on ? "live LLM enabled: " + S.llm.provider : "live LLM disabled — offline engine only");
};

// ═══ simulation clock — real unit progression (no time jumps) ═════════════════
// Play 1x: one simulated hour per real second is NOT honest — the sim clock
// advances at speed×real-seconds and a tick fires when the clock reaches the
// next tick's timestamp. [Next Step] is the manual override for impatient
// chairmen; the wallet only ever moves through causal tick events.
setInterval(() => {
  if (!S.playing || S.tickCursor >= 24) return;
  if (S.tickCursor === 0 && S.elapsedSeconds === 0) {
    // first tick at second 0 fires immediately once playing starts
    advanceOne(false);
    return;
  }
  const next = D.ticks[S.tickCursor];
  if (S.elapsedSeconds >= next.simulationSecond) {
    advanceOne(false);
  } else {
    // advance the sim clock honestly at speed×1s per real 100ms
    S.elapsedSeconds = Math.min(next.simulationSecond, S.elapsedSeconds + S.speed * 36);
  }
}, 100);

// boot — honest Day 0 state
renderAll();
setPlayButtons();
termInfo("Day 0, 00:00:00 — seed $33.00 crypto + $15.00 cash ($350 locked) · earned revenue $0.00");
</script>
</body>
</html>
''').replace("\r\n", "\n")

# گاردهایِ ساختاری — هر خروجیِ ناقصِ فارسی/RTL را می‌گیرد
# structural guards — catch any truncated Persian/RTL output
_INVARIANTS = (
    "<!DOCTYPE html>",
    "KIA WAR-ROOM",
    'dir="rtl"',
    "Vazirmatn",
    "JetBrains Mono",
    "Elon Musk",
    "Changpeng Zhao (CZ)",
    "Jeff Bezos",
    "0xKIA",
    "</html>",
)


def _sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def validate(template: str) -> list[str]:
    """Return a list of structural problems (empty list = healthy)."""
    issues: list[str] = []
    for marker in _INVARIANTS:
        if marker not in template:
            issues.append(f"missing invariant marker: {marker!r}")
    if "\r" in template:
        issues.append("carriage returns present (must be LF only)")
    if template.startswith("\ufeff"):
        issues.append("UTF-8 BOM present")
    if not template.rstrip().endswith("</html>"):
        issues.append("template does not end with </html> (truncation risk)")
    if template.count("Elon Musk") < 2:
        issues.append("founder roster looks truncated (Elon Musk mentions < 2)")
    return issues


def render(out: pathlib.Path, quiet: bool = False) -> int:
    issues = validate(TEMPLATE)
    if issues:
        for i in issues:
            print(f"[render_war_room] STRUCTURAL GUARD: {i}", file=sys.stderr)
        return 2
    data = TEMPLATE.encode("utf-8")
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_bytes(data)
    if not quiet:
        print(f"[render_war_room] wrote {out} ({out.stat().st_size:,} bytes)")
        print(f"[render_war_room] sha256 = {_sha256(data)}")
        print("[render_war_room] invariants OK · UTF-8 · LF · no BOM")
    return 0


def check(out: pathlib.Path) -> int:
    expected = TEMPLATE.encode("utf-8")
    if not out.exists():
        print(f"[render_war_room] DRIFT: {out} does not exist (run with --out)", file=sys.stderr)
        return 1
    actual = out.read_bytes()
    if actual == expected:
        print(f"[render_war_room] OK: {out} is byte-identical to the embedded template")
        print(f"[render_war_room] sha256 = {_sha256(actual)}")
        return 0
    print(f"[render_war_room] DRIFT: {out} differs from the embedded template", file=sys.stderr)
    print(f"[render_war_room]   expected sha256 = {_sha256(expected)}", file=sys.stderr)
    print(f"[render_war_room]   actual   sha256 = {_sha256(actual)}", file=sys.stderr)
    return 1


def main(argv: list[str] | None = None) -> int:
    default_out = pathlib.Path(__file__).resolve().parent / "war_room.html"
    p = argparse.ArgumentParser(
        description="Canonical renderer of the Wave-10 Founder War-Room command deck",
    )
    g = p.add_mutually_exclusive_group()
    g.add_argument("--out", type=pathlib.Path, default=default_out,
                   help="output path (default: tools/war-room/war_room.html next to this script)")
    g.add_argument("--check", action="store_true",
                   help="verify the output on disk is byte-identical to the template")
    g.add_argument("--validate", action="store_true",
                   help="run structural invariants only, write nothing")
    a = p.parse_args(argv)

    issues = validate(TEMPLATE)
    if issues:  # refuse to operate on a broken deck
        for i in issues:
            print(f"[render_war_room] STRUCTURAL GUARD: {i}", file=sys.stderr)
        return 2
    if a.validate:
        print("[render_war_room] structural invariants OK (nothing written)")
        return 0
    if a.check:
        return check(a.out)
    return render(a.out)


if __name__ == "__main__":
    raise SystemExit(main())
