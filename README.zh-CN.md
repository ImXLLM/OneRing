<div align="center">

# ⚒️ KIAN v2 — 主权 AI 智能体内核、多模型基因组路由器与双信任市场

**OneRing 单仓托管版。** 一个企业级、零 `unsafe` 的 Rust 单体仓库（monorepo），与 Next.js /
Telegram Mini-App 前端结合：包含 7 角色自主智能体舰队、免密钥多模型 AI 路由、AI HTML 产品工作室、
带 CAS 锁定托管的复式微型账本，以及 `Mairon REA + 5 星贝叶斯` 双信任市场引擎。

[![Rust Toolchain](https://img.shields.io/badge/rustc-1.100.0--pinned-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![Workspace Tests](https://img.shields.io/badge/tests-784_passed_%7C_0_failed-success.svg)](#验证)
[![Mairon REA Audit](https://img.shields.io/badge/Mairon_REA_v6.10.1-92%2F100_PASS-blue.svg)](#验证)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](./LICENSE)

**中文** | [English](README.md) | [فارسی](README.fa.md) | [Español](README.es.md)

</div>

## 项目概览

KIAN v2 是一个完整的 sovereign AI agent 内核与创作者市场，以 **11 个 Rust crate**
（全部 `#![forbid(unsafe_code)]`）为核心，配合 TypeScript 契约、SDK 与三个前端应用
（Web、Telegram Mini-App、管理后台）。仓库内含生产种子数据库、Solidity 合约、
知识库（DNA 基因组）与全套演进谱系文档。

## 架构速览

| 层 | 位置 |
| :--- | :--- |
| Rust 内核 —— 智能体舰队、AI 网格、账本、市场引擎 | `crates/`（11 个 crate，`#![forbid(unsafe_code)]`） |
| Web · Mini-App · 管理（Next.js 16，pnpm + Turborepo） | `apps/` |
| 共享 TypeScript 包（契约、SDK、UI） | `packages/` |
| 创作者 DNA 宪法与模型基因组 | `KIA-BRAIN-OS/` · `OMNI-MODEL-GENOME/` |
| 生产种子数据库 —— 334 技能、466 公民、平衡账本 | `seed/kian-production.db` |
| REA 质量门证据（92/100，0 硬失败） | `REA_FINAL_VERDICT.json` · `docs/rea/` |
| Wave 10 创始人作战室（四象限 OLED 指挥甲板） | `tools/war-room/war_room.html` |

## 验证（仓库根目录执行）

```bash
cargo clippy --workspace --all-targets -- -D warnings   # 0 lint 警告
cargo test --workspace                                   # 784 通过 / 0 失败
cat REA_FINAL_VERDICT.json                               # 92/100 PASS, hard_failures: []
```

## 亮点

- **Wave 10「创始人作战室」**：Elon Musk、CZ、Jeff Bezos 三位联合创始人在
  `0xKIA` 主席之下，从 Day 0 第 0 秒开始逐 tick 真实推进的 24/7 远程变现模拟引擎 ——
  严格的 20/80 分账章程（Base 8453 · USDC · Telegram Stars XTR），零虚构时间跳跃。
- **免密钥多模型推理网格**：`OMNI-MODEL-GENOME` 5 模型 × 9 文件 DNA 矩阵。
- **银行级账本**：复式记账、CAS 锁定托管、谱系纪律（`SUM(amount) == 0`）。
- **Mairon REA**：实时产品质量门 + 5 星贝叶斯社区评审，当前得分 92/100。

## 许可证

MIT —— 详见 [`LICENSE`](./LICENSE)。
