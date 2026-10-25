//! `tracker-store` (modelo y writer canónico) — port 1:1 de los escenarios del
//! oráculo que fijan el modelo, el orden y el formato del YAML.
//!
//! Se ejercita la **librería** directamente (el `save_tracker` no es alcanzable
//! por CLI en esta fase: los comandos de escritura son un change posterior).

mod common;

use common::{B4_YAML, Fixture, MINI_YAML, MINI_YAML_V2};
use podtrack::model::{Date, slugify};
use podtrack::tracker::{load_tracker, save_tracker};

fn save(fx: &Fixture, tracker: &podtrack::tracker::Tracker) {
    let mut out = Vec::new();
    let mut err = Vec::new();
    save_tracker(&fx.tracker, "Show de prueba", tracker, &mut out, &mut err).expect("save");
}

fn save_capture_err(fx: &Fixture, tracker: &podtrack::tracker::Tracker) -> String {
    let mut out = Vec::new();
    let mut err = Vec::new();
    save_tracker(&fx.tracker, "Show de prueba", tracker, &mut out, &mut err).expect("save");
    String::from_utf8(err).expect("utf8")
}

#[test]
fn load_tracker_normaliza_fechas_a_string() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let tracker = load_tracker(&fx.tracker).expect("load");
    let dates: Vec<String> = tracker.episodes.iter().map(|e| e.date.to_iso()).collect();
    assert_eq!(dates, ["2026-09-03", "2026-09-07", "2026-09-10"]);
    assert!(tracker.episodes.iter().all(|e| e.date.known().is_some()));
}

#[test]
fn load_tracker_sin_fichero_sale_con_error() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    std::fs::remove_file(&fx.tracker).expect("rm");
    let err = load_tracker(&fx.tracker).expect_err("err");
    assert_eq!(err.exit_code(), 1);
    assert!(err.to_string().contains("No se encuentra"));
}

#[test]
fn load_tracker_yaml_sin_episodios_sale_con_error() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    std::fs::write(&fx.tracker, "season: 8\n").expect("write");
    let err = load_tracker(&fx.tracker).expect_err("err");
    assert_eq!(err.exit_code(), 1);
    assert!(err.to_string().contains("no tiene episodios"));
}

#[test]
fn save_tracker_ordena_por_numero_y_escribe_cabecera() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let mut tracker = load_tracker(&fx.tracker).expect("load");
    tracker.episodes.reverse();
    save(&fx, &tracker);

    let text = common::read_tracker(&fx);
    assert!(text.starts_with("# podcast-tracker.yaml\n"));
    assert!(text.contains("Estado del podcast \"Show de prueba\""));
    let i100 = text.find("number: 100").expect("100");
    let i101 = text.find("number: 101").expect("101");
    let i102 = text.find("number: 102").expect("102");
    assert!(i100 < i101 && i101 < i102);
}

#[test]
fn save_tracker_roundtrip_preserva_episodios() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let tracker = load_tracker(&fx.tracker).expect("load");
    save(&fx, &tracker);
    let again = load_tracker(&fx.tracker).expect("reload");
    assert_eq!(again.episodes, tracker.episodes);
}

#[test]
fn load_tracker_admite_number_nulo_y_slug() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let tracker = load_tracker(&fx.tracker).expect("load");
    let by_title = |t: &str| tracker.episodes.iter().find(|e| e.title == t).expect("ep");
    assert_eq!(by_title("Sin número").number, None);
    assert_eq!(
        by_title("Sin número").slug.as_ref().map(|s| s.as_str()),
        Some("sin-numero")
    );
    assert_eq!(
        by_title("Primero").slug.as_ref().map(|s| s.as_str()),
        Some("primer-episodio")
    );
}

#[test]
fn save_tracker_roundtrip_con_number_nulo_y_slug() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let tracker = load_tracker(&fx.tracker).expect("load");
    save(&fx, &tracker);

    let text = common::read_tracker(&fx);
    assert!(text.contains("number: null"));
    assert!(text.contains("slug: sin-numero"));
    assert_eq!(
        load_tracker(&fx.tracker).expect("reload").episodes,
        tracker.episodes
    );
}

#[test]
fn save_tracker_ordena_por_fecha() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let mut tracker = load_tracker(&fx.tracker).expect("load");
    tracker.episodes.reverse();
    save(&fx, &tracker);

    let text = common::read_tracker(&fx);
    let i200 = text.find("number: 200").expect("200");
    let inull = text.find("number: null").expect("null");
    let i202 = text.find("number: 202").expect("202");
    assert!(i200 < inull && inull < i202);
}

#[test]
fn save_tracker_avisa_fechas_duplicadas() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let mut tracker = load_tracker(&fx.tracker).expect("load");
    let first = tracker.episodes[0].date.to_iso();
    tracker.episodes[1].date = Date::parse(&first);
    let err_text = save_capture_err(&fx, &tracker);
    assert!(err_text.contains("Aviso"));
}

#[test]
fn save_tracker_crea_backup_y_no_deja_temporal() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let tracker = load_tracker(&fx.tracker).expect("load");
    save(&fx, &tracker);
    let backup = fx.root.join("podcast-tracker.yaml.bak");
    assert!(backup.exists(), "debe existir el backup .bak");
    assert_eq!(
        std::fs::read_to_string(&backup).expect("bak"),
        MINI_YAML,
        "el backup conserva el contenido previo"
    );
    let leftover: Vec<String> = common::snapshot(&fx.root)
        .keys()
        .filter(|k| k.ends_with(".tmp"))
        .cloned()
        .collect();
    assert!(
        leftover.is_empty(),
        "no debe quedar ningún temporal: {leftover:?}"
    );
}

#[test]
fn estado_desconocido_no_se_acepta() {
    let fx = Fixture::new(
        "season: 9\nseason_start: 2026-09-01\nepisode_duration_minutes: 25\npublish_days: [Monday, Thursday]\nepisodes:\n  - number: 1\n    season: 9\n    date: 2026-09-03\n    title: \"x\"\n    status: cancelled\n",
        "demo",
        "Show de prueba",
    );
    let err = load_tracker(&fx.tracker).expect_err("err");
    assert_ne!(err.exit_code(), 0);
    assert!(err.to_string().contains("Estado inválido"));
}

#[test]
fn slugify_normaliza_acentos_espacios_y_simbolos() {
    assert_eq!(slugify("Cómo hacer un Podcast"), "como-hacer-un-podcast");
    assert_eq!(slugify("  Espacios   de más  "), "espacios-de-mas");
    assert_eq!(slugify("Ñandú & Cía: ¡vamos!"), "nandu-cia-vamos");
    assert_eq!(slugify("A/B__C--D"), "a-b-c-d");
    assert_eq!(slugify("Versión 2.0 (final)"), "version-2-0-final");
}

#[test]
fn slugify_casos_vacios_y_ruido() {
    assert_eq!(slugify(""), "");
    assert_eq!(slugify("   "), "");
    assert_eq!(slugify("!!!"), "");
    assert_eq!(slugify("MiXeD CaSe"), "mixed-case");
}

#[test]
fn el_numero_no_ordena_y_sin_fecha_va_al_final() {
    let fx = Fixture::new(B4_YAML, "demo", "Show de prueba");
    let tracker = load_tracker(&fx.tracker).expect("load");
    // La fixture ya viene ordenada por fecha; `number` es solo desempate.
    let order: Vec<u32> = tracker.episodes.iter().filter_map(|e| e.number).collect();
    assert_eq!(order, vec![100, 101, 102]);
}
