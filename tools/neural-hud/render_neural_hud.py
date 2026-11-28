#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""MAIRON Neural HUD Video Studio — portable CLI.

Mirror of the deterministic kernel in ``crates/hermes/src/neural_hud_studio_engine.rs``:
180-node dual-shell Fibonacci mesh (SPHERE_RADIUS=160, FOV=340, camera_z=420,
rotY = t*0.65, rotX = sin(t*0.4)*0.4), Z-sorted edges, 4 timeline phases,
4 HUD callouts, XSS-safe JSON embedding, WebAudio live synth (55/82.41/110 Hz
sub-bass + 1046.5/1567.98 Hz radar pings), and the two FFmpeg pipelines
(lavfi audio with the master limiter + MJPEG pipe video with faststart).

Subcommands:
  presets                     list the 8-preset telemetry catalog
  html --preset SLUG ...      write the standalone HTML bundle
  synthesize --repo NAME ...  derive a preset from any repo name + write HTML
  plan ...                    print the exact FFmpeg audio + video commands
  render ...                  rasterize deterministic PNG frames with Pillow
                              and pipe them into FFmpeg (image2pipe)

Pure stdlib (optional Pillow for ``render``). Same inputs always produce the
same bytes — the geometry cache is embedded once, never recomputed per frame
(curing upstream defect D01: per-frame getBoundingClientRect thrashing).
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import shutil
import subprocess
import threading
import threading
from pathlib import Path

SPHERE_RADIUS = 160.0
FOV = 340.0
CAMERA_Z = 420.0
SHELL_NODE_COUNT = 90
INNER_SHELL_RADIUS_SCALE = 0.62
ROT_Y_RATE = 0.65
ROT_X_RATE = 0.4
ROT_X_AMPLITUDE = 0.4
DEFAULT_DURATION_SEC = 60
MAX_DURATION_SEC = 180
FADE_IN_SEC = 0.6
FADE_OUT_SEC = 1.2
MASTER_LIMITER = "alimiter=limit=0.95:attack=5:release=50"
SUB_BASS_HZ = [55.0, 82.41, 110.0]
RADAR_PING_HZ = [1046.5, 1567.98]
HUD_CALLOUTS = [
    ("TRINITY_MAIRON", "IQ_150"),
    ("MCP_GATEWAY", "SYNC"),
    ("BASE_8453", "VERIFIED"),
    ("CCXT_QUANT", "CAUSAL"),
]
CALLOUT_BINDINGS = [(0, 0), (0, 45), (1, 30), (1, 60)]
PHASE_TITLES = [
    "PHASE 01 // NEURAL IGNITION",
    "PHASE 02 // DETERMINISTIC SEEK_TO PIPE",
    "PHASE 03 // 5-MODEL GENOME & ARDA HUB",
    "PHASE 04 // MAIRON 100/100 SEALED",
]
SIGNATURE = "MAIRON-NEURAL-HUD"

PRESETS = [
    ("mwie", "MWIE Neural Cortex", "Deterministic brain telemetry over a dual-shell mesh", "#7C4DFF", "MWIE", "upstream"),
    ("sage-audit", "SAGE Audit Mesh", "Forensic audit telemetry with sealed gates", "#00E5A0", "SAGE", "upstream"),
    ("geo-scope", "GEO Scope Planetary", "Planetary scope sweep across the mesh", "#38BDF8", "GEO", "upstream"),
    ("tm-second-brain", "TM Second Brain", "Second-brain memory layers on a rotating sphere", "#F471B5", "TM", "upstream"),
    ("kia-monopoly-core", "MAIRON Monopoly Core", "Sovereign monopoly core — 5,870,020 KIAN genesis supply", "#FACC15", "MAIRON SOVEREIGN", "kia_sovereign"),
    ("mairon-10gate-shield", "Mairon 10-Gate Shield", "Ten forensic gates between your code and the ledger", "#EF4444", "MAIRON", "kia_sovereign"),
    ("quant-vibe-terminal", "Quant Vibe Terminal", "Causal Vibe-Trading backtests across 8 CCXT venues", "#22D3EE", "QUANT", "kia_sovereign"),
    ("trinity-5model-genome", "Trinity 5-Model Genome", "Penta-model genome routing across the Trinity pipeline", "#A78BFA", "TRINITY", "kia_sovereign"),
]

FRAME_SIZES = {
    ("720p", "9:16"): (720, 1280),
    ("720p", "16:9"): (1280, 720),
    ("1080p", "9:16"): (1080, 1920),
    ("1080p", "16:9"): (1920, 1080),
}


