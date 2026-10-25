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
fn get_muestra_slug_y_ruta_del_material() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["get", "200"]);

    // `Slug:` y `Ruta:` arrancan en la misma columna que el valor de `Título:`
    // (etiqueta de 10 + un espacio, tras la sangría de 2).
    let esperado_slug = format!("  {:<10} {}", "Slug:", "primer-episodio");
    assert!(
        out.lines().any(|l| l == esperado_slug.as_str()),
        "falta `Slug:` alineado; salida:\n{out}"
    );

    let esperado_ruta = format!(
        "  {:<10} {}",
        "Ruta:",
        fx.root.join("primer-episodio").display()
    );
    assert!(
        out.lines().any(|l| l == esperado_ruta.as_str()),
        "falta `Ruta:` con la ruta absoluta del material; salida:\n{out}"
    );
}

#[test]
fn get_conserva_las_cinco_lineas_existentes() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["get", "200"]);

    assert!(out.contains("  Episodio #200"), "salida:\n{out}");
    assert!(out.contains("  Título:    Primero"), "salida:\n{out}");
    assert!(out.contains("  Estado:    ✅ published"), "salida:\n{out}");
    assert!(out.contains("  Fecha:     2025-09-04"), "salida:\n{out}");
    assert!(out.contains("  Temporada: 8"), "salida:\n{out}");
}

#[test]
fn get_sin_slug_muestra_guion_en_slug_y_ruta() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["get", "202"]);

    let esperado_slug = format!("  {:<10} {}", "Slug:", "—");
    assert!(
        out.lines().any(|l| l == esperado_slug.as_str()),
        "falta `Slug: —`; salida:\n{out}"
    );

    let esperado_ruta = format!("  {:<10} {}", "Ruta:", "—");
    assert!(
        out.lines().any(|l| l == esperado_ruta.as_str()),
        "falta `Ruta: —`; salida:\n{out}"
    );

    // No se inventa ninguna ruta de material para un episodio sin slug.
    assert!(
        !out.contains(&fx.root.display().to_string()),
        "no debe inventarse ruta; salida:\n{out}"
    );
}

#[test]
fn get_muestra_la_ruta_unque_el_material_no_exista() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    assert!(!fx.root.join("primer-episodio").exists());

    let out = common::run_ok(&fx, &["get", "200"]);
    assert!(
        out.contains(&fx.root.join("primer-episodio").display().to_string()),
        "salida:\n{out}"
    );
    // `get` describe; no crea el directorio (eso es `where`, que además informa de la existencia).
    assert!(!fx.root.join("primer-episodio").exists());
}

#[test]
fn get_por_slug_muestra_slug_y_ruta() {
    // El escenario «se resuelve por slug» de la spec se verifica resolviendo por
    // slug (no por número).
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["get", "primer-episodio"]);

    let esperado_slug = format!("  {:<10} {}", "Slug:", "primer-episodio");
    assert!(
        out.lines().any(|l| l == esperado_slug.as_str()),
        "falta `Slug:` alineado al resolver por slug; salida:\n{out}"
    );

    let esperado_ruta = format!(
        "  {:<10} {}",
        "Ruta:",
        fx.root.join("primer-episodio").display()
    );
    assert!(
        out.lines().any(|l| l == esperado_ruta.as_str()),
        "falta `Ruta:` alineada al resolver por slug; salida:\n{out}"
    );
}

#[test]
fn get_coloca_slug_y_ruta_entre_temporada_y_el_cierre() {
    // No basta con que las líneas existan: deben ir *después* de `Temporada:` y
    // *antes* del separador `═` de cierre.
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["get", "200"]);
    let lines: Vec<&str> = out.lines().collect();

    let idx = |etiqueta: &str| {
        lines
            .iter()
            .position(|l| l.trim_start().starts_with(etiqueta))
    };
    let temporada = idx("Temporada:").expect("línea `Temporada:`");
    let slug = idx("Slug:").expect("línea `Slug:`");
    let ruta = idx("Ruta:").expect("línea `Ruta:`");
    let cierre = lines
        .iter()
        .rposition(|l| l.contains('═'))
        .expect("separador de cierre");

    assert!(
        temporada < slug,
        "Slug: debe ir tras Temporada:; salida:\n{out}"
    );
    assert!(slug < ruta, "Ruta: debe ir tras Slug:; salida:\n{out}");
    assert!(
        ruta < cierre,
        "Ruta: debe ir antes del cierre; salida:\n{out}"
    );
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

// ── Ventana final de `status`: último publicado + 7 próximos ────────────────

/// Dos publicados y dos próximos, con fechas que fuerzan el orden.
const WINDOW_YAML: &str = "\
season: 9
season_start: 2026-09-01
episode_duration_minutes: 25
publish_days: [Monday, Thursday]

episodes:
  - number: 100
    slug: viejo
    season: 9
    date: 2026-09-01
    title: \"Antiguo\"
    status: published
  - number: 105
    slug: reciente
    season: 9
    date: 2026-09-15
    title: \"Reciente\"
    status: published
  - number: 106
    slug: proximo-uno
    season: 9
    date: 2026-09-18
    title: \"Proximo uno\"
    status: scripted
  - number: 107
    slug: proximo-dos
    season: 9
    date: 2026-09-22
    title: \"Proximo dos\"
    status: scripted
";

/// Sin publicados: la ventana solo puede mostrar próximos.
const NO_PUBLISHED_YAML: &str = "\
season: 9
season_start: 2026-09-01
episode_duration_minutes: 25
publish_days: [Monday, Thursday]

