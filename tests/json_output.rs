//! `json-output` — la salida estructurada `--json` (opt-in, versionada, estable).

mod common;

use common::{B4_YAML, Fixture, MINI_YAML};
use serde_json::Value;

#[test]
fn sin_json_la_salida_no_es_json() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["get", "100"]);
    assert!(!out.trim_start().starts_with('{'), "salida: {out}");
    assert!(serde_json::from_str::<Value>(&out).is_err());
}

#[test]
fn con_json_es_un_documento_con_schema_version_y_show() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["status", "--json"]);
    let doc: Value = serde_json::from_str(&out).expect("JSON válido");
    assert_eq!(doc["schema_version"], Value::from(1));
    assert_eq!(doc["show"], Value::from("demo"));
}

#[test]
fn status_json_expone_el_siguiente_a_publicar() {
    let fx = Fixture::new(B4_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["status", "--json"]);
    let doc: Value = serde_json::from_str(&out).expect("JSON válido");
    assert_eq!(doc["schema_version"], Value::from(1));
    assert_eq!(doc["show"], Value::from("demo"));

    let ntp = &doc["next_to_publish"];
    assert_eq!(ntp["number"], Value::from(101));
    assert_eq!(ntp["slug"], Value::from("provisional-uno"));
    assert_eq!(ntp["date"], Value::from("2026-09-07"));
    let material = ntp["material"].as_str().expect("material string");
    assert!(
        material.ends_with("provisional-uno"),
        "material: {material}"
    );
}

#[test]
fn los_comandos_de_lectura_emiten_json_valido() {
    let fx = Fixture::new(B4_YAML, "demo", "Show de prueba");
    for args in [
        vec!["next", "--json"],
        vec!["next-date", "--json"],
        vec!["list", "--json"],
        vec!["list-range", "100", "102", "--json"],
        vec!["get", "101", "--json"],
        vec!["where", "101", "--json"],
    ] {
        let out = common::run_ok(&fx, &args);
        let doc: Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{args:?}: {e}"));
        assert_eq!(doc["schema_version"], Value::from(1), "args: {args:?}");
        assert_eq!(doc["show"], Value::from("demo"), "args: {args:?}");
    }
}

#[test]
fn error_con_json_es_json_en_stderr() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let (code, out, err) = common::run_fail(&fx, &["status", "--json", "--show", "no-existe"]);
    assert_ne!(code, 0);
    assert!(
        out.trim().is_empty(),
        "sin salida de texto en stdout: {out}"
    );
    let doc: Value = serde_json::from_str(err.trim()).expect("stderr es JSON");
    assert!(doc["error"].as_str().is_some(), "clave error: {err}");
}