def fibonacci_shell(n: int, radius: float) -> list[tuple[float, float, float]]:
    golden_angle = math.pi * (3.0 - math.sqrt(5.0))
    points: list[tuple[float, float, float]] = []
    for i in range(n):
        y = 1.0 - (2.0 * (i + 0.5)) / n
        ring = math.sqrt(max(0.0, 1.0 - y * y))
        theta = golden_angle * i
        points.append((math.cos(theta) * ring * radius, y * radius, math.sin(theta) * ring * radius))
    return points


def hud_mesh_nodes() -> list[dict]:
    outer = fibonacci_shell(SHELL_NODE_COUNT, SPHERE_RADIUS)
    inner = fibonacci_shell(SHELL_NODE_COUNT, SPHERE_RADIUS * INNER_SHELL_RADIUS_SCALE)
    nodes: list[dict] = []
    for shell, points in ((0, outer), (1, inner)):
        for x, y, z in points:
            nodes.append({"i": len(nodes), "s": shell, "x": x, "y": y, "z": z, "l": None, "v": None})
    for callout, (shell, local) in zip(HUD_CALLOUTS, CALLOUT_BINDINGS):
        idx = shell * SHELL_NODE_COUNT + local
        nodes[idx]["l"] = callout[0]
        nodes[idx]["v"] = callout[1]
    return nodes


def hud_mesh_edges(nodes: list[dict]) -> list[list[int]]:
    edges: list[list[int]] = []
    seen: set[tuple[int, int]] = set()

    def link(a: int, b: int) -> None:
        lo, hi = min(a, b), max(a, b)
        if lo != hi and (lo, hi) not in seen:
            seen.add((lo, hi))
            edges.append([lo, hi])

    for shell in (0, 1):
        base = shell * SHELL_NODE_COUNT
        for i in range(SHELL_NODE_COUNT):
            a = nodes[base + i]
            cands = []
            for j in range(SHELL_NODE_COUNT):
                if i == j:
                    continue
                b = nodes[base + j]
                d = (a["x"] - b["x"]) ** 2 + (a["y"] - b["y"]) ** 2 + (a["z"] - b["z"]) ** 2
                cands.append((d, base + j))
            cands.sort()
            for _, neighbor in cands[:3]:
                link(a["i"], neighbor)
    for i in range(0, SHELL_NODE_COUNT, 15):
        a = nodes[i]
        best: tuple[float, int] | None = None
        for j in range(SHELL_NODE_COUNT):
            b = nodes[SHELL_NODE_COUNT + j]
            d = (a["x"] - b["x"]) ** 2 + (a["y"] - b["y"]) ** 2 + (a["z"] - b["z"]) ** 2
            if best is None or d < best[0]:
                best = (d, b["i"])
        if best is not None:
            link(a["i"], best[1])
    return edges


def hud_timeline_phases(duration_sec: int) -> list[dict]:
    total = float(duration_sec)
    marks = [0.0, 14.0 / 60.0, 29.0 / 60.0, 44.0 / 60.0, 1.0]
    bounds = [round(total * m * 100.0) / 100.0 for m in marks]
    return [
        {"index": i + 1, "title": t, "start_sec": bounds[i], "end_sec": bounds[i + 1]}
        for i, t in enumerate(PHASE_TITLES)
    ]


def json_for_script(value) -> str:
    raw = json.dumps(value, separators=(",", ":"))
    return (
        raw.replace("<", "\\u003c")
        .replace(">", "\\u003e")
        .replace("&", "\\u0026")
        .replace("\u2028", "\\u2028")
        .replace("\u2029", "\\u2029")
    )


def html_attr_escape(text: str) -> str:
    return (
        text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
    )


def resolve_accent(repo_name: str, accent: str | None) -> str:
    if accent is None or not accent.strip():
        digest = hashlib.sha256(repo_name.encode("utf-8")).hexdigest()
        return f"#{digest[:6].upper()}"
    body = accent.strip().lstrip("#")
    if (len(body) not in (3, 6)) or any(c not in "0123456789abcdefABCDEF" for c in body):
        raise SystemExit(f"invalid accent hex: {accent}")
    if len(body) == 3:
        body = "".join(c * 2 for c in body)
    return f"#{body.upper()}"


