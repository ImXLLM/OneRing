# TDesktop Telegram gems for KIAN v2

This package contains a sparse source extraction and small KIAN adapter examples.

- `01_tdesktop_source/`: seven audited Telegram Desktop C++ source files, upstream `LEGAL`, and `SOURCES.txt` mapping flattened filenames to their repository paths.
- `02_kian_v2_adapters/kian_telegram_bot_api9.rs`: dependency-free Rust Mini App initData HMAC validation, strict `p_<productId>_r_<refTgId>` parsing, and Stars invoice payload types.
- `02_kian_v2_adapters/kian-tma-bridge.ts`: zero-runtime-dependency browser helper for safe areas, optional haptics, swipe behavior, and referral startapp/share links.
- `02_kian_v2_adapters/inject_tdesktop_tma_skills.py`: idempotent SQLite injector for the two requested `KbEntry` keys.
- `tdesktop_kian_v2_audit.md`: source-linked audit notes and KIAN integration recommendations.

## Checks performed

Rust tests:

```sh
rustc --edition=2021 --test 02_kian_v2_adapters/kian_telegram_bot_api9.rs -o /tmp/kian_telegram_bot_api9_tests
/tmp/kian_telegram_bot_api9_tests
```

All 8 tests passed with rustc 1.85.1. The TypeScript helper passed strict `tsc --noEmit` checking with TypeScript 5.9.3. The SQLite injector passed syntax checking and a two-run idempotence test against a temporary `KbEntry` table.

The KIAN production database was not present, so the injector was tested on a temporary schema only. From a KIAN repo root, run it explicitly against the real database:

```sh
python3 02_kian_v2_adapters/inject_tdesktop_tma_skills.py --db seed/kian-production.db
```

The script refuses to create a missing database and refuses unknown required columns rather than guessing.
