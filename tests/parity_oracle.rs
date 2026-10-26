//! Guardián de paridad congelada: compara, comando a comando, la salida del
//! binario con el **golden congelado** (`tests/golden/parity_oracle/*.json`)
//! sobre el **mismo fixture** (texto + exit codes).
//!
//! El golden se grabó **una sola vez** con el oráculo recuperado del histórico de
//! `~/.agents` (commit `c70287f^`), que ya no existe: el oráculo es el **origen**
//! del golden, no el contrato vivo. La paridad **no es en vivo** y el golden **no
//! se regenera solo** ni se generó nunca desde el binario; es evidencia histórica y
//! candado contra regresiones.

mod common;

use std::path::Path;
use std::process::Command;

use common::{Fixture, MINI_YAML, MINI_YAML_V2};

struct Outcome {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run_podtrack(fx: &Fixture, harness_dir: &Path, args: &[&str]) -> Outcome {
    let output = Command::new(env!("CARGO_BIN_EXE_podtrack"))
        .args(args)
        .env("PODTRACK_SHOWS_DIR", &fx.shows)
        .env("PODTRACK_SHOW", &fx.slug)
        .env("HOME", harness_dir)
        .env_remove("XDG_CONFIG_HOME")
        .output()
        .expect("podtrack");
    Outcome {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

/// Especificaciones deterministas (sin dependencia del reloj de pared).
// `status` se omite a propósito desde el change `status-upcoming-window`:
// su ventana final diverge del oráculo por diseño (último publicado + 7 próximos
// en vez de los 8 últimos por parrilla). El resto de comandos siguen en paridad.
//
// `get 100` y `get 101` se omiten **solo en el test del fixture `mini`** desde
// `close-the-port`: el binario imprime `Slug:` y `Ruta:` desde el change archivado
// `2026-10-25-get-shows-slug-and-path` (commit `c7a15e9`) y el oráculo recuperado
// (`c70287f^`) no lo hacía. Esas dos líneas están cubiertas a mano en
// `tests/parity_read.rs` (líneas 154-275). La matriz es una **constante compartida**
// con el fixture `v2`, donde `get 100`/`get 101` **sí están en paridad** (allí esos
// episodios no existen y la salida es «no encontrado»), así que la exclusión se pasa
// por test a `assert_parity` en vez de quitar entradas de `SPECS`.
const SPECS: &[&[&str]] = &[
    &["next"],
    &["list"],
    &["list", "--status", "published"],
    &["list", "--season", "8", "--last", "2"],
    &["list-range", "100", "102"],
    &["get", "100"],
    &["get", "101"],
    &["get", "999"],
    &["where", "100"],
    &["where", "no-existe"],
];

fn assert_parity(fx: &Fixture, golden: &str, excluded: &[&str]) {
    // La matriz de este test es `SPECS` menos los escenarios excluidos (por etiqueta, el
    // `argv` unido por espacios): los índices del golden corresponden a la matriz **filtrada**.
    let specs: Vec<&[&str]> = SPECS
        .iter()
        .copied()
        .filter(|spec| !excluded.contains(&spec.join(" ").as_str()))
        .collect();

    for (idx, spec) in specs.iter().enumerate() {
        let expected = common::golden::case(golden, idx);
        let actual = run_podtrack(fx, fx.dir.path(), spec);
        expected.assert_outcome(fx.dir.path(), actual.code, &actual.stdout, &actual.stderr);
    }
}

#[test]
fn paridad_oraculo_fixture_mini() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    assert_parity(
        &fx,
        "parity_oracle/paridad_oraculo_fixture_mini",
        &["get 100", "get 101"],
    );
}

#[test]
fn paridad_oraculo_fixture_v2() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    assert_parity(&fx, "parity_oracle/paridad_oraculo_fixture_v2", &[]);
}