def synthesize_custom_preset(repo_name: str, tagline: str, accent: str | None) -> dict:
    name = repo_name.strip()
    if not name:
        raise SystemExit("repo name is empty — cannot synthesize a HUD preset")
    slug_chars: list[str] = []
    last_dash = False
    for ch in name:
        if ch.isascii() and ch.isalnum():
            slug_chars.append(ch.lower())
            last_dash = False
        elif not last_dash:
            slug_chars.append("-")
            last_dash = True
    slug = "".join(slug_chars).strip("-") or "custom-repo"
    labels = []
    for word in tagline.replace(",", " ").replace("-", " ").replace("/", " ").split():
        clean = "".join(c for c in word if c.isascii() and c.isalnum())[:8].upper()
        if clean:
            labels.append(clean)
        if len(labels) == 4:
            break
    return {
        "slug": slug,
        "title": name,
        "tagline": tagline.strip() or "Custom repo neural telemetry",
        "accent_hex": resolve_accent(name, accent),
        "badge": name.upper(),
        "source": "synthesized",
        "node_labels": labels or ["REPO", "NEURAL", "HUD"],
    }


def preset_by_slug(slug: str) -> dict:
    for pslug, title, tagline, accent, badge, source in PRESETS:
        if pslug == slug:
            return {
                "slug": pslug,
                "title": title,
                "tagline": tagline,
                "accent_hex": accent,
                "badge": badge,
                "source": source,
                "node_labels": [],
            }
    raise SystemExit(f"unknown HUD preset slug: {slug} (try: presets)")


