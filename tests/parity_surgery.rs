//! `migration-surgery` — contrato de texto/exit/tracker/material de `renumber`,
//! `rename-material`, `backfill-slugs` y `migrate-material`, con **contraste en
//! vivo contra el oráculo** sobre el mismo fixture.

mod common;

use common::{Fixture, MINI_YAML, MINI_YAML_V2, build_material, snapshot};

/// Tracker B3 del oráculo (Fase B3): un publicado, un borrador con slug y otro sin.
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
  - number: 302
    season: 9
    date: 2026-09-10
    title: \"Sin slug\"
    status: scripted
";

/// Como `B3_YAML` pero con `302` con slug (para el escenario de colisión).
const B3_YAML_302_SLUG: &str = "\
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
  - number: 302
    slug: sin-slug
    season: 9
    date: 2026-09-10
    title: \"Sin slug\"
    status: scripted
";

/// Tracker con dos títulos repetidos (desambiguación `-2`).
const REPEATED_YAML: &str = "\
season: 9
season_start: 2026-09-01
episode_duration_minutes: 25
publish_days: [Monday, Thursday]

episodes:
  - number: 400
    season: 9
    date: 2026-09-03
    title: \"Repetido\"
    status: scripted
  - number: 401
    season: 9
    date: 2026-09-07
    title: \"Repetido\"
    status: scripted
";

/// Tracker con un título que no produce slug utilizable.
const NO_SLUG_TITLE_YAML: &str = "\
season: 9
season_start: 2026-09-01
episode_duration_minutes: 25
publish_days: [Monday, Thursday]

episodes:
  - number: 500
    season: 9
    date: 2026-09-03
    title: \"!!!\"
    status: scripted
";

fn numbers(fx: &Fixture) -> Vec<Option<u32>> {
    let tracker = podtrack::tracker::load_tracker(&fx.tracker).expect("load");
    let mut nums: Vec<Option<u32>> = tracker.episodes.iter().map(|e| e.number).collect();
    nums.sort_by_key(|n| (n.is_none(), n.unwrap_or(0)));
    nums
}

fn dates(fx: &Fixture) -> Vec<String> {
    podtrack::tracker::load_tracker(&fx.tracker)
        .expect("load")
        .episodes
        .iter()
        .map(|e| e.date.to_iso())
        .collect()
}

/// Contraste en vivo: mismo fixture, misma ruta de tracker, mismo argv; compara
/// stdout, stderr, exit code y **árbol completo** (tracker + material).
fn assert_parity(yaml: &str, args: &[&str], setup: &dyn Fn(&Fixture)) {
    common::assert_oracle_parity(yaml, args, setup);
}

fn no_setup(_: &Fixture) {}

fn material_setup(fx: &Fixture) {
    build_material(&fx.root);
}

// ─── renumber: contrato explícito ───────────────────────────────────

#[test]
fn renumber_desplaza_solo_numeros() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    common::run_ok(&fx, &["renumber", "--from", "101", "--delta", "5"]);
    assert_eq!(numbers(&fx), vec![Some(100), Some(106), Some(107)]);
    assert_eq!(dates(&fx), ["2026-09-03", "2026-09-07", "2026-09-10"]);
}

#[test]
fn renumber_delta_cero_es_error() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let (code, _o, err) = common::run_fail(&fx, &["renumber", "--from", "101", "--delta", "0"]);
    assert_eq!(code, 1);
    assert!(err.to_lowercase().contains("delta"));
    assert_eq!(common::read_tracker(&fx), MINI_YAML);
}

#[test]
fn renumber_umbral_sin_episodios_es_error() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let (code, _o, err) = common::run_fail(&fx, &["renumber", "--from", "999", "--delta", "1"]);
    assert_eq!(code, 1);
    assert!(err.contains("No hay episodios"));
    assert_eq!(common::read_tracker(&fx), MINI_YAML);
}

#[test]
fn renumber_colision_es_error() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let (code, _o, err) = common::run_fail(&fx, &["renumber", "--from", "101", "--delta", "-1"]);
    assert_eq!(code, 1);
    assert!(err.to_lowercase().contains("colisión"));
    assert_eq!(common::read_tracker(&fx), MINI_YAML);
}

#[test]
fn renumber_dry_run_no_escribe() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let out = common::run_ok(
        &fx,
        &["renumber", "--from", "101", "--delta", "5", "--dry-run"],
    );
    assert!(out.contains("DRY-RUN"));
    assert!(out.contains("#101 → #106"));
    assert_eq!(common::read_tracker(&fx), MINI_YAML);
}

