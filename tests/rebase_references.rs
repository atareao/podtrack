//! `reference-rebase` — reescritura de hooks `Próximo episodio`, reporte de prosa
//! y de hooks sin título, con contraste en vivo contra el oráculo.

mod common;

use common::{Fixture, snapshot};
use podtrack::rebase::{RebaseKind, numeric_references, rebase_line};

/// Tracker del escenario de rebase (un publicado y dos borradores con slug).
const REBASE_YAML: &str = "\
season: 9
season_start: 2026-09-01
episode_duration_minutes: 25
publish_days: [Monday, Thursday]

episodes:
  - number: 500
    slug: publicado
    season: 9
    date: 2026-09-03
    title: \"Publicado\"
    status: published
  - number: 501
    slug: borrador-uno
    season: 9
    date: 2026-09-07
    title: \"Borrador uno\"
    status: scripted
  - number: 502
    slug: borrador-dos
    season: 9
    date: 2026-09-10
    title: \"Borrador dos\"
    status: scripted
";

const GUION_UNO: &str = "\
# Guion Borrador Uno

**Próximo episodio:** 502 — RAG II: Profundizando en la base de conocimiento

Algunas notas:
- **Próximo episodio:** 502 - RAG III: el regreso

Próximo episodio: 502 – Sin negritas ni nada

Proximo episodio: 502: con dos puntos y sin tilde
";

const ESCALETA_UNO: &str = "\
# Escaleta

En el episodio 501 hablamos de todo esto.
Nos escuchamos en el episodio 866.
El 502⭐ salió redondo.
";

const GUION_DOS: &str = "\
# Guion Borrador Dos

En el episodio 502 hablamos de más cosas.

Cierre.
";

const GUION_PUBLICADO: &str = "\
# Guion Publicado

**Próximo episodio:** 501 — algo del pasado
";

/// Material del escenario de rebase (`guiones_rebase` del oráculo).
fn setup_rebase(fx: &Fixture) {
    for (slug, content) in [
        ("publicado", GUION_PUBLICADO),
        ("borrador-uno", GUION_UNO),
        ("borrador-dos", GUION_DOS),
    ] {
        let folder = fx.root.join(slug);
        std::fs::create_dir_all(&folder).expect("mkdir");
        std::fs::write(folder.join(format!("guion-{slug}.md")), content).expect("write");
    }
    std::fs::write(
        fx.root.join("borrador-uno/escaleta-borrador-uno.md"),
        ESCALETA_UNO,
    )
    .expect("write");
}

fn rebase_file(fx: &Fixture, slug: &str) -> String {
    std::fs::read_to_string(fx.root.join(slug).join(format!("guion-{slug}.md"))).expect("read")
}

// ─── unitarios ──────────────────────────────────────────────────────

#[test]
fn rebase_line_clasifica_los_hooks() {
    let (line, kind) = rebase_line("## Próximo episodio: 844 — Encuadernar apuntes");
    assert_eq!(line, "## Próximo episodio: Encuadernar apuntes");
    assert_eq!(kind, Some(RebaseKind::Rewritten));

    let (line, kind) = rebase_line("Proximo episodio: 844 - Algo");
    assert_eq!(line, "Proximo episodio: Algo");
    assert_eq!(kind, Some(RebaseKind::Rewritten));

    let (line, kind) = rebase_line("## Próximo episodio: 844");
    assert_eq!(line, "## Próximo episodio: 844");
    assert_eq!(kind, Some(RebaseKind::NoTitle));

    let (line, kind) = rebase_line("En el episodio 501 hablamos de todo.");
    assert_eq!(line, "En el episodio 501 hablamos de todo.");
    assert_eq!(kind, None);
}

#[test]
fn numeric_references_encuentra_de_tres_a_cuatro_digitos() {
    assert_eq!(numeric_references("El 502 salió y 866 no"), vec![502, 866]);
    assert!(numeric_references("sin números").is_empty());
    assert!(numeric_references("el 12 y el 12345").is_empty());
}

// ─── contrato explícito (CLI) ───────────────────────────────────────

#[test]
fn reescribe_hooks_con_varias_formas() {
    let fx = Fixture::new(REBASE_YAML, "demo", "Show de prueba");
    setup_rebase(&fx);
    let out = common::run_ok(&fx, &["rebase-references"]);
    let text = rebase_file(&fx, "borrador-uno");

    assert!(
        text.contains("**Próximo episodio:** RAG II: Profundizando en la base de conocimiento")
    );
    assert!(text.contains("- **Próximo episodio:** RAG III: el regreso"));
    assert!(text.contains("Próximo episodio: Sin negritas ni nada"));
    assert!(text.contains("Proximo episodio: con dos puntos y sin tilde"));
    assert!(!text.contains("502 —"));
    assert!(!text.contains("502 -"));
    assert!(out.contains("Rebase de hooks estructurales"));
}

