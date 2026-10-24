//! `json-output` (escrituras) — documentos versionados de los comandos de
//! escritura y `--dry-run --json`.

mod common;

use common::{B4_YAML, Fixture, MINI_YAML};
use serde_json::Value;

fn parse(out: &str) -> Value {
    serde_json::from_str(out).unwrap_or_else(|e| panic!("JSON inválido: {e}\n{out}"))
}

#[test]
fn publish_json_expone_el_numero_estampado() {
    let fx = Fixture::new(B4_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["publish", "provisional-uno", "--json"]);
    let doc = parse(&out);
    assert_eq!(doc["schema_version"], Value::from(1));
    assert_eq!(doc["show"], Value::from("demo"));
    assert_eq!(doc["action"], Value::from("publish"));
    assert_eq!(doc["episode"]["number"], Value::from(101));
    assert_eq!(doc["episode"]["status"], Value::from("published"));
    assert_eq!(doc["dry_run"], Value::from(false));

    let reloaded = podtrack::tracker::load_tracker(&fx.tracker).expect("reload");
    assert_eq!(
        common::by_slug(&reloaded, "provisional-uno").number,
        Some(101)
    );
}

#[test]
fn dry_run_json_no_escribe_y_marca_previsualizacion() {
    let fx = Fixture::new(B4_YAML, "demo", "Show de prueba");
    let before = std::fs::read(&fx.tracker).expect("read");
    let out = common::run_ok(&fx, &["publish", "provisional-uno", "--json", "--dry-run"]);
    let doc = parse(&out);
    assert_eq!(doc["action"], Value::from("publish"));
    assert_eq!(doc["dry_run"], Value::from(true));
    assert_eq!(std::fs::read(&fx.tracker).expect("read"), before);
    assert!(!fx.root.join("podcast-tracker.yaml.bak").exists());
}

#[test]
fn add_json_emite_la_accion_y_el_episodio() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["add", "Cuarto", "--json"]);
    let doc = parse(&out);
    assert_eq!(doc["action"], Value::from("add"));
    assert_eq!(doc["episode"]["number"], Value::from(103));
    assert_eq!(doc["episode"]["title"], Value::from("Cuarto"));
    assert_eq!(doc["episode"]["status"], Value::from("scripted"));
    // Sin salida de texto (solo el documento JSON).
    assert!(out.trim_start().starts_with('{'));
    assert!(!out.contains("Tracker actualizado"));
}

#[test]
fn update_json_emite_el_cambio() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["update", "101", "--status", "recorded", "--json"]);
    let doc = parse(&out);
    assert_eq!(doc["action"], Value::from("update"));
    assert_eq!(doc["episode"]["status"], Value::from("recorded"));
    assert_eq!(doc["changes"][0]["from"], Value::from("scripted"));
    assert_eq!(doc["changes"][0]["to"], Value::from("recorded"));
}

#[test]
fn insert_json_expone_desplazamientos_y_material() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let out = common::run_ok(
        &fx,
        &["insert", "--before", "101", "--title", "X", "--json"],
    );
    let doc = parse(&out);
    assert_eq!(doc["action"], Value::from("insert"));
    assert_eq!(doc["episode"]["number"], Value::from(101));
    assert_eq!(doc["dry_run"], Value::from(false));
    assert!(doc["shifts"].is_array());
}

#[test]
fn move_json_emite_el_cambio_de_fecha() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["move", "101", "--date", "2026-09-14", "--json"]);
    let doc = parse(&out);
    assert_eq!(doc["action"], Value::from("move"));
    assert_eq!(doc["changes"][0]["from"], Value::from("2026-09-07"));
    assert_eq!(doc["changes"][0]["to"], Value::from("2026-09-14"));
}

#[test]
fn dry_run_json_de_insert_no_toca_el_disco() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let before = std::fs::read(&fx.tracker).expect("read");
    let out = common::run_ok(
        &fx,
        &[
            "insert",
            "--before",
            "101",
            "--title",
            "X",
            "--json",
            "--dry-run",
        ],
    );
    let doc = parse(&out);
    assert_eq!(doc["action"], Value::from("insert"));
    assert_eq!(doc["dry_run"], Value::from(true));
    assert_eq!(std::fs::read(&fx.tracker).expect("read"), before);
}