#[test]
fn renumber_lista_en_orden_ascendente() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["renumber", "--from", "101", "--delta", "1"]);
    let i1 = out.find("#101 → #102").expect("101");
    let i2 = out.find("#102 → #103").expect("102");
    assert!(i1 < i2, "salida:\n{out}");
}

#[test]
fn renumber_ignora_numeros_nulos() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    common::run_ok(&fx, &["renumber", "--from", "202", "--delta", "5"]);
    assert_eq!(numbers(&fx), vec![Some(200), Some(207), None]);
}

#[test]
fn renumber_no_toca_material() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    build_material(&fx.root);
    let before = common::material_snapshot(&fx.root);
    common::run_ok(&fx, &["renumber", "--from", "101", "--delta", "1"]);
    assert_eq!(common::material_snapshot(&fx.root), before);
}

// ─── rename-material: contrato explícito ────────────────────────────

#[test]
fn rename_material_renombra_carpetas_ficheros_y_subcarpetas() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    build_material(&fx.root);
    let out = common::run_ok(&fx, &["rename-material", "--from", "101", "--delta", "1"]);

    assert!(!fx.root.join("0101").exists());
    assert!(fx.root.join("0102/guion-0102.md").exists());
    assert!(fx.root.join("0102/escaleta-0102.md").exists());
    assert!(fx.root.join("0102/presentacion-0102.typ").exists());
    assert!(fx.root.join("0102/ejemplos-0102").is_dir());
    assert!(fx.root.join("0102/working-e102.md").exists());
    assert!(fx.root.join("0103/guion-0103.md").exists());
    assert!(fx.root.join("0103/ejemplos-0103").is_dir());
    assert!(fx.root.join("0100/guion-0100.md").exists());
    assert!(out.contains("0101") && out.contains("0102"));
    assert!(out.contains("elemento(s) renombrado(s)"));
}

#[test]
fn rename_material_no_toca_el_tracker() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    build_material(&fx.root);
    let original = std::fs::read(&fx.tracker).expect("read");
    common::run_ok(&fx, &["rename-material", "--from", "101", "--delta", "1"]);
    assert_eq!(std::fs::read(&fx.tracker).expect("read"), original);
}

#[test]
fn rename_material_dry_run_no_toca_disco_ni_resume() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    build_material(&fx.root);
    let before = snapshot(&fx.root);
    let out = common::run_ok(
        &fx,
        &[
            "rename-material",
            "--from",
            "101",
            "--delta",
            "1",
            "--dry-run",
        ],
    );
    assert_eq!(snapshot(&fx.root), before);
    assert!(out.contains("0102"));
    assert!(!out.contains("elemento(s) renombrado(s)"));
}

#[test]
fn rename_material_exige_from_y_delta() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let (code, _o, err) = common::run_fail(&fx, &["rename-material"]);
    assert_eq!(code, 1);
    assert!(err.contains("requiere --from y --delta"));
}

#[test]
fn rename_material_delta_cero_es_error() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let (code, _o, err) =
        common::run_fail(&fx, &["rename-material", "--from", "101", "--delta", "0"]);
    assert_eq!(code, 1);
    assert!(err.to_lowercase().contains("delta"));
}

#[test]
fn rename_material_check_detecta_huerfanos() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    build_material(&fx.root);
    std::fs::create_dir_all(fx.root.join("0150")).expect("mkdir");
    std::fs::write(fx.root.join("0150/guion-0150.md"), "x\n").expect("write");
    let out = common::run_ok(&fx, &["rename-material", "--check"]);
    assert!(out.contains("0150"));
    assert!(out.to_lowercase().contains("desajuste"));
}

#[test]
fn rename_material_check_sin_huerfanos() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    build_material(&fx.root);
    let out = common::run_ok(&fx, &["rename-material", "--check"]);
    assert!(out.contains("Material alineado con el tracker."));
    assert!(!out.to_lowercase().contains("desajuste"));
}

// ─── backfill-slugs: contrato explícito ─────────────────────────────

fn slugs(fx: &Fixture) -> std::collections::BTreeMap<u32, String> {
    podtrack::tracker::load_tracker(&fx.tracker)
        .expect("load")
        .episodes
        .iter()
        .filter_map(|e| {
            e.number
                .zip(e.slug.as_ref().map(|s| s.as_str().to_string()))
        })
        .collect()
}

#[test]
fn backfill_asigna_a_los_sin_slug() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["backfill-slugs"]);
    assert!(
        out.contains("3 slugs asignados de 3 episodios"),
        "salida:\n{out}"
    );
    let expected: std::collections::BTreeMap<u32, String> = [
        (100, "primero".to_string()),
        (101, "segundo".to_string()),
        (102, "tercero".to_string()),
    ]
    .into_iter()
    .collect();
    assert_eq!(slugs(&fx), expected);
}