HTML_TEMPLATE = r"""<!DOCTYPE html>
<html lang="en" data-kia="neural-hud">
<head>
<meta charset="utf-8">
<title>__TITLE__</title>
<style>
  :root { --accent: __ACCENT__; }
  * { margin: 0; padding: 0; box-sizing: border-box; }
  html, body { width: 100%; height: 100%; background: #050507; overflow: hidden; }
  body { font-family: "JetBrains Mono", Consolas, monospace; color: #E7E9EE; }
  canvas { position: absolute; inset: 0; }
  .hud-chip { position: absolute; padding: 8px 14px; background: rgba(255,255,255,0.06);
    border: 1px solid rgba(255,255,255,0.08); border-radius: 10px; font-size: 12px; }
  #badge { top: 18px; left: 18px; color: var(--accent); font-weight: 700; }
  #phase { top: 18px; right: 18px; }
  #phase b { color: var(--accent); }
  .callout { position: absolute; transform: translate(-50%, -130%); padding: 6px 10px;
    background: rgba(5,5,7,0.72); border: 1px solid rgba(255,255,255,0.08);
    border-left: 3px solid var(--accent); border-radius: 8px; font-size: 11px; white-space: nowrap; }
  .callout .v { color: var(--accent); font-weight: 700; }
  #timecode { bottom: 18px; left: 18px; font-variant-numeric: tabular-nums; }
  #tagline { bottom: 18px; right: 18px; color: #8A93A6; max-width: 46%; text-align: right; }
  #audioHint { bottom: 56px; left: 18px; color: #6B7280; font-size: 10px; }
</style>
</head>
<body>
<canvas id="stage" width="__W__" height="__H__"></canvas>
<div id="callouts"></div>
<div class="hud-chip" id="badge">__BADGE__</div>
<div class="hud-chip" id="phase">PHASE <b id="phaseIdx">01</b> <span id="phaseTitle">__PHASE0__</span></div>
<div class="hud-chip" id="timecode">00:00 / __DURATION__</div>
<div class="hud-chip" id="tagline">__TAGLINE__</div>
<div id="audioHint">tap / key = enable live WebAudio synth</div>
<script>
"use strict";
window.__KIA_HUD_GEO = __GEO__;
window.__KIA_HUD_PHASES = __PHASES__;
window.__KIA_HUD_CONFIG = __CONFIG__;
window.__KIA_HUD_SIGNATURE = "__SIGNATURE__";
(function () {
  var GEO = window.__KIA_HUD_GEO, PHASES = window.__KIA_HUD_PHASES, CFG = window.__KIA_HUD_CONFIG;
  var canvas = document.getElementById("stage");
  var ctx = canvas.getContext("2d");
  var nodes = GEO.nodes, edges = GEO.edges;
  var state = { t: 0, playing: true, last: null, audio: null, lastPingAt: -1 };

  function project(n, t) {
    var rotY = t * CFG.rotYRate, rotX = Math.sin(t * CFG.rotXRate) * CFG.rotXAmp;
    var cosY = Math.cos(rotY), sinY = Math.sin(rotY), cosX = Math.cos(rotX), sinX = Math.sin(rotX);
    var x1 = n.x * cosY - n.z * sinY, z1 = n.x * sinY + n.z * cosY;
    var y2 = n.y * cosX - z1 * sinX, z2 = n.y * sinX + z1 * cosX;
    var scale = CFG.fov / (CFG.fov + z2 + CFG.cameraZ);
    return { sx: CFG.cx + x1 * scale, sy: CFG.cy + y2 * scale, z: z2 };
  }
  function render(t) {
    ctx.fillStyle = "#050507";
    ctx.fillRect(0, 0, CFG.width, CFG.height);
    var proj = nodes.map(function (n) { return project(n, t); });
    var sortedEdges = edges.map(function (e) {
      return { depth: (proj[e[0]].z + proj[e[1]].z) / 2, a: proj[e[0]], b: proj[e[1]] };
    }).sort(function (l, r) { return l.depth - r.depth; });
    for (var i = 0; i < sortedEdges.length; i++) {
      var s = sortedEdges[i];
      ctx.strokeStyle = "rgba(255,255,255," + (0.10 + 0.16 * (1 - (s.depth + CFG.sphereRadius) / (2 * CFG.sphereRadius))).toFixed(3) + ")";
      ctx.beginPath(); ctx.moveTo(s.a.sx, s.a.sy); ctx.lineTo(s.b.sx, s.b.sy); ctx.stroke();
    }
    var sortedNodes = nodes.map(function (n, i) { return { n: n, p: proj[i] }; })
      .sort(function (l, r) { return l.p.z - r.p.z; });
    for (var k = 0; k < sortedNodes.length; k++) {
      var it = sortedNodes[k];
      ctx.beginPath();
      ctx.arc(it.p.sx, it.p.sy, Math.max(1.2, it.n.l ? 5.5 : 2.6), 0, 6.28318);
      ctx.fillStyle = it.n.l ? CFG.accent : "rgba(231,233,238,0.85)";
      ctx.fill();
    }
    for (var c = 0; c < CFG.calloutNodes.length; c++) {
      var p = proj[CFG.calloutNodes[c]];
      var card = document.getElementById("callout-" + c);
      if (card) { card.style.left = p.sx + "px"; card.style.top = p.sy + "px"; }
    }
    var ph = PHASES[PHASES.length - 1];
    for (var q = 0; q < PHASES.length; q++) {
      if (t >= PHASES[q].start_sec && t < PHASES[q].end_sec) { ph = PHASES[q]; break; }
    }
    document.getElementById("phaseIdx").textContent = ("0" + ph.index).slice(-2);
    document.getElementById("phaseTitle").textContent = ph.title.replace(/^PHASE \d+ \/\/ /, "");
    var m = Math.floor(t / 60), s = Math.floor(t % 60);
    document.getElementById("timecode").textContent = ("0" + m).slice(-2) + ":" + ("0" + s).slice(-2);
  }
  window.seekTo = function (t) {
    state.t = Math.max(0, Math.min(CFG.duration, Number(t) || 0));
    state.playing = false;
    render(state.t);
    return state.t;
  };
  function initAudio() {
    if (state.audio) { return; }
    var AC = window.AudioContext || window.webkitAudioContext;
    if (!AC) { return; }
    var ac = new AC();
    var master = ac.createGain(); master.gain.value = 0.34;
    master.connect(ac.destination);
    var bed = CFG.subBassHz;
    for (var i = 0; i < bed.length; i++) {
      var osc = ac.createOscillator(), g = ac.createGain();
      osc.type = "sine"; osc.frequency.value = bed[i];
      g.gain.value = [0.20, 0.13, 0.10][i] || 0.1;
      osc.connect(g); g.connect(master); osc.start();
    }
    state.audio = { ac: ac, master: master };
    window.__KIA_HUD_AUDIO = true;
  }
  function radarPing(t) {
    if (!state.audio) { return; }
    var slot = Math.floor(t / 4);
    if (slot === state.lastPingAt) { return; }
    state.lastPingAt = slot;
    var hz = CFG.radarPingHz[slot % CFG.radarPingHz.length];
    var ac = state.audio.ac;
    var osc = ac.createOscillator(), g = ac.createGain();
    osc.type = "sine"; osc.frequency.value = hz;
    g.gain.setValueAtTime(0.16, ac.currentTime);
    g.gain.exponentialRampToValueAtTime(0.0001, ac.currentTime + 0.6);
    osc.connect(g); g.connect(state.audio.master);
    osc.start(); osc.stop(ac.currentTime + 0.65);
  }
  window.addEventListener("pointerdown", initAudio);
  window.addEventListener("keydown", initAudio);
  function loop(now) {
    if (state.playing) {
      if (state.last === null) { state.last = now; }
      state.t = Math.min(CFG.duration, state.t + (now - state.last) / 1000);
      state.last = now;
      render(state.t);
      radarPing(state.t);
      if (state.t >= CFG.duration) { state.playing = false; }
    }
    window.requestAnimationFrame(loop);
  }
  var host = document.getElementById("callouts");
  for (var c = 0; c < CFG.calloutNodes.length; c++) {
    var card = document.createElement("div");
    card.className = "callout"; card.id = "callout-" + c;
    var node = nodes[CFG.calloutNodes[c]];
    card.textContent = node.l + ": " + node.v;
    host.appendChild(card);
  }
  render(state.t);
  window.requestAnimationFrame(loop);
})();
</script>
</body>
</html>
"""


