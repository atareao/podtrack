//! `podtrack` — rastreador de la parrilla y del ciclo de vida de los episodios.
//!
//! Este crate **fue** el port de paridad de `podcast-tracker.py`: se verificó
//! byte a byte que reproduce su salida de texto y sus *exit codes* para la
//! superficie de lectura (`status`, `next`, `next-date`, `list`, `list-range`,
//! `get`, `where`), y esa paridad se conserva hoy como golden congelado en
//! `tests/golden/`, grabado una sola vez con el oráculo y no regenerable. La
//! salida `--json` sigue siendo opt-in y versionada. La lógica vive aquí
//! (librería) para que los tests de integración y unitarios puedan ejercitarla;
//! el binario (`main.rs`) es solo la CLI.

pub mod commands;
pub mod error;
pub mod json;
pub mod material;
pub mod model;
pub mod profile;
pub mod rebase;
pub mod srt_stage;
pub mod tracker;
