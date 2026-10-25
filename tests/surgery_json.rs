//! `json-output` (cirugía) — documentos `--json` de los cinco comandos, `--dry-run`
//! y "sin `--json` no cambia el texto".

mod common;

use common::{Fixture, MINI_YAML};
use serde_json::Value;

/// Tracker B3 (para `migrate-material`).
const B3_YAML: &str = "\
season: 9
season_start: 2026-09-01
episode_duration_minutes: 25
publish_days: [Monday, Thursday]

episodes:
  - number: 300
    slug: publicado
    season: 9
    date: 2026-09-03
    title: \"Publicado\"
    status: published
  - number: 301
    slug: borrador
    season: 9
    date: 2026-09-07
    title: \"Borrador\"
    status: scripted
";

/// Tracker con un borrador con material para `rebase-references`.
const REBASE_YAML: &str = "\
season: 9
season_start: 2026-09-01
episode_duration_minutes: 25
publish_days: [Monday, Thursday]

episodes:
  - number: 501
    slug: borrador-uno
    season: 9
    date: 2026-09-07
    title: \"Borrador uno\"
    status: scripted
";

fn run_json(fx: &Fixture, args: &[&str]) -> (i32, Value) {
    let output = fx.cmd().args(args).output().expect("run");
    let code = output.status.code().expect("code");
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    let doc = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("JSON inválido ({args:?}): {e}\n{stdout}"));
    (code, doc)
}

#[test]
fn renumber_json_lista_los_cambios() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let (code, doc) = run_json(
        &fx,
        &["renumber", "--from", "101", "--delta", "1", "--json"],
    );
    assert_eq!(code, 0);
    assert_eq!(doc["schema_version"], Value::from(1));
    assert_eq!(doc["show"], Value::from("demo"));
    assert_eq!(doc["action"], Value::from("renumber"));
    assert_eq!(doc["dry_run"], Value::from(false));
    assert_eq!(doc["changes"][0]["from"], Value::from(101));
    assert_eq!(doc["changes"][0]["to"], Value::from(102));
}

#[test]
fn rename_material_json_y_check() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    common::build_material(&fx.root);
    let (_, doc) = run_json(
        &fx,
        &["rename-material", "--from", "101", "--delta", "1", "--json"],
    );
    assert_eq!(doc["action"], Value::from("rename-material"));
    assert_eq!(doc["dry_run"], Value::from(false));
    assert!(doc["changes"].is_array());

    let fx2 = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let (_, doc2) = run_json(&fx2, &["rename-material", "--check", "--json"]);
    assert_eq!(doc2["action"], Value::from("rename-material"));
    assert_eq!(doc2["check"], Value::from(true));
}

#[test]
fn backfill_json_lista_los_slugs() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let (_, doc) = run_json(&fx, &["backfill-slugs", "--json"]);
    assert_eq!(doc["action"], Value::from("backfill-slugs"));
    assert_eq!(doc["assigned"], Value::from(3));
    assert_eq!(doc["changes"][0]["slug"], Value::from("primero"));
}

#[test]
fn migrate_json_refleja_la_migracion() {
    let fx = Fixture::new(B3_YAML, "demo", "Show de prueba");
    common::build_slug_material(&fx.root, 301);
    let (code, doc) = run_json(&fx, &["migrate-material", "--json"]);
    assert_eq!(code, 0);
    assert_eq!(doc["action"], Value::from("migrate-material"));
    assert!(doc["migrations"].is_array());
}

#[test]
fn migrate_json_colision_mantiene_exit_uno() {
    let fx = Fixture::new(B3_YAML, "demo", "Show de prueba");
    common::build_slug_material(&fx.root, 301);
    std::fs::create_dir_all(fx.root.join("borrador")).expect("mkdir");
    let (code, doc) = run_json(&fx, &["migrate-material", "--json"]);
    assert_eq!(code, 1);
    assert_eq!(doc["action"], Value::from("migrate-material"));
    assert!(
        doc["collisions"].as_array().is_some_and(|c| !c.is_empty()),
        "doc: {doc}"
    );
}

#[test]
fn rebase_json_lista_los_hooks() {
    let fx = Fixture::new(REBASE_YAML, "demo", "Show de prueba");
    let folder = fx.root.join("borrador-uno");
    std::fs::create_dir_all(&folder).expect("mkdir");
    std::fs::write(
        folder.join("guion-borrador-uno.md"),
        "# Guion\n\n**Próximo episodio:** 502 — Algo\n",
    )
    .expect("write");

    let (code, doc) = run_json(&fx, &["rebase-references", "--json"]);
    assert_eq!(code, 0);
    assert_eq!(doc["action"], Value::from("rebase-references"));
    assert_eq!(doc["dry_run"], Value::from(false));
    assert!(doc["rewritten"].is_array());
}

#[test]
fn dry_run_json_marca_y_no_escribe() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let before = std::fs::read(&fx.tracker).expect("read");
    let (_, doc) = run_json(
        &fx,
        &[
            "renumber",
            "--from",
            "101",
            "--delta",
            "1",
            "--dry-run",
            "--json",
        ],
    );
    assert_eq!(doc["dry_run"], Value::from(true));
    assert_eq!(std::fs::read(&fx.tracker).expect("read"), before);
}

#[test]
fn sin_json_no_hay_json() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["renumber", "--from", "101", "--delta", "1"]);
    assert!(!out.trim_start().starts_with('{'));
    assert!(serde_json::from_str::<Value>(&out).is_err());
}
