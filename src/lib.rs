//! `podtrack` — rastreador de la parrilla y del ciclo de vida de los episodios.
//!
//! Este crate es un **port de paridad** de `podcast-tracker.py`: reproduce su
//! salida de texto byte a byte y sus *exit codes* para la superficie de lectura
//! (`status`, `next`, `next-date`, `list`, `list-range`, `get`, `where`), y añade
//! una salida `--json` opt-in y versionada. La lógica vive aquí (librería) para
//! que los tests de integración y unitarios puedan ejercitarla; el binario
//! (`main.rs`) es solo la CLI.

pub mod commands;
pub mod error;
pub mod json;
pub mod material;
pub mod model;
pub mod profile;
pub mod rebase;
pub mod tracker;
