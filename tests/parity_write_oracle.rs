//! Guardián de paridad **congelada** de la **escritura**: compara, comando a
//! comando, la salida **y el tracker/material resultante** del binario con el
//! **golden** (`tests/golden/parity_write_oracle/*.json`).
//!
//! El golden se grabó **una sola vez** con el oráculo recuperado del histórico de
//! `~/.agents` (commit `c70287f^`), que **ya no existe**; el oráculo es el **origen**
//! del golden, no el contrato vivo. La paridad **no es en vivo** y el golden **no se
//! regenera solo** ni se generó nunca desde el binario: es evidencia histórica y
//! candado contra regresiones. Un caso ausente falla nombrando su fichero (ver
//! `common::golden`).

mod common;

use std::process::Command as ProcessCommand;

use common::{B4_YAML, Fixture, MINI_YAML, MINI_YAML_V2, build_material};

struct Outcome {
    code: i32,
    stdout: String,
    stderr: String,
}

fn reset(fx: &Fixture, yaml: &str, setup: &dyn Fn(&Fixture)) {
    let _ = std::fs::remove_dir_all(&fx.root);
    std::fs::create_dir_all(&fx.root).expect("mkdir root");
    std::fs::write(&fx.tracker, yaml).expect("write tracker");
    setup(fx);
}

