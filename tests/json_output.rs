//! `json-output` — la salida estructurada `--json` (opt-in, versionada, estable).

mod common;

use common::{B4_YAML, Fixture, MINI_YAML, MINI_YAML_V2};
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

#[test]
fn get_json_expone_el_material_del_episodio() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["get", "200", "--json"]);
    let doc: Value = serde_json::from_str(&out).expect("JSON válido");

    assert_eq!(doc["schema_version"], Value::from(1));
    assert_eq!(doc["episode"]["slug"], Value::from("primer-episodio"));
    let material = doc["episode"]["material"]
        .as_str()
        .expect("material string");
    assert!(
        material.ends_with("primer-episodio"),
        "material: {material}"
    );
}

#[test]
fn get_json_sin_slug_deja_material_nulo() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["get", "202", "--json"]);
    let doc: Value = serde_json::from_str(&out).expect("JSON válido");

    let episode = doc["episode"].as_object().expect("episode objeto");
    assert!(episode.contains_key("material"), "falta la clave material");
    assert!(episode["slug"].is_null(), "slug: {}", episode["slug"]);
    assert!(
        episode["material"].is_null(),
        "material: {}",
        episode["material"]
    );
}

#[test]
fn get_json_sin_numero_expone_material_por_slug() {
    // `sin-numero` tiene `number: null` pero SÍ tiene slug: su `material` no es `null`.
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["get", "sin-numero", "--json"]);
    let doc: Value = serde_json::from_str(&out).expect("JSON válido");

    assert!(
        doc["episode"]["number"].is_null(),
        "number: {}",
        doc["episode"]["number"]
    );
    assert_eq!(doc["episode"]["slug"], Value::from("sin-numero"));
    let material = doc["episode"]["material"]
        .as_str()
        .expect("material string");
    assert!(material.ends_with("sin-numero"), "material: {material}");
}

#[test]
fn el_material_es_exclusivo_de_get() {
    // Blindaje de D5: `material` NO debe arrastrarse al `episode` de los otros
    // comandos que comparten `episode_fields`.
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["list", "--json"]);
    let doc: Value = serde_json::from_str(&out).expect("JSON válido");
    let primero = doc["episodes"]
        .as_array()
        .expect("episodes array")
        .iter()
        .find(|e| e["slug"].as_str() == Some("primer-episodio"))
        .expect("episodio `primer-episodio`")
        .clone();
    let obj = primero.as_object().expect("episode objeto");
    assert!(
        !obj.contains_key("material"),
        "`list --json` no debe exponer `material`: {primero}"
    );

    // Y tampoco en un comando de escritura (`add --json`).
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["add", "Cuarto", "--json"]);
    let doc: Value = serde_json::from_str(&out).expect("JSON válido");
    let obj = doc["episode"].as_object().expect("episode objeto");
    assert!(
        !obj.contains_key("material"),
        "`add --json` no debe exponer `material`: {}",
        doc["episode"]
    );
}

/// Tracker de cinco episodios consecutivos (`676`–`680`), todos publicados.
const BARRIO_YAML: &str = "\
season: 8
season_start: 2026-09-01
episode_duration_minutes: 25
publish_days: [Monday, Thursday]

episodes:
  - number: 676
    season: 8
    date: 2026-09-03
    title: \"Episodio 676\"
    status: published
  - number: 677
    season: 8
    date: 2026-09-04
    title: \"Episodio 677\"
    status: published
  - number: 678
    season: 8
    date: 2026-09-05
    title: \"Episodio 678\"
    status: published
  - number: 679
    season: 8
    date: 2026-09-06
    title: \"Episodio 679\"
    status: published
  - number: 680
    season: 8
    date: 2026-09-07
    title: \"Episodio 680\"
    status: published
";

#[test]
fn neighborhood_json_parte_la_vecindad_en_anteriores_y_siguientes() {
    let fx = Fixture::new(BARRIO_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["neighborhood", "678", "--json"]);
    let doc: Value = serde_json::from_str(&out).expect("JSON válido");

    assert_eq!(doc["schema_version"], Value::from(1));
    assert_eq!(doc["show"], Value::from("demo"));
    assert_eq!(doc["episode"]["number"], Value::from(678));

    let numeros = |key: &str| -> Vec<u64> {
        doc[key]
            .as_array()
            .unwrap_or_else(|| panic!("{key} debe ser un array"))
            .iter()
            .map(|e| e["number"].as_u64().expect("number entero"))
            .collect()
    };
    assert_eq!(numeros("before"), vec![676, 677], "677 al final de before");
    assert_eq!(
        numeros("after"),
        vec![679, 680],
        "679 al principio de after"
    );

    let mut episodios = vec![&doc["episode"]];
    episodios.extend(doc["before"].as_array().expect("before array").iter());
    episodios.extend(doc["after"].as_array().expect("after array").iter());
    for ep in episodios {
        let obj = ep.as_object().expect("episodio objeto");
        assert!(
            !obj.contains_key("material"),
            "ningún episodio debe exponer `material`: {ep}"
        );
    }
}

#[test]
fn neighborhood_json_sin_vecinos_deja_listas_vacias() {
    let fx = Fixture::new(
        "season: 8\n\
         season_start: 2026-09-01\n\
         episode_duration_minutes: 25\n\
         publish_days: [Monday, Thursday]\n\
         \n\
         episodes:\n\
         \x20 - number: 678\n\
         \x20   season: 8\n\
         \x20   date: 2026-09-05\n\
         \x20   title: \"Solo\"\n\
         \x20   status: published\n",
        "demo",
        "Show de prueba",
    );
    let out = common::run_ok(&fx, &["neighborhood", "678", "--json"]);
    let doc: Value = serde_json::from_str(&out).expect("JSON válido");

    assert_eq!(doc["episode"]["number"], Value::from(678));
    assert!(doc.get("before").is_some(), "falta la clave before");
    assert!(doc.get("after").is_some(), "falta la clave after");
    assert_eq!(
        doc["before"].as_array().expect("before array").len(),
        0,
        "before debe estar vacío: {}",
        doc["before"]
    );
    assert_eq!(
        doc["after"].as_array().expect("after array").len(),
        0,
        "after debe estar vacío: {}",
        doc["after"]
    );
}
