<div align="center">

# ⚒️ KIAN v2 — Núcleo soberano de agentes de IA, enrutador genomio multi-modelo y mercado de doble confianza

**Edición alojada en OneRing.** Monorepo Rust empresarial sin `unsafe` combinado con un frontend
Next.js / Telegram Mini-App: una flota autónoma de agentes de 7 roles, enrutamiento multi-modelo
sin claves, un estudio de productos HTML con IA, una microcontabilidad de doble entrada con
escrow bloqueado por CAS y el motor de mercado de doble confianza `Mairon REA + 5 estrellas bayesianas`.

[![Rust Toolchain](https://img.shields.io/badge/rustc-1.100.0--pinned-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![Workspace Tests](https://img.shields.io/badge/tests-779_passed_%7C_0_failed-success.svg)](#verificación)
[![Mairon REA Audit](https://img.shields.io/badge/Mairon_REA_v6.10.1-92%2F100_PASS-blue.svg)](#verificación)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](./LICENSE)

**Español** | [English](README.md) | [فارسی](README.fa.md) | [中文](README.zh-CN.md)

</div>

## Visión general

KIAN v2 es un núcleo soberano de agentes de IA y un mercado para creadores, con un corazón de
**11 crates de Rust** (todos con `#![forbid(unsafe_code)]`), contratos TypeScript, un SDK y tres
aplicaciones frontend (Web, Telegram Mini-App, panel de administración). El repositorio incluye
la base de datos semilla de producción, contratos Solidity, el ADN del conocimiento (genomas) y
toda la documentación de linaje evolutivo.

## Arquitectura

| Capa | Ubicación |
| :--- | :--- |
| Núcleo Rust — flota de agentes, malla de IA, contabilidad, motor de mercado | `crates/` (11 crates, `#![forbid(unsafe_code)]`) |
| Web · Mini-App · Admin (Next.js 16, pnpm + Turborepo) | `apps/` |
| Paquetes TypeScript compartidos (contratos, SDK, UI) | `packages/` |
| Constitución ADN del creador y genomas de modelos | `KIA-BRAIN-OS/` · `OMNI-MODEL-GENOME/` |
| Base de datos semilla de producción — 325 habilidades, 466 ciudadanos, libro balanceado | `seed/kian-production.db` |
| Evidencia del control de calidad REA (92/100, 0 fallos duros) | `REA_FINAL_VERDICT.json` · `docs/rea/` |
| Wave 10 Sala de Guerra de Fundadores (puente de mando OLED de 4 cuadrantes) | `tools/war-room/war_room.html` |

## Verificación (desde la raíz del repositorio)

```bash
cargo clippy --workspace --all-targets -- -D warnings   # 0 advertencias de lint
cargo test --workspace                                   # 779 aprobadas / 0 fallidas
cat REA_FINAL_VERDICT.json                               # 92/100 PASS, hard_failures: []
```

## Aspectos destacados

- **Wave 10 «Sala de Guerra de Fundadores»**: Elon Musk, CZ y Jeff Bezos como cofundadores
  24/7 remotos bajo la presidencia de `0xKIA`, monetizando desde el segundo 0 del Día 0 con un
  motor de simulación tick a tick honesto — carta de reparto 20/80 (Base 8453 · USDC ·
  Telegram Stars XTR), sin saltos de tiempo ficticios.
- **Malla de inferencia multi-modelo sin claves**: matriz ADN de 5 modelos × 9 archivos
  (`OMNI-MODEL-GENOME`).
- **Contabilidad de grado bancario**: doble entrada, escrow con bloqueo CAS, disciplina de
  linaje (`SUM(amount) == 0`).
- **Mairon REA**: puerta de calidad del producto en tiempo real + reseñas bayesianas de 5
  estrellas de la comunidad; puntuación actual 92/100.

## Licencia

MIT — véase [`LICENSE`](./LICENSE).