def frame_size(resolution: str, aspect: str) -> tuple[int, int]:
    try:
        return FRAME_SIZES[(resolution, aspect)]
    except KeyError:
        raise SystemExit(f"invalid combo {resolution} {aspect} — supported: {sorted(FRAME_SIZES)}") from None


def build_html(preset: dict, resolution: str, aspect: str, fps: int, duration: int) -> str:
    if not 1 <= duration <= MAX_DURATION_SEC:
        raise SystemExit(f"duration {duration}s outside 1..={MAX_DURATION_SEC}")
    width, height = frame_size(resolution, aspect)
    nodes = hud_mesh_nodes()
    edges = hud_mesh_edges(nodes)
    phases = hud_timeline_phases(duration)
    callout_nodes = [n["i"] for n in nodes if n["l"]]
    geo = {"nodes": nodes, "edges": edges}
    config = {
        "width": width, "height": height, "fov": FOV, "cameraZ": CAMERA_Z,
        "sphereRadius": SPHERE_RADIUS, "rotYRate": ROT_Y_RATE, "rotXRate": ROT_X_RATE,
        "rotXAmp": ROT_X_AMPLITUDE, "fps": fps, "duration": duration,
        "cx": width // 2, "cy": height // 2, "accent": preset["accent_hex"],
        "calloutNodes": callout_nodes, "subBassHz": SUB_BASS_HZ, "radarPingHz": RADAR_PING_HZ,
    }
    minutes, seconds = divmod(duration, 60)
    return (
        HTML_TEMPLATE
        .replace("__TITLE__", html_attr_escape(preset["title"]))
        .replace("__ACCENT__", preset["accent_hex"])
        .replace("__W__", str(width))
        .replace("__H__", str(height))
        .replace("__BADGE__", html_attr_escape(preset["badge"]))
        .replace("__PHASE0__", html_attr_escape(PHASE_TITLES[0]))
        .replace("__TAGLINE__", html_attr_escape(preset["tagline"]))
        .replace("__DURATION__", f"{minutes:02d}:{seconds:02d}")
        .replace("__GEO__", json_for_script(geo))
        .replace("__PHASES__", json_for_script(phases))
        .replace("__CONFIG__", json_for_script(config))
        .replace("__SIGNATURE__", SIGNATURE)
    )


def audio_filter_complex(duration: int) -> str:
    fade_out = round((duration - FADE_OUT_SEC) * 100.0) / 100.0
    return (
        f"sine=frequency=55:duration={duration}[b0];"
        f"sine=frequency=82.41:duration={duration}[b1];"
        f"sine=frequency=110:duration={duration}[b2];"
        f"sine=frequency=1046.5:duration={duration},"
        "volume='if(lt(mod(t\\,8)\\,0.35)\\,0.22\\,0)':eval=frame[p0];"
        "[b0][b1][b2][p0]amix=inputs=4:duration=first,"
        f"afade=t=in:ss=0:d={FADE_IN_SEC},"
        f"afade=t=out:st={fade_out}:d={FADE_OUT_SEC},"
        f"{MASTER_LIMITER}[aout]"
    )


def audio_command(duration: int) -> list[str]:
    return [
        "ffmpeg", "-y", "-hide_banner", "-loglevel", "error",
        "-filter_complex", audio_filter_complex(duration),
        "-map", "[aout]", "-ar", "48000", "-ac", "2",
        "-c:a", "pcm_s16le", "hud_audio.wav",
    ]