#[test]
fn backfill_respeta_los_existentes() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["backfill-slugs"]);
    assert!(out.contains("1 slugs asignados de 3 episodios"));
    let s = slugs(&fx);
    assert_eq!(s.get(&200).map(String::as_str), Some("primer-episodio"));
    assert_eq!(s.get(&202).map(String::as_str), Some("tercero"));
}

#[test]
fn backfill_desambigua_titulos_repetidos() {
    let fx = Fixture::new(REPEATED_YAML, "demo", "Show de prueba");
    common::run_ok(&fx, &["backfill-slugs"]);
    let s = slugs(&fx);
    assert_eq!(s.get(&400).map(String::as_str), Some("repetido"));
    assert_eq!(s.get(&401).map(String::as_str), Some("repetido-2"));
}

#[test]
fn backfill_titulo_sin_slug_deja_sin_slug() {
    let fx = Fixture::new(NO_SLUG_TITLE_YAML, "demo", "Show de prueba");
    let output = fx.cmd().arg("backfill-slugs").output().expect("run");
    assert!(output.status.success());
    let err = String::from_utf8(output.stderr).expect("utf8");
    assert!(
        err.contains("no produce un slug utilizable"),
        "stderr:\n{err}"
    );
    let s = slugs(&fx);
    assert!(s.is_empty(), "no debe asignarse slug: {s:?}");
}

#[test]
fn backfill_es_idempotente() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    common::run_ok(&fx, &["backfill-slugs"]);
    let after_first = std::fs::read(&fx.tracker).expect("read");
    let out = common::run_ok(&fx, &["backfill-slugs"]);
    assert!(out.contains("0 slugs asignados de 3 episodios"));
    assert_eq!(std::fs::read(&fx.tracker).expect("read"), after_first);
}

#[test]
fn backfill_dry_run_no_escribe() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["backfill-slugs", "--dry-run"]);
    assert!(out.contains("DRY-RUN"));
    assert!(out.contains("3 slugs asignados de 3 episodios"));
    assert_eq!(common::read_tracker(&fx), MINI_YAML);
}

// ─── migrate-material: contrato explícito ───────────────────────────

#[test]
fn migrate_renombra_carpeta_y_ficheros() {
    let fx = Fixture::new(B3_YAML, "demo", "Show de prueba");
    common::build_slug_material(&fx.root, 301);
    let out = common::run_ok(&fx, &["migrate-material"]);
    assert!(!fx.root.join("0301").exists());
    let dest = fx.root.join("borrador");
    assert!(dest.join("guion-borrador.md").exists());
    assert!(dest.join("investigacion-borrador.md").exists());
    assert!(dest.join("notas-borrador.md").exists());
    assert!(dest.join("episodio-borrador.md").exists());
    assert!(dest.join("ADR-episodio-borrador.md").exists());
    assert!(dest.join("ejemplos-borrador/demo.sh").exists());
    assert!(dest.join("template/base.typ").exists());
    assert!(dest.join("datos.db").exists());
    assert!(dest.join("presentacion.typ").exists());
    assert!(dest.join("cosas.bak").exists());
    assert_eq!(
        std::fs::read_to_string(dest.join("guion-borrador.md")).expect("read"),
        "x\n"
    );
    assert!(out.contains("0301") && out.contains("borrador"));
}

#[test]
fn migrate_omite_publicados_por_defecto() {
    let fx = Fixture::new(B3_YAML, "demo", "Show de prueba");
    common::build_slug_material(&fx.root, 300);
    common::build_slug_material(&fx.root, 301);
    common::run_ok(&fx, &["migrate-material"]);
    assert!(fx.root.join("0300/guion-0300.md").exists());
    assert!(!fx.root.join("publicado").exists());
    assert!(fx.root.join("borrador/guion-borrador.md").exists());
}

#[test]
fn migrate_include_published() {
    let fx = Fixture::new(B3_YAML, "demo", "Show de prueba");
    common::build_slug_material(&fx.root, 300);
    common::build_slug_material(&fx.root, 301);
    common::run_ok(&fx, &["migrate-material", "--include-published"]);
    assert!(!fx.root.join("0300").exists());
    assert!(fx.root.join("publicado/guion-publicado.md").exists());
    assert!(fx.root.join("borrador/guion-borrador.md").exists());
}