#[test]
fn no_toca_la_prosa_y_la_reporta() {
    let fx = Fixture::new(REBASE_YAML, "demo", "Show de prueba");
    setup_rebase(&fx);
    let out = common::run_ok(&fx, &["rebase-references"]);
    let text = std::fs::read_to_string(fx.root.join("borrador-uno/escaleta-borrador-uno.md"))
        .expect("read");
    assert_eq!(text, ESCALETA_UNO);
    assert!(out.contains("Referencias en prosa (no se tocan):"));
    assert!(out.contains("escaleta-borrador-uno.md"));
    assert!(out.contains("3 en 2 fichero"));
    assert!(out.contains("l.3"));
}

#[test]
fn hook_sin_titulo_reporta_y_devuelve_uno() {
    let fx = Fixture::new(REBASE_YAML, "demo", "Show de prueba");
    setup_rebase(&fx);
    let guion = fx.root.join("borrador-dos/guion-borrador-dos.md");
    std::fs::write(&guion, format!("{GUION_DOS}\n**Próximo episodio:** 999\n")).expect("write");

    let (code, out, _err) = common::run_fail(&fx, &["rebase-references"]);
    assert_eq!(code, 1);
    assert!(out.contains("Hooks sin título"));
    assert!(out.contains("999"));
    // El hook sin título NO se reescribe.
    let text = std::fs::read_to_string(&guion).expect("read");
    assert!(text.contains("**Próximo episodio:** 999"));
}

#[test]
fn no_toca_episodios_publicados() {
    let fx = Fixture::new(REBASE_YAML, "demo", "Show de prueba");
    setup_rebase(&fx);
    let before = rebase_file(&fx, "publicado");
    common::run_ok(&fx, &["rebase-references"]);
    assert_eq!(rebase_file(&fx, "publicado"), before);
}

#[test]
fn dry_run_no_escribe() {
    let fx = Fixture::new(REBASE_YAML, "demo", "Show de prueba");
    setup_rebase(&fx);
    let before = snapshot(&fx.root);
    let out = common::run_ok(&fx, &["rebase-references", "--dry-run"]);
    assert!(out.contains("DRY-RUN"));
    assert!(out.contains("ahora:"));
    assert_eq!(snapshot(&fx.root), before);
}

#[test]
fn es_idempotente() {
    let fx = Fixture::new(REBASE_YAML, "demo", "Show de prueba");
    setup_rebase(&fx);
    common::run_ok(&fx, &["rebase-references"]);
    let after_first = snapshot(&fx.root);
    let out = common::run_ok(&fx, &["rebase-references"]);
    assert_eq!(snapshot(&fx.root), after_first);
    assert!(out.contains("(sin hooks que reescribir)"));
}

#[test]
fn no_crea_ficheros_temporales() {
    let fx = Fixture::new(REBASE_YAML, "demo", "Show de prueba");
    setup_rebase(&fx);
    common::run_ok(&fx, &["rebase-references"]);
    let temps: Vec<String> = snapshot(&fx.root)
        .keys()
        .filter(|k| k.ends_with(".tmp"))
        .cloned()
        .collect();
    assert!(temps.is_empty(), "temporales: {temps:?}");
}

#[test]
fn sin_material_no_falla() {
    let fx = Fixture::new(REBASE_YAML, "demo", "Show de prueba");
    let out = common::run_ok(&fx, &["rebase-references"]);
    assert!(out.contains("(sin hooks que reescribir)"));
}

// ─── contraste en vivo ──────────────────────────────────────────────

#[test]
fn oraculo_rebase_references() {
    let no_title = |fx: &Fixture| {
        setup_rebase(fx);
        let guion = fx.root.join("borrador-dos/guion-borrador-dos.md");
        std::fs::write(&guion, format!("{GUION_DOS}\n**Próximo episodio:** 999\n")).expect("write");
    };
    let bare_hook = |fx: &Fixture| {
        setup_rebase(fx);
        let guion = fx.root.join("borrador-dos/guion-borrador-dos.md");
        std::fs::write(&guion, format!("{GUION_DOS}\nProximo episodio: 999\n")).expect("write");
    };

    common::assert_oracle_parity(
        REBASE_YAML,
        &["--show", "demo", "rebase-references"],
        &setup_rebase,
    );
    common::assert_oracle_parity(
        REBASE_YAML,
        &["--show", "demo", "rebase-references", "--dry-run"],
        &setup_rebase,
    );
    common::assert_oracle_parity(
        REBASE_YAML,
        &["--show", "demo", "rebase-references"],
        &no_title,
    );
    common::assert_oracle_parity(
        REBASE_YAML,
        &["--show", "demo", "rebase-references"],
        &bare_hook,
    );
    common::assert_oracle_parity(
        REBASE_YAML,
        &["--show", "demo", "rebase-references"],
        &|_fx| {},
    );
}