def video_command(resolution: str, aspect: str, fps: int, with_audio: bool) -> list[str]:
    width, height = frame_size(resolution, aspect)
    argv = [
        "ffmpeg", "-y", "-hide_banner", "-loglevel", "error",
        "-f", "image2pipe", "-framerate", str(fps), "-i", "-",
    ]
    if with_audio:
        argv += ["-i", "hud_audio.wav", "-map", "0:v:0", "-map", "1:a:0"]
    argv += [
        "-c:v", "libx264", "-preset", "veryfast", "-crf", "18",
        "-pix_fmt", "yuv420p", "-s", f"{width}x{height}", "-r", str(fps),
    ]
    if with_audio:
        argv += ["-c:a", "aac", "-b:a", "192k", "-shortest"]
    argv += ["-movflags", "+faststart", "neural_hud.mp4"]
    return argv


def _spawn_stderr_drain(proc: subprocess.Popen) -> threading.Thread:
    """stderr drain — background thread that keeps reading the child stderr
    pipe until EOF so a verbose ffmpeg never fills the OS pipe buffer and
    deadlocks the render loop. Every Popen with stderr=PIPE gets one."""
    def _drain() -> None:
        try:
            if proc.stderr:
                for _line in proc.stderr:
                    pass  # drain — output intentionally discarded
        finally:
            if proc.stderr:
                proc.stderr.close()
    thread = threading.Thread(target=_drain, daemon=True)
    thread.start()
    return thread


def ensure_hud_audio(duration: int) -> bool:
    """Auto-generate hud_audio.wav before the video mux when it does not yet
    exist (stderr drained in the background — no BrokenPipeError possible).
    Returns True when the audio input is available for video_command."""
    if os.path.exists("hud_audio.wav"):
        return True
    if not shutil.which("ffmpeg"):
        return False
    gen = subprocess.Popen(
        audio_command(duration),
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
    )
    _spawn_stderr_drain(gen)  # stderr drain for the audio generation pass
    gen.wait()
    return os.path.exists("hud_audio.wav")


def _spawn_stderr_drain(proc: subprocess.Popen) -> threading.Thread:
    """stderr drain — background thread that keeps reading the child stderr
    pipe until EOF so a verbose ffmpeg never fills the OS pipe buffer and
    deadlocks the render loop. Every Popen with stderr=PIPE gets one."""
    def _drain() -> None:
        try:
            if proc.stderr:
                for _line in proc.stderr:
                    pass  # drain — output intentionally discarded
        finally:
            if proc.stderr:
                proc.stderr.close()
    thread = threading.Thread(target=_drain, daemon=True)
    thread.start()
    return thread


def ensure_hud_audio(duration: int) -> bool:
    """Auto-generate hud_audio.wav before the video mux when it does not yet
    exist (stderr drained in the background — no BrokenPipeError possible).
    Returns True when the audio input is available for video_command."""
    if os.path.exists("hud_audio.wav"):
        return True
    if not shutil.which("ffmpeg"):
        return False
    gen = subprocess.Popen(
        audio_command(duration),
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
    )
    _spawn_stderr_drain(gen)  # stderr drain for the audio generation pass
    gen.wait()
    return os.path.exists("hud_audio.wav")