#[test]
fn migrate_detecta_colision_y_no_pisa() {
    let fx = Fixture::new(B3_YAML_302_SLUG, "demo", "Show de prueba");
    common::build_slug_material(&fx.root, 301);
    std::fs::write(fx.root.join("0301/investigacion-0301.md"), "a\n").expect("write");
    common::build_slug_material(&fx.root, 302);

    let (code, _out, err) = common::run_fail(&fx, &["migrate-material"]);
    assert_eq!(code, 1);
    assert!(err.to_lowercase().contains("colisión"));
    assert!(fx.root.join("0301/investigacion-0301.md").exists());
    assert!(fx.root.join("0301/investigacion-e301.md").exists());
    assert!(!fx.root.join("borrador").exists());
    // Las demás carpetas siguen.
    assert!(!fx.root.join("0302").exists());
    assert!(fx.root.join("sin-slug/guion-sin-slug.md").exists());
}

#[test]
fn migrate_destino_existente_colisiona() {
    let fx = Fixture::new(B3_YAML, "demo", "Show de prueba");
    common::build_slug_material(&fx.root, 301);
    std::fs::create_dir_all(fx.root.join("borrador")).expect("mkdir");
    let (code, _out, err) = common::run_fail(&fx, &["migrate-material"]);
    assert_eq!(code, 1);
    assert!(err.to_lowercase().contains("colisión"));
    assert!(fx.root.join("0301/guion-0301.md").exists());
}

#[test]
fn migrate_dry_run_no_toca_disco() {
    let fx = Fixture::new(B3_YAML, "demo", "Show de prueba");
    common::build_slug_material(&fx.root, 301);
    let before = snapshot(&fx.root);
    let output = fx
        .cmd()
        .args(["migrate-material", "--dry-run"])
        .output()
        .expect("run");
    assert!(output.status.success(), "code={:?}", output.status.code());
    let out = String::from_utf8(output.stdout).expect("utf8");
    assert!(out.contains("DRY-RUN"));
    assert!(out.contains("borrador"));
    assert_eq!(snapshot(&fx.root), before);
}

#[test]
fn migrate_episodio_sin_slug_se_salta() {
    let fx = Fixture::new(B3_YAML, "demo", "Show de prueba");
    common::build_slug_material(&fx.root, 302);
    let output = fx.cmd().arg("migrate-material").output().expect("run");
    assert!(output.status.success());
    let err = String::from_utf8(output.stderr).expect("utf8");
    assert!(err.to_lowercase().contains("no tiene slug"));
    assert!(fx.root.join("0302/guion-0302.md").exists());
}

#[test]
fn migrate_no_toca_el_tracker() {
    let fx = Fixture::new(B3_YAML, "demo", "Show de prueba");
    common::build_slug_material(&fx.root, 301);
    let original = std::fs::read(&fx.tracker).expect("read");
    common::run_ok(&fx, &["migrate-material"]);
    assert_eq!(std::fs::read(&fx.tracker).expect("read"), original);
}

// ─── contraste en vivo con el oráculo ───────────────────────────────

#[test]
fn oraculo_renumber() {
    assert_parity(
        MINI_YAML,
        &[
            "--show", "demo", "renumber", "--from", "101", "--delta", "5",
        ],
        &no_setup,
    );
    assert_parity(
        MINI_YAML,
        &[
            "--show", "demo", "renumber", "--from", "101", "--delta", "-1",
        ],
        &no_setup,
    );
    assert_parity(
        MINI_YAML,
        &[
            "--show", "demo", "renumber", "--from", "101", "--delta", "0",
        ],
        &no_setup,
    );
    assert_parity(
        MINI_YAML,
        &[
            "--show", "demo", "renumber", "--from", "999", "--delta", "1",
        ],
        &no_setup,
    );
    assert_parity(
        MINI_YAML,
        &[
            "--show",
            "demo",
            "renumber",
            "--from",
            "101",
            "--delta",
            "1",
            "--dry-run",
        ],
        &no_setup,
    );
    assert_parity(
        MINI_YAML_V2,
        &[
            "--show", "demo", "renumber", "--from", "202", "--delta", "5",
        ],
        &no_setup,
    );
}

