//! Guardián de paridad en vivo de la **escritura**: compara, comando a comando,
//! la salida **y el tracker/material resultante** del binario con los del oráculo
//! Python sobre el **mismo fixture** (misma ruta de tracker, mismo árbol).
//!
//! Se salta si el oráculo no está disponible; con `PODTRACK_REQUIRE_ORACLE=1`
//! falla en vez de saltar (ver `common::require_oracle`).

mod common;

use std::path::Path;
use std::process::Command as ProcessCommand;

use common::{B4_YAML, Fixture, MINI_YAML, MINI_YAML_V2, build_material};

/// Harness que carga el módulo, apunta `SHOWS_DIR` al fixture y llama a `main()`
/// con los argumentos del oráculo (mismo camino que producción).
const HARNESS: &str = r#"
import importlib.util, sys
from pathlib import Path
spec = importlib.util.spec_from_file_location("pt", sys.argv[1])
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
m.SHOWS_DIR = Path(sys.argv[2])
sys.argv = ["podcast-tracker.py"] + sys.argv[3:]
m.main()
"#;

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

fn run_oracle(module: &Path, harness: &Path, shows: &Path, args: &[&str]) -> Outcome {
    let output = ProcessCommand::new("python3")
        .arg(harness)
        .arg(module)
        .arg(shows)
        .args(args)
        .output()
        .expect("python3");
    Outcome {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
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

fn assert_case_with(yaml: &str, args: &[&str], setup: &dyn Fn(&Fixture)) {
    let Some(module) = common::require_oracle() else {
        return;
    };
    let fx = Fixture::new(yaml, "demo", "Show de prueba");
    let harness = fx.dir.path().join("write_harness.py");
    std::fs::write(&harness, HARNESS).expect("harness");

    reset(&fx, yaml, setup);
    let oracle = run_oracle(&module, &harness, &fx.shows, args);
    let oracle_tracker = std::fs::read(&fx.tracker).expect("read tracker");
    let oracle_material = common::material_snapshot(&fx.root);

    reset(&fx, yaml, setup);
    let lib = run_podtrack(&fx, args);
    let lib_tracker = std::fs::read(&fx.tracker).expect("read tracker");
    let lib_material = common::material_snapshot(&fx.root);

    assert_eq!(oracle.stdout, lib.stdout, "stdout difiere para {args:?}");
    assert_eq!(oracle.stderr, lib.stderr, "stderr difiere para {args:?}");
    assert_eq!(oracle.code, lib.code, "exit code difiere para {args:?}");
    assert_eq!(
        oracle_tracker, lib_tracker,
        "tracker resultante difiere para {args:?}"
    );
    assert_eq!(
        oracle_material, lib_material,
        "material difiere para {args:?}"
    );
}

/// Como `assert_case_with`, con el material numerado estándar.
fn assert_case(yaml: &str, args: &[&str]) {
    assert_case_with(yaml, args, &|fx| build_material(&fx.root));
}

// ─── add ────────────────────────────────────────────────────────────

#[test]
fn oraculo_add_numero_explicito() {
    assert_case(MINI_YAML, &["--show", "demo", "add", "103", "Cuarto"]);
}

#[test]
fn oraculo_add_sin_numero() {
    assert_case(MINI_YAML, &["--show", "demo", "add", "Cuarto sin número"]);
}

#[test]
fn oraculo_add_slug_explicito_y_derivado() {
    assert_case(
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
        MINI_YAML_V2,
        &["--show", "demo", "add", "Cómo montar un NAS"],
    );
}

#[test]
fn oraculo_add_duplicado() {
    assert_case(MINI_YAML, &["--show", "demo", "add", "101", "Repetido"]);
}

// ─── update ─────────────────────────────────────────────────────────

#[test]
fn oraculo_update_numero_y_slug() {
    assert_case(
        MINI_YAML,
        &["--show", "demo", "update", "101", "--status", "recorded"],
    );
    assert_case(
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
    assert_case(B4_YAML, &["--show", "demo", "publish", "101"]);
    assert_case(B4_YAML, &["--show", "demo", "publish", "provisional-uno"]);
    assert_case(B4_YAML, &["--show", "demo", "publish", "101", "--dry-run"]);
    assert_case(B4_YAML, &["--show", "demo", "publish", "100"]);
    assert_case(B4_YAML, &["--show", "demo", "publish", "999"]);
}

// ─── insert ─────────────────────────────────────────────────────────

#[test]
fn oraculo_insert() {
    assert_case(
        MINI_YAML,
        &[
            "--show", "demo", "insert", "--before", "101", "--title", "Nuevo",
        ],
    );
    assert_case(
        MINI_YAML,
        &[
            "--show",
            "demo",
            "insert",
            "--before",
            "101",
            "--title",
            "Nuevo",
            "--dry-run",
        ],
    );
    assert_case(
        MINI_YAML,
        &[
            "--show", "demo", "insert", "--before", "999", "--title", "Nuevo",
        ],
    );
    assert_case(
        MINI_YAML,
        &[
            "--show",
            "demo",
            "insert",
            "--before",
            "101",
            "--title",
            "Nuevo",
            "--date",
            "2026-09-08",
        ],
    );
}

/// Contraste en vivo del **orden de renombrado** con material anidado (tokens en
/// varios niveles): el oráculo ordena `sorted(rglob, key=len(parts), reverse=True)`
/// y el binario debe emitir exactamente las mismas líneas en el mismo orden.
#[test]
fn oraculo_insert_material_anidado() {
    assert_case_with(
        MINI_YAML,
        &[
            "--show", "demo", "insert", "--before", "101", "--title", "Nuevo",
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
        MINI_YAML,
        &["--show", "demo", "move", "101", "--date", "2026-09-14"],
    );
    assert_case(
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
        MINI_YAML,
        &["--show", "demo", "move", "101", "--date", "2026-09-10"],
    );
    assert_case(
        MINI_YAML,
        &["--show", "demo", "move", "101", "--date", "2026-09-08"],
    );
    assert_case(
        MINI_YAML,
        &["--show", "demo", "move", "999", "--date", "2026-09-14"],
    );
}
