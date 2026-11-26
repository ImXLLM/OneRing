//! Build script for `kian-db` — tracks the migrations directory for the
//! `sqlx::migrate!()`.
//!
//! `sqlx::migrate!()` macro: it embeds the contents of `migrations/*.sql` at macro
//! expansion time, but cargo does not track the migrations directory itself in the incremental
//! graph; without this line, adding a file or changing a migration slips past the expansion
//! cache and a stale schema would be embedded. This script closes that compatibility
//! hole (fail-closed for every fresh migration).

fn main() {
    println!("cargo:rerun-if-changed=migrations");
    println!("cargo:rerun-if-changed=build.rs");
}