fn run_podtrack(fx: &Fixture, args: &[&str]) -> Outcome {
    let output = ProcessCommand::new(env!("CARGO_BIN_EXE_podtrack"))
        .args(args)
        .env("PODTRACK_SHOWS_DIR", &fx.shows)
        .env("PODTRACK_SHOW", &fx.slug)
        .env("HOME", fx.dir.path())
        .env_remove("XDG_CONFIG_HOME")
        .output()
        .expect("podtrack");
    Outcome {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn assert_case_with(golden: &str, idx: usize, yaml: &str, args: &[&str], setup: &dyn Fn(&Fixture)) {
    let fx = Fixture::new(yaml, "demo", "Show de prueba");

    let expected = common::golden::case(golden, idx);

    reset(&fx, yaml, setup);
    let lib = run_podtrack(&fx, args);
    let lib_tracker = std::fs::read(&fx.tracker).expect("read tracker");
    let lib_material = common::material_snapshot(&fx.root);

    expected.assert_outcome(fx.dir.path(), lib.code, &lib.stdout, &lib.stderr);
    expected.assert_tracker(&lib_tracker);
    expected.assert_material(&lib_material);
}

/// Como `assert_case_with`, con el material numerado estándar.
fn assert_case(golden: &str, idx: usize, yaml: &str, args: &[&str]) {
    assert_case_with(golden, idx, yaml, args, &|fx| build_material(&fx.root));
}

// ─── add ────────────────────────────────────────────────────────────

#[test]
fn oraculo_add_numero_explicito() {
    assert_case(
        "parity_write_oracle/oraculo_add_numero_explicito",
        0,
        MINI_YAML,
        &["--show", "demo", "add", "103", "Cuarto"],
    );
}

#[test]
fn oraculo_add_sin_numero() {
    assert_case(
        "parity_write_oracle/oraculo_add_sin_numero",
        0,
        MINI_YAML,
        &["--show", "demo", "add", "Cuarto sin número"],
    );
}

#[test]
fn oraculo_add_slug_explicito_y_derivado() {
    assert_case(
        "parity_write_oracle/oraculo_add_slug_explicito_y_derivado",
        0,
        MINI_YAML_V2,
        &[
            "--show",
            "demo",
            "add",
            "Nuevo sin número",
            "--slug",
            "nuevo-slug",
        ],
    );
    assert_case(
        "parity_write_oracle/oraculo_add_slug_explicito_y_derivado",
        1,
        MINI_YAML_V2,
        &["--show", "demo", "add", "Cómo montar un NAS"],
    );
}

#[test]
fn oraculo_add_duplicado() {
    assert_case(
        "parity_write_oracle/oraculo_add_duplicado",
        0,
        MINI_YAML,
        &["--show", "demo", "add", "101", "Repetido"],
    );
}

// ─── update ─────────────────────────────────────────────────────────

#[test]
fn oraculo_update_numero_y_slug() {
    assert_case(
        "parity_write_oracle/oraculo_update_numero_y_slug",
        0,
        MINI_YAML,
        &["--show", "demo", "update", "101", "--status", "recorded"],
    );
    assert_case(
        "parity_write_oracle/oraculo_update_numero_y_slug",
        1,
        MINI_YAML_V2,
        &[
            "--show",
            "demo",
            "update",
            "sin-numero",
            "--status",
            "recorded",
        ],
    );
    assert_case(
        "parity_write_oracle/oraculo_update_numero_y_slug",
        2,
        MINI_YAML_V2,
        &[
            "--show",
            "demo",
            "update",
            "no-existe",
            "--status",
            "recorded",
        ],
    );
}

// ─── publish ────────────────────────────────────────────────────────

#[test]
fn oraculo_publish() {
    assert_case(
        "parity_write_oracle/oraculo_publish",
        0,
        B4_YAML,
        &["--show", "demo", "publish", "101"],
    );
    assert_case(
        "parity_write_oracle/oraculo_publish",
        1,
        B4_YAML,
        &["--show", "demo", "publish", "provisional-uno"],
    );
    assert_case(
        "parity_write_oracle/oraculo_publish",
        2,
        B4_YAML,
        &["--show", "demo", "publish", "101", "--dry-run"],
    );
    assert_case(
        "parity_write_oracle/oraculo_publish",
        3,
        B4_YAML,
        &["--show", "demo", "publish", "100"],
    );
    assert_case(
        "parity_write_oracle/oraculo_publish",
        4,
        B4_YAML,
        &["--show", "demo", "publish", "999"],
    );
}

// ─── insert ─────────────────────────────────────────────────────────
//
// Nota (change `insert-derives-slug`): `insert` ahora deriva el `slug` del título
// cuando falta `--slug`, cosa que el oráculo (retirado) no hacía. Para seguir
// contrastando lo que el oráculo SÍ reproduce —desplazamiento de números, fechas
// y material—, estos casos fijan `--slug` explícito. La derivación se cubre con
// pruebas de fixture en `parity_write.rs`.

#[test]
fn oraculo_insert() {
    assert_case(
        "parity_write_oracle/oraculo_insert",
        0,
        MINI_YAML,
        &[
            "--show", "demo", "insert", "--before", "101", "--title", "Nuevo", "--slug", "nuevo",
        ],
    );
    assert_case(
        "parity_write_oracle/oraculo_insert",
        1,
        MINI_YAML,
        &[
            "--show",
            "demo",
            "insert",
            "--before",
            "101",
            "--title",
            "Nuevo",
            "--slug",
            "nuevo",
            "--dry-run",
        ],
    );
    assert_case(
        "parity_write_oracle/oraculo_insert",
        2,
        MINI_YAML,
        &[
            "--show", "demo", "insert", "--before", "999", "--title", "Nuevo", "--slug", "nuevo",
        ],
    );
    assert_case(
        "parity_write_oracle/oraculo_insert",
        3,
        MINI_YAML,
        &[
            "--show",
            "demo",
            "insert",
            "--before",
            "101",
            "--title",
            "Nuevo",
            "--slug",
            "nuevo",
            "--date",
            "2026-09-08",
        ],
    );
}

/// Contraste **congelado** del **orden de renombrado** con material anidado
/// (tokens en varios niveles): el oráculo ordena
/// `sorted(rglob, key=len(parts), reverse=True)` y el binario debe emitir
/// exactamente las mismas líneas en el mismo orden.
#[test]
fn oraculo_insert_material_anidado() {
    assert_case_with(
        "parity_write_oracle/oraculo_insert_material_anidado",
        0,
        MINI_YAML,
        &[
            "--show", "demo", "insert", "--before", "101", "--title", "Nuevo", "--slug", "nuevo",
        ],
        &|fx| {
            common::build_nested_material(&fx.root, 101);
            // Segundo episodio con material anidado, para ejercitar el plan de 102.
            common::build_nested_material(&fx.root, 102);
        },
    );
}

// ─── move ───────────────────────────────────────────────────────────

#[test]
fn oraculo_move() {
    assert_case(
        "parity_write_oracle/oraculo_move",
        0,
        MINI_YAML,
        &["--show", "demo", "move", "101", "--date", "2026-09-14"],
    );
    assert_case(
        "parity_write_oracle/oraculo_move",
        1,
        MINI_YAML,
        &[
            "--show",
            "demo",
            "move",
            "101",
            "--date",
            "2026-09-14",
            "--dry-run",
        ],
    );
    assert_case(
        "parity_write_oracle/oraculo_move",
        2,
        MINI_YAML,
        &["--show", "demo", "move", "101", "--date", "2026-09-10"],
    );
    assert_case(
        "parity_write_oracle/oraculo_move",
        3,
        MINI_YAML,
        &["--show", "demo", "move", "101", "--date", "2026-09-08"],
    );
    assert_case(
        "parity_write_oracle/oraculo_move",
        4,
        MINI_YAML,
        &["--show", "demo", "move", "999", "--date", "2026-09-14"],
    );
}