episodes:
  - number: 101
    slug: uno
    season: 9
    date: 2026-09-07
    title: \"Uno\"
    status: scripted
  - number: 102
    slug: dos
    season: 9
    date: 2026-09-10
    title: \"Dos\"
    status: recorded
";

/// Un publicado y diez próximos: la ventana debe cortar en 1 + 7.
const WINDOW_MANY_YAML: &str = "\
season: 9
season_start: 2026-09-01
episode_duration_minutes: 25
publish_days: [Monday, Thursday]

episodes:
  - number: 100
    slug: publicado
    season: 9
    date: 2026-09-01
    title: \"Publicado\"
    status: published
  - number: 101
    slug: p101
    season: 9
    date: 2026-09-03
    title: \"Proximo 101\"
    status: scripted
  - number: 102
    slug: p102
    season: 9
    date: 2026-09-07
    title: \"Proximo 102\"
    status: scripted
  - number: 103
    slug: p103
    season: 9
    date: 2026-09-10
    title: \"Proximo 103\"
    status: scripted
  - number: 104
    slug: p104
    season: 9
    date: 2026-09-14
    title: \"Proximo 104\"
    status: scripted
  - number: 105
    slug: p105
    season: 9
    date: 2026-09-17
    title: \"Proximo 105\"
    status: scripted
  - number: 106
    slug: p106
    season: 9
    date: 2026-09-21
    title: \"Proximo 106\"
    status: scripted
  - number: 107
    slug: p107
    season: 9
    date: 2026-09-24
    title: \"Proximo 107\"
    status: scripted
  - number: 108
    slug: p108
    season: 9
    date: 2026-09-28
    title: \"Proximo 108\"
    status: scripted
  - number: 109
    slug: p109
    season: 9
    date: 2026-10-01
    title: \"Proximo 109\"
    status: scripted
  - number: 110
    slug: p110
    season: 9
    date: 2026-10-05
    title: \"Proximo 110\"
    status: scripted
";

/// Tracker sin episodios.
const EMPTY_YAML: &str = "\
season: 9
season_start: 2026-09-01
episode_duration_minutes: 25
publish_days: [Monday, Thursday]

episodes: []
";

/// Cabecera del bloque de ventana.
const WINDOW_HEADER: &str = "Último publicado y próximos a publicar:";

/// Devuelve el texto de la ventana (todo lo que sigue a su cabecera).
fn window_of(out: &str) -> &str {
    out.split(WINDOW_HEADER)
        .nth(1)
        .unwrap_or_else(|| panic!("sin cabecera de ventana «{WINDOW_HEADER}»:\n{out}"))
}

#[test]
fn status_ventana_abre_con_el_ultimo_publicado_y_sigue_con_los_proximos() {
    let fx = Fixture::new(WINDOW_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["status"]);
    let w = window_of(&out);
    assert!(
        w.contains("105"),
        "la ventana debe abrir con el último publicado:\n{w}"
    );
    assert!(!w.contains("105*"), "el publicado NO lleva marca:\n{w}");
    assert!(
        !w.contains("100"),
        "el publicado antiguo no aparece en la ventana:\n{w}"
    );
    let i106 = w.find("106").expect("106 en la ventana");
    let i107 = w.find("107").expect("107 en la ventana");
    assert!(i106 < i107, "los próximos van en orden ascendente:\n{w}");
    assert!(
        w.contains("106*") && w.contains("107*"),
        "los próximos llevan *:\n{w}"
    );
}

#[test]
fn status_ventana_sin_publicados_muestra_solo_los_proximos() {
    let fx = Fixture::new(NO_PUBLISHED_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["status"]);
    let w = window_of(&out);
    assert!(
        w.contains("101*") && w.contains("102*"),
        "sin publicados, solo próximos:\n{w}"
    );
    assert!(
        !w.trim().is_empty(),
        "la ventana no debe quedar vacía:\n{w}"
    );
}

#[test]
fn status_ventana_sin_proximos_muestra_solo_el_publicado() {
    let fx = Fixture::new(B4_YAML, "demo", "Show de prueba");
    rewrite(&fx, "status: scripted", "status: published");
    let out = common::run_ok(&fx, &["status"]);
    let w = window_of(&out);
    assert!(
        w.contains("102"),
        "sin próximos, solo el último publicado:\n{w}"
    );
    assert!(!w.contains('*'), "sin próximos no hay marcas:\n{w}");
    assert!(!w.contains("101"), "el publicado antiguo no aparece:\n{w}");
}

#[test]
fn status_ventana_corta_en_1_publicado_y_7_proximos() {
    let fx = Fixture::new(WINDOW_MANY_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["status"]);
    let w = window_of(&out);
    for n in [101, 102, 103, 104, 105, 106, 107] {
        assert!(
            w.contains(&format!("{n}*")),
            "esperado {n}* en la ventana:\n{w}"
        );
    }
    assert!(!w.contains("108"), "la ventana corta en 7 próximos:\n{w}");
    assert!(
        !w.contains("109") && !w.contains("110"),
        "no debe pasar de 7:\n{w}"
    );
}

#[test]
fn status_sin_episodios_lo_dice() {
    let fx = Fixture::new(EMPTY_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["status"]);
    assert!(
        out.contains(WINDOW_HEADER),
        "siempre se imprime la cabecera:\n{out}"
    );
    assert!(
        out.contains("No hay episodios."),
        "sin episodios se dice:\n{out}"
    );
}