#[test]
fn oraculo_rename_material() {
    assert_parity(
        MINI_YAML,
        &[
            "--show",
            "demo",
            "rename-material",
            "--from",
            "101",
            "--delta",
            "1",
        ],
        &material_setup,
    );
    assert_parity(
        MINI_YAML,
        &[
            "--show",
            "demo",
            "rename-material",
            "--from",
            "101",
            "--delta",
            "-1",
        ],
        &material_setup,
    );
    assert_parity(
        MINI_YAML,
        &[
            "--show",
            "demo",
            "rename-material",
            "--from",
            "101",
            "--delta",
            "1",
            "--dry-run",
        ],
        &material_setup,
    );
    assert_parity(
        MINI_YAML,
        &["--show", "demo", "rename-material"],
        &material_setup,
    );
    assert_parity(
        MINI_YAML,
        &[
            "--show",
            "demo",
            "rename-material",
            "--from",
            "101",
            "--delta",
            "0",
        ],
        &material_setup,
    );
    assert_parity(
        MINI_YAML,
        &["--show", "demo", "rename-material", "--check"],
        &material_setup,
    );
}

#[test]
fn oraculo_rename_material_check_huerfano() {
    assert_parity(
        MINI_YAML,
        &["--show", "demo", "rename-material", "--check"],
        &|fx| {
            build_material(&fx.root);
            std::fs::create_dir_all(fx.root.join("0150")).expect("mkdir");
            std::fs::write(fx.root.join("0150/guion-0150.md"), "x\n").expect("write");
        },
    );
}

#[test]
fn oraculo_backfill_slugs() {
    assert_parity(MINI_YAML, &["--show", "demo", "backfill-slugs"], &no_setup);
    assert_parity(
        MINI_YAML_V2,
        &["--show", "demo", "backfill-slugs"],
        &no_setup,
    );
    assert_parity(
        REPEATED_YAML,
        &["--show", "demo", "backfill-slugs"],
        &no_setup,
    );
    assert_parity(
        NO_SLUG_TITLE_YAML,
        &["--show", "demo", "backfill-slugs"],
        &no_setup,
    );
    assert_parity(
        MINI_YAML,
        &["--show", "demo", "backfill-slugs", "--dry-run"],
        &no_setup,
    );
}

#[test]
fn oraculo_backfill_idempotente() {
    // Segunda pasada sobre el tracker ya rellenado: el oráculo y el binario deben
    // emitir `0 slugs asignados…` y no tocar el fichero.
    let Some(module) = common::require_oracle() else {
        return;
    };
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let harness = fx.dir.path().join("oracle_main.py");
    std::fs::write(&harness, common::MAIN_HARNESS).expect("harness");
    let args = ["--show", "demo", "backfill-slugs"];

    common::reset_root(&fx, MINI_YAML, &no_setup);
    let _ = common::run_oracle_main(&module, &harness, &fx.shows, &args);
    let second_oracle = common::run_oracle_main(&module, &harness, &fx.shows, &args);
    let oracle_tree = snapshot(&fx.root);

    common::reset_root(&fx, MINI_YAML, &no_setup);
    let _ = common::run_podtrack(&fx, &args);
    let second_lib = common::run_podtrack(&fx, &args);
    let lib_tree = snapshot(&fx.root);

    assert_eq!(second_oracle.stdout, second_lib.stdout);
    assert_eq!(second_oracle.stderr, second_lib.stderr);
    assert_eq!(second_oracle.code, second_lib.code);
    assert_eq!(oracle_tree, lib_tree);
}

#[test]
fn oraculo_migrate_material() {
    let only_301 = |fx: &Fixture| common::build_slug_material(&fx.root, 301);
    let two = |fx: &Fixture| {
        common::build_slug_material(&fx.root, 300);
        common::build_slug_material(&fx.root, 301);
    };
    let no_slug = |fx: &Fixture| common::build_slug_material(&fx.root, 302);
    let convergence = |fx: &Fixture| {
        common::build_slug_material(&fx.root, 301);
        std::fs::write(fx.root.join("0301/investigacion-0301.md"), "a\n").expect("write");
        common::build_slug_material(&fx.root, 302);
    };
    let dest_exists = |fx: &Fixture| {
        common::build_slug_material(&fx.root, 301);
        std::fs::create_dir_all(fx.root.join("borrador")).expect("mkdir");
    };

    assert_parity(B3_YAML, &["--show", "demo", "migrate-material"], &only_301);
    assert_parity(B3_YAML, &["--show", "demo", "migrate-material"], &two);
    assert_parity(
        B3_YAML,
        &["--show", "demo", "migrate-material", "--include-published"],
        &two,
    );
    assert_parity(
        B3_YAML,
        &["--show", "demo", "migrate-material", "--dry-run"],
        &only_301,
    );
    assert_parity(B3_YAML, &["--show", "demo", "migrate-material"], &no_slug);
    assert_parity(
        B3_YAML_302_SLUG,
        &["--show", "demo", "migrate-material"],
        &convergence,
    );
    assert_parity(
        B3_YAML,
        &["--show", "demo", "migrate-material"],
        &dest_exists,
    );
}
