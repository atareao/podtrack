//! `tracker-read` — port 1:1 de la superficie de solo lectura del oráculo.

mod common;

use common::{B4_YAML, Fixture, MINI_YAML, MINI_YAML_V2, read_tracker, snapshot};

/// Tracker cuyo pendiente más antiguo **no** tiene slug (bloque sin ruta).
const NO_SLUG_YAML: &str = "\
season: 9
season_start: 2026-09-01
episode_duration_minutes: 25
publish_days: [Monday, Thursday]

episodes:
  - number: 100
    slug: publicado
    season: 9
    date: 2026-09-03
    title: \"Publicado\"
    status: published
  - number: 101
    season: 9
    date: 2026-09-07
    title: \"Provisional uno\"
    status: scripted
  - number: 102
    slug: provisional-dos
    season: 9
    date: 2026-09-10
    title: \"Provisional dos\"
    status: scripted
";

/// Tracker F1 con un episodio por slug (el `add` del oráculo es otra fase).
const F1_TRACKER: &str = "\
season: 9
season_start: 2026-09-01
episode_duration_minutes: 25
publish_days: [Monday, Thursday]

episodes:
  - number: null
    slug: cafe-con-nandu
    season: 9
    date: 2026-09-03
    title: \"Café con Ñandú\"
    status: scripted
";

fn rewrite(fx: &Fixture, from: &str, to: &str) {
    let text = read_tracker(fx);
    std::fs::write(&fx.tracker, text.replace(from, to)).expect("rewrite");
}

#[test]
fn cmd_status_smoke() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["status"]);
    assert!(out.contains("Show de prueba"));
    assert!(out.contains("Total epis:"));
    assert!(out.contains("Publicados:"));
}

#[test]
fn cmd_next_devuelve_max_mas_uno() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["next"]);
    assert!(out.contains("#103"), "salida: {out}");
}

#[test]
fn next_ignora_numeros_nulos() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["next"]);
    assert!(out.contains("Siguiente episodio: #203"), "salida: {out}");
}

#[test]
fn next_todos_nulos_empieza_en_uno() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    rewrite(&fx, "number: 200", "number: null");
    rewrite(&fx, "number: 202", "number: null");
    let out = common::run_ok(&fx, &["next"]);
    assert!(out.contains("Siguiente episodio: #1 "), "salida: {out}");
}

#[test]
fn list_formatea_numero_nulo_como_guion() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["list"]);
    assert!(out.contains("Sin número"));
    assert!(out.contains("—"));
}

#[test]
fn list_range_ignora_nulos() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["list-range", "200", "202"]);
    assert!(out.contains("Primero") && out.contains("Tercero"));
    assert!(!out.contains("Sin número"));
}

#[test]
fn get_numero_inexistente_error_claro() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let (code, _o, err) = common::run_fail(&fx, &["get", "999"]);
    assert_eq!(code, 1);
    assert!(err.to_lowercase().contains("no encontrado"));
}

#[test]
fn get_muestra_numero_normal() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["get", "202"]);
    assert!(out.contains("Tercero"));
}

#[test]
fn cmd_get_por_slug() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["get", "primer-episodio"]);
    assert!(out.contains("Primero"));
    assert!(out.contains("#200"));
}

#[test]
fn cmd_get_por_slug_sin_numero_muestra_guion() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["get", "sin-numero"]);
    assert!(out.contains("Sin número"));
    assert!(out.contains("#—"));
}

#[test]
fn cmd_get_slug_inexistente_error_claro() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let (code, _o, err) = common::run_fail(&fx, &["get", "no-existe"]);
    assert_eq!(code, 1);
    assert!(err.to_lowercase().contains("no encontrado"));
}

#[test]
fn get_cli_por_slug() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show Demo");
    let out = common::run_ok(&fx, &["get", "primer-episodio"]);
    assert!(out.contains("Primero"));
}

#[test]
fn where_imprime_ruta_y_existencia() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    std::fs::create_dir_all(fx.root.join("sin-numero")).expect("mkdir");

    let out = common::run_ok(&fx, &["where", "200"]);
    assert_eq!(
        out.trim(),
        format!("{} no existe", fx.root.join("primer-episodio").display())
    );

    let out = common::run_ok(&fx, &["where", "sin-numero"]);
    assert_eq!(
        out.trim(),
        format!("{} existe", fx.root.join("sin-numero").display())
    );
}

#[test]
fn where_episodio_sin_slug_sale_con_error() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let (code, _o, err) = common::run_fail(&fx, &["where", "202"]);
    assert_ne!(code, 0);
    assert!(err.to_lowercase().contains("slug"));
}

#[test]
fn where_inexistente_sale_con_error() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let (code, _o, err) = common::run_fail(&fx, &["where", "no-existe"]);
    assert_ne!(code, 0);
    assert!(err.to_lowercase().contains("no encontrado"));
}

#[test]
fn where_no_crea_nada() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let before = snapshot(&fx.root);
    let _ = common::run_ok(&fx, &["where", "200"]);
    assert_eq!(snapshot(&fx.root), before);
}

#[test]
fn where_cli_acepta_show() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show Demo");
    let out = common::run_ok(&fx, &["--show", "demo", "where", "sin-numero"]);
    assert!(out.contains(&fx.root.join("sin-numero").display().to_string()));
}