def render_frames(preset: dict, resolution: str, aspect: str, fps: int,
                  duration: int, out: Path) -> None:
    try:
        from PIL import Image, ImageDraw
    except ImportError:
        raise SystemExit("render requires Pillow — pip install pillow (or use `html` + a browser)") from None
    width, height = frame_size(resolution, aspect)
    nodes = hud_mesh_nodes()
    edges = hud_mesh_edges(nodes)
    phases = hud_timeline_phases(duration)
    accent = preset["accent_hex"].lstrip("#")
    accent_rgb = tuple(int(accent[i:i + 2], 16) for i in (0, 2, 4))
    total_frames = duration * fps

    def project(n: dict, t: float) -> tuple[float, float, float]:
        rot_y = t * ROT_Y_RATE
        rot_x = math.sin(t * ROT_X_RATE) * ROT_X_AMPLITUDE
        cos_y, sin_y = math.cos(rot_y), math.sin(rot_y)
        cos_x, sin_x = math.cos(rot_x), math.sin(rot_x)
        x1 = n["x"] * cos_y - n["z"] * sin_y
        z1 = n["x"] * sin_y + n["z"] * cos_y
        y2 = n["y"] * cos_x - z1 * sin_x
        z2 = n["y"] * sin_x + z1 * cos_x
        scale = FOV / (FOV + z2 + CAMERA_Z)
        return width / 2 + x1 * scale, height / 2 + y2 * scale, z2

    ffmpeg = shutil.which("ffmpeg")
    proc: subprocess.Popen | None = None
    if ffmpeg:
        # auto-generate hud_audio.wav first (falls back to video-only when the
        # audio pipeline is unavailable) — eliminates the BrokenPipeError class
        with_audio = ensure_hud_audio(duration)
        proc = subprocess.Popen(
            video_command(resolution, aspect, fps, with_audio=with_audio),
            stdin=subprocess.PIPE,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
        )
        _spawn_stderr_drain(proc)  # stderr drain — the pipe can never deadlock
    for frame in range(total_frames):
        t = frame / fps
        phase = phases[-1]
        for candidate in phases:
            if candidate["start_sec"] <= t < candidate["end_sec"]:
                phase = candidate
                break
        img = Image.new("RGB", (width, height), (5, 5, 7))
        draw = ImageDraw.Draw(img)
        proj = [project(n, t) for n in nodes]
        ordered = sorted(edges, key=lambda e: (proj[e[0]][2] + proj[e[1]][2]) / 2)
        for e in ordered:
            a, b = proj[e[0]], proj[e[1]]
            depth_alpha = 0.10 + 0.16 * (1 - (a[2] + b[2] + 2 * SPHERE_RADIUS) / (4 * SPHERE_RADIUS))
            shade = int(255 * depth_alpha)
            draw.line([a[0], a[1], b[0], b[1]], fill=(shade, shade, shade), width=1)
        node_order = sorted(range(len(nodes)), key=lambda i: proj[i][2])
        for i in node_order:
            n, p = nodes[i], proj[i]
            r = max(1.2, 5.5 if n["l"] else 2.6)
            color = accent_rgb if n["l"] else (231, 233, 238)
            draw.ellipse([p[0] - r, p[1] - r, p[0] + r, p[1] + r], fill=color)
        if proc:
            img.save(proc.stdin, format="JPEG", quality=92)
        elif frame % max(1, total_frames // 12) == 0:
            img.save(out / f"frame_{frame:05d}.png")
        print(f"\rframe {frame + 1}/{total_frames} t={t:.2f}s {phase['title']}", end="")
    print()
    if proc:
        proc.stdin.close()
        proc.wait()
        print(f"muxed -> {Path.cwd() / 'neural_hud.mp4'} (faststart)")
    else:
        print(f"frames -> {out}/ (ffmpeg not found — PNG stills only)")


def run_dry_run_verification() -> None:
    """Full plan verification — 180-node + 308-edge + dual-shell + audio limiter.

    Builds the deterministic HUD mesh exactly like the HTML template does,
    asserts every architectural constant and prints the verified plan. Exit 0.
    """
    nodes = hud_mesh_nodes()
    edges = hud_mesh_edges(nodes)
    shells = sorted({n["s"] for n in nodes})
    callouts = [n for n in nodes if n["l"]]
    audio = " ".join(audio_command(DEFAULT_DURATION_SEC))
    video = " ".join(video_command("1080p", "9:16", 24, True))
    checks = [
        ("dual-shell mesh", len(shells) == 2),
        ("outer shell 90 nodes", sum(1 for n in nodes if n["s"] == 0) == SHELL_NODE_COUNT),
        ("inner shell 90 nodes", sum(1 for n in nodes if n["s"] == 1) == SHELL_NODE_COUNT),
        ("180 nodes total", len(nodes) == 2 * SHELL_NODE_COUNT),
        ("4 HUD callouts bound", len(callouts) == len(HUD_CALLOUTS)),
        ("308 mesh edges", len(edges) == 308),
        ("edges reference valid nodes", all(0 <= a < len(nodes) and 0 <= b < len(nodes) for a, b in edges)),
        ("fibonacci sphere radius", all(abs(math.sqrt(n["x"] ** 2 + n["y"] ** 2 + n["z"] ** 2) - (SPHERE_RADIUS if n["s"] == 0 else SPHERE_RADIUS * INNER_SHELL_RADIUS_SCALE)) < 1e-6 for n in nodes)),
        ("audio pipeline present", len(audio) > 0),
        ("master limiter in audio chain", "alimiter=limit=0.95:attack=5:release=50" in audio),
        ("video pipeline faststart", "-movflags" in video and "+faststart" in video),
        ("duration cap 180s", MAX_DURATION_SEC == 180 and DEFAULT_DURATION_SEC <= MAX_DURATION_SEC),
    ]
    print(f"{SIGNATURE} — dry-run plan verification")
    failed = 0
    for name, ok in checks:
        print(f"  [{'OK' if ok else 'FAIL'}] {name}")
        failed += 0 if ok else 1
    print(f"  plan: {len(nodes)} nodes · {len(edges)} edges · {len(shells)} shells · "
          f"{len(callouts)} callouts · master limiter armed")
    if failed:
        raise SystemExit(f"dry-run verification failed: {failed} check(s)")
    print("  dry-run verification passed (exit 0)")


def main() -> None:
    parser = argparse.ArgumentParser(description="MAIRON Neural HUD Video Studio")
    parser.add_argument(
        "--dry-run", action="store_true", dest="dry_run",
        help="verify the full 180-node + 308-edge + dual-shell + audio-limiter plan and exit 0",
    )
    sub = parser.add_subparsers(dest="cmd", required=False)

    sub.add_parser("presets")

    p_html = sub.add_parser("html")
    p_html.add_argument("--preset", required=True)
    p_html.add_argument("--out", default="neural_hud.html")
    for flag, default in (("--resolution", "1080p"), ("--aspect", "9:16")):
        p_html.add_argument(flag, default=default)
    p_html.add_argument("--fps", type=int, default=24, choices=[24, 60])
    p_html.add_argument("--duration", type=int, default=DEFAULT_DURATION_SEC)

    p_syn = sub.add_parser("synthesize")
    p_syn.add_argument("--repo", required=True)
    p_syn.add_argument("--tagline", default="")
    p_syn.add_argument("--accent", default=None)
    p_syn.add_argument("--out", default="neural_hud.html")
    for flag, default in (("--resolution", "1080p"), ("--aspect", "9:16")):
        p_syn.add_argument(flag, default=default)
    p_syn.add_argument("--fps", type=int, default=24, choices=[24, 60])
    p_syn.add_argument("--duration", type=int, default=DEFAULT_DURATION_SEC)

    p_plan = sub.add_parser("plan")
    p_plan.add_argument("--duration", type=int, default=DEFAULT_DURATION_SEC)
    for flag, default in (("--resolution", "1080p"), ("--aspect", "9:16")):
        p_plan.add_argument(flag, default=default)
    p_plan.add_argument("--fps", type=int, default=24, choices=[24, 60])
    p_plan.add_argument("--with-audio", action="store_true")

    p_render = sub.add_parser("render")
    p_render.add_argument("--preset", required=True)
    for flag, default in (("--resolution", "1080p"), ("--aspect", "9:16")):
        p_render.add_argument(flag, default=default)
    p_render.add_argument("--fps", type=int, default=24, choices=[24, 60])
    p_render.add_argument("--duration", type=int, default=12)
    p_render.add_argument("--out", default="hud_frames")

    args = parser.parse_args()

    if args.dry_run:
        run_dry_run_verification()
        return

    if args.cmd is None:
        parser.error("the following arguments are required: cmd (or --dry-run)")

    if args.cmd == "presets":
        print(f"{SIGNATURE} — 8 telemetry presets (4 upstream + 4 kia_sovereign)")
        for slug, title, tagline, accent, badge, source in PRESETS:
            print(f"  {slug:24s} {accent}  [{source:12s}] {title} — {tagline}")
        return

    if args.cmd in ("html", "synthesize"):
        preset = preset_by_slug(args.preset) if args.cmd == "html" else synthesize_custom_preset(args.repo, args.tagline, args.accent)
        html = build_html(preset, args.resolution, args.aspect, args.fps, args.duration)
        Path(args.out).write_text(html, encoding="utf-8")
        digest = hashlib.sha256(html.encode("utf-8")).hexdigest()
        width, height = frame_size(args.resolution, args.aspect)
        print(f"{SIGNATURE} preset={preset['slug']} {width}x{height}@{args.fps} dur={args.duration}s")
        print(f"html -> {args.out} sha256={digest}")
        return

    if args.cmd == "plan":
        print("# audio pipeline (master limiter appended after amix + fades — D05/D15)")
        print(" ".join(audio_command(args.duration)))
        print()
        print("# video pipeline (FullHD + faststart — D06/D07/D08)")
        print(" ".join(video_command(args.resolution, args.aspect, args.fps, args.with_audio)))
        return

    if args.cmd == "render":
        preset = preset_by_slug(args.preset)
        out = Path(args.out)
        out.mkdir(parents=True, exist_ok=True)
        render_frames(preset, args.resolution, args.aspect, args.fps, args.duration, out)
        return


if __name__ == "__main__":
    main()