#[test]
fn f1_where_resuelve_scripts_root_como_padre_del_tracker() {
    // Adaptación del escenario F1: el `add` del oráculo pertenece a la fase de
    // escritura, así que aquí se fija la mitad que toca a esta fase — que `where`
    // resuelve `<raíz de guiones>/<slug>` (raíz = padre del tracker) y no crea nada.
    let fx = Fixture::new(F1_TRACKER, "demo", "Show Demo");
    let out = common::run_ok(&fx, &["where", "cafe-con-nandu"]);
    assert_eq!(
        out.trim(),
        format!("{} no existe", fx.root.join("cafe-con-nandu").display())
    );
    assert!(!fx.root.join("cafe-con-nandu").exists());
}

#[test]
fn next_muestra_hueco_y_proximo_a_publicar() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["next"]);
    assert!(out.contains("#103"));
    assert!(out.contains("Próximo a publicar: #101"));
}

#[test]
fn status_marca_provisionales_y_proximo_a_publicar() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["status"]);
    assert!(out.contains("101*"));
    assert!(out.contains("102*"));
    assert!(!out.contains("100*"));
    assert!(out.contains("* número provisional (se fija al publicar)"));
    assert!(out.contains("▶ Siguiente a publicar: #101 «Segundo»"));
}

#[test]
fn list_marca_no_publicados_con_asterisco_y_leyenda() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["list"]);
    assert!(out.contains("101*"));
    assert!(out.contains("102*"));
    assert!(!out.contains("100*"));
    assert!(out.contains("* número provisional (se fija al publicar)"));
}

#[test]
fn list_sin_no_publicados_no_lleva_asterisco_ni_leyenda() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["list", "--status", "published"]);
    assert!(out.contains("Primero"));
    assert!(!out.contains('*'));
}

#[test]
fn list_respeta_season_y_last_con_marcas() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["list", "--season", "8", "--last", "2"]);
    assert!(out.contains("101*") && out.contains("102*"));
    assert!(out.contains("* número provisional (se fija al publicar)"));
}

#[test]
fn list_range_marca_no_publicados_con_asterisco_y_leyenda() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["list-range", "100", "102"]);
    assert!(out.contains("101*"));
    assert!(out.contains("102*"));
    assert!(!out.contains("100*"));
    assert!(out.contains("* número provisional (se fija al publicar)"));
}

#[test]
fn list_range_solo_publicados_no_lleva_asterisco_ni_leyenda() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["list-range", "100", "100"]);
    assert!(out.contains("Primero"));
    assert!(!out.contains('*'));
}

#[test]
fn list_no_marca_episodios_sin_numero() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["list"]);
    assert!(out.contains("Sin número"));
    assert!(!out.contains("—*"));
    assert!(out.contains("202*"));
}

#[test]
fn status_muestra_siguiente_a_publicar_con_slug_y_ruta() {
    let fx = Fixture::new(B4_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["status"]);
    assert!(out.contains("▶ Siguiente a publicar: #101 «Provisional uno»"));
    assert!(out.contains("slug:  provisional-uno"));
    assert!(out.contains(&format!(
        "ruta:  {}",
        fx.root.join("provisional-uno").display()
    )));
    assert!(!out.contains("Próximo a publicar: #101"));
}

#[test]
fn status_siguiente_a_publicar_es_el_mas_antiguo_por_fecha() {
    let fx = Fixture::new(B4_YAML, "demo", "Show de prueba");
    rewrite(&fx, "date: 2026-09-07", "date: 2026-09-14");
    rewrite(&fx, "date: 2026-09-10", "date: 2026-09-07");
    let out = common::run_ok(&fx, &["status"]);
    assert!(out.contains("▶ Siguiente a publicar: #102 «Provisional dos»"));
    assert!(out.contains("slug:  provisional-dos"));
}

#[test]
fn status_sin_pendientes_omite_el_bloque() {
    let fx = Fixture::new(B4_YAML, "demo", "Show de prueba");
    rewrite(&fx, "status: scripted", "status: published");
    let out = common::run_ok(&fx, &["status"]);
    assert!(!out.contains("Siguiente a publicar"));
}

#[test]
fn status_siguiente_sin_slug_muestra_guion_sin_ruta() {
    let fx = Fixture::new(NO_SLUG_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["status"]);
    assert!(out.contains("▶ Siguiente a publicar: #101 «Provisional uno»"));
    assert!(out.contains("slug:  —"));
    assert!(!out.contains("ruta:"));
}

#[test]
fn status_no_modifica_el_tracker() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let before_content = read_tracker(&fx);
    let before_mtime = std::fs::metadata(&fx.tracker)
        .expect("meta")
        .modified()
        .expect("mtime");
    let before_tree = snapshot(&fx.root);

    let _ = common::run_ok(&fx, &["status"]);

    assert_eq!(read_tracker(&fx), before_content, "contenido inalterado");
    let after_mtime = std::fs::metadata(&fx.tracker)
        .expect("meta")
        .modified()
        .expect("mtime");
    assert_eq!(before_mtime, after_mtime, "mtime inalterado");
    assert_eq!(
        snapshot(&fx.root),
        before_tree,
        "sin .bak ni ficheros nuevos"
    );
    assert!(
        !fx.root.join("podcast-tracker.yaml.bak").exists(),
        "no debe crearse .bak"
    );
}
