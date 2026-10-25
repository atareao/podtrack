//! `material-layout-alignment` — la carpeta de material de un episodio se
//! resuelve con **una única regla** (`<raíz>/<episodes.scripts.dir renderizado>`)
//! en `where`, `get`, `status`, sus documentos `--json` y `migrate-material`
//! (change `material-layout-alignment`).
//!
//! Fixtures **temporales**: cada perfil y cada tracker viven en un `tempfile`.
//! **NUNCA** se toca `/data/notas`.
//!
//! Con un perfil cuyo `episodes.scripts.dir` es `"episodio-{n:02d}"` y un
//! episodio `#7` (`episodio-siete`), la carpeta de material debe ser
//! `<raíz>/episodio-07` —y **no** `<raíz>/episodio-siete`— en todas las
//! superficies.

use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

/// Slug del **show** (nombre del perfil), distinto del slug del episodio.
const SHOW: &str = "la-era-de-las-distros";
/// Nombre visible del show.
const NAME: &str = "La era de las distros";
/// Slug del episodio de fixture.
const EP_SLUG: &str = "episodio-siete";
/// Número del episodio de fixture.
const NUMBER: u32 = 7;

/// Carpeta de material que produce `dir: "episodio-{n:02d}"` para [`NUMBER`].
fn numeric_folder() -> String {
    format!("episodio-{NUMBER:02}")
}

/// Carpeta de material legacy (convención numérica `0NNN`) de [`NUMBER`].
fn legacy_folder() -> String {
    format!("{NUMBER:04}")
}

/// Tracker de fixture con un único episodio `#7` (`episodio-siete`), no publicado
/// (de modo que `status` lo señale como «Siguiente a publicar»).
const TRACKER_YAML: &str = "\
season: 1
season_start: 2026-01-01
episode_duration_minutes: 25
publish_days: [Monday]

episodes:
  - number: 7
    slug: episodio-siete
    season: 1
    date: 2026-01-05
    title: \"Episodio siete\"
    status: scripted
";

/// Tracker de fixture con un único episodio `#7` **sin slug**, no publicado.
///
/// Sirve para comprobar la cláusula «un episodio sin slug no tiene ruta de
/// material» con una plantilla que **no** necesita el slug (`episodio-{n:02d}`).
const TRACKER_YAML_NO_SLUG: &str = "\
season: 1
season_start: 2026-01-01
episode_duration_minutes: 25
publish_days: [Monday]

episodes:
  - number: 7
    season: 1
    date: 2026-01-05
    title: \"Episodio siete\"
    status: scripted
";

/// Número del episodio de fixture sin slug (mismo `#7`).
const NO_SLUG_NUMBER: u32 = 7;

/// Tracker de fixture con un único episodio **con slug y sin número**
/// (`number: null`), no publicado. Es un estado que el propio binario produce:
/// `backfill-slugs` asigna slug a episodios sin número.
const TRACKER_YAML_NO_NUMBER: &str = "\
season: 1
season_start: 2026-01-01
episode_duration_minutes: 25
publish_days: [Monday]

episodes:
  - number: null
    slug: episodio-sin-numero
    season: 1
    date: 2026-01-05
    title: \"Sin número\"
    status: scripted
";

/// Slug del episodio de fixture con slug pero sin número.
const NO_NUMBER_SLUG: &str = "episodio-sin-numero";

/// Resultado de una ejecución del binario.
struct Outcome {
    code: i32,
    stdout: String,
    stderr: String,
}

/// Fixture: `shows/<SHOW>.yaml` (con un `episodes.scripts.dir` dado) apuntando a
/// un `guiones/podcast-tracker.yaml`.
struct Harness {
    dir: TempDir,
    /// Raíz de material (directorio padre del tracker).
    root: PathBuf,
    shows: PathBuf,
    slug: String,
}

impl Harness {
    /// Fixture con el `episodes.scripts.dir` indicado.
    fn new(dir_template: &str) -> Self {
        Self::with_tracker(dir_template, TRACKER_YAML)
    }

    /// Fixture con el `episodes.scripts.dir` indicado y un tracker **sin slug**
    /// para el episodio `#7`.
    fn new_without_slug(dir_template: &str) -> Self {
        Self::with_tracker(dir_template, TRACKER_YAML_NO_SLUG)
    }

    /// Fixture con el `episodes.scripts.dir` y el tracker dados.
    fn with_tracker(dir_template: &str, tracker_yaml: &str) -> Self {
        let scripts = format!("  scripts:\n    dir: \"{dir_template}\"\n");
        Self::with_scripts(&scripts, tracker_yaml)
    }

    /// Fixture con un bloque `episodes.scripts` **completo** (ya indentado a dos
    /// espacios, p. ej. con `files.presentacion` para `open`) y el tracker dado.
    fn with_scripts(scripts_block: &str, tracker_yaml: &str) -> Self {
        let dir = TempDir::new().expect("tempdir");
        let root = dir.path().join("guiones");
        std::fs::create_dir_all(&root).expect("mkdir guiones");
        let tracker = root.join("podcast-tracker.yaml");
        std::fs::write(&tracker, tracker_yaml).expect("write tracker");

        let shows = dir.path().join("shows");
        std::fs::create_dir_all(&shows).expect("mkdir shows");
        let profile = format!(
            "slug: {SHOW}\nname: {NAME}\ntracker: {tracker}\nepisodes:\n{scripts_block}",
            tracker = tracker.display(),
        );
        std::fs::write(shows.join(format!("{SHOW}.yaml")), profile).expect("write profile");

        Self {
            dir,
            root,
            shows,
            slug: SHOW.to_string(),
        }
    }

    /// Carpeta de material esperada para un nombre de carpeta dado.
    fn material(&self, folder: &str) -> PathBuf {
        self.root.join(folder)
    }

    /// Crea `<raíz>/<folder>/` con un contenido sin token de número (para que el
    /// plan de `migrate-material` solo contenga el renombrado de la carpeta).
    fn write_material(&self, folder: &str) {
        let dir = self.material(folder);
        std::fs::create_dir_all(&dir).expect("mkdir material");
        std::fs::write(dir.join("notas.md"), "x\n").expect("write notas");
    }

    fn cmd(&self) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_podtrack"));
        cmd.env("PODTRACK_SHOWS_DIR", &self.shows);
        cmd.env("PODTRACK_SHOW", &self.slug);
        cmd.env("HOME", self.dir.path());
        cmd.env_remove("XDG_CONFIG_HOME");
        cmd
    }

    fn run(&self, args: &[&str]) -> Outcome {
        let output = self.cmd().args(args).output().expect("podtrack");
        Outcome {
            code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }
}

/// Línea de `stdout` que contiene `needle` (para mirar la línea `Ruta:`/`ruta:`).
fn line_with<'a>(stdout: &'a str, needle: &str) -> &'a str {
    stdout
        .lines()
        .find(|line| line.contains(needle))
        .unwrap_or_else(|| panic!("no hay línea con {needle:?} en:\n{stdout}"))
}

fn expected_display(path: &Path) -> String {
    path.display().to_string()
}

// ── 1.2 lectura: where / get / status ───────────────────────────────────────

#[test]
fn where_con_plantilla_numerica_imprime_la_carpeta_renderizada() {
    let h = Harness::new("episodio-{n:02d}");
    h.write_material(&numeric_folder());

    let out = h.run(&["where", EP_SLUG]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);

    let expected = expected_display(&h.material(&numeric_folder()));
    assert!(
        out.stdout.contains(&expected),
        "se esperaba {expected:?} en stdout:\n{}",
        out.stdout
    );
    let wrong = expected_display(&h.material(EP_SLUG));
    assert!(
        !out.stdout.contains(&wrong),
        "no se esperaba la carpeta por slug {wrong:?} en stdout:\n{}",
        out.stdout
    );
}

#[test]
fn get_con_plantilla_numerica_muestra_la_ruta_renderizada() {
    let h = Harness::new("episodio-{n:02d}");

    let out = h.run(&["get", EP_SLUG]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);

    let expected = expected_display(&h.material(&numeric_folder()));
    let ruta = line_with(&out.stdout, "Ruta:");
    assert!(
        ruta.contains(&expected),
        "línea Ruta: {ruta:?} no contiene {expected:?}"
    );
}

#[test]
fn status_con_plantilla_numerica_muestra_la_ruta_en_siguiente_a_publicar() {
    let h = Harness::new("episodio-{n:02d}");

    let out = h.run(&["status"]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    assert!(
        out.stdout.contains("Siguiente a publicar"),
        "stdout:\n{}",
        out.stdout
    );

    let expected = expected_display(&h.material(&numeric_folder()));
    let ruta = line_with(&out.stdout, "ruta:");
    assert!(
        ruta.contains(&expected),
        "línea ruta: {ruta:?} no contiene {expected:?}"
    );
}

// ── 1.3 lectura JSON: where --json / get --json ─────────────────────────────

#[test]
fn where_json_con_plantilla_numerica_emite_la_ruta_renderizada() {
    let h = Harness::new("episodio-{n:02d}");

    let out = h.run(&["where", EP_SLUG, "--json"]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);

    let expected = expected_display(&h.material(&numeric_folder()));
    let doc: serde_json::Value = serde_json::from_str(&out.stdout).expect("JSON de where");
    assert_eq!(
        doc["material"],
        serde_json::json!(expected),
        "material: {}",
        doc["material"]
    );
}

#[test]
fn get_json_con_plantilla_numerica_emite_la_ruta_renderizada() {
    let h = Harness::new("episodio-{n:02d}");

    let out = h.run(&["get", EP_SLUG, "--json"]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);

    let expected = expected_display(&h.material(&numeric_folder()));
    let doc: serde_json::Value = serde_json::from_str(&out.stdout).expect("JSON de get");
    assert_eq!(
        doc["episode"]["material"],
        serde_json::json!(expected),
        "material: {}",
        doc["episode"]["material"]
    );
}

// ── 1.4 cirugía: migrate-material planifica el destino renderizado ──────────

#[test]
fn migrate_material_planifica_el_destino_renderizado_no_el_slug() {
    let h = Harness::new("episodio-{n:02d}");
    // Material legacy en la convención numérica `0NNN`.
    h.write_material(&legacy_folder());

    let out = h.run(&["migrate-material", "--dry-run", "--include-published"]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    assert!(
        out.stdout
            .contains(&format!("{}  →  {}", legacy_folder(), numeric_folder())),
        "el destino debe ser {}; stdout:\n{}",
        numeric_folder(),
        out.stdout
    );
    assert!(
        !out.stdout.contains("→  episodio-siete"),
        "el destino NO debe ser la carpeta por slug; stdout:\n{}",
        out.stdout
    );
}

// ── 1.5 endurecimiento: un `dir` absoluto sale con 2 ────────────────────────

#[test]
fn dir_absoluto_hace_que_where_salga_con_dos_y_nombre_el_perfil() {
    let h = Harness::new("/etc");

    let out = h.run(&["where", EP_SLUG]);
    assert_eq!(out.code, 2, "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains(SHOW),
        "stderr debe nombrar el perfil {SHOW:?}: {}",
        out.stderr
    );
}

// ── Regresión: `dir: "{slug}"` sigue dando la carpeta del slug ──────────────
//
// Guardia de regresión: ya PASA hoy (la fórmula vieja colapsa a esta) y debe
// seguir pasando después del reapuntado. Si esto falla, se ha roto la paridad.

#[test]
fn where_con_dir_slug_sigue_imprimiendo_la_carpeta_del_slug() {
    let h = Harness::new("{slug}");

    let out = h.run(&["where", EP_SLUG]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);

    let expected = expected_display(&h.material(EP_SLUG));
    assert!(
        out.stdout.contains(&expected),
        "se esperaba {expected:?} en stdout:\n{}",
        out.stdout
    );
}

// ── 1.7 sin slug no hay ruta (regla uniforme en texto y JSON) ───────────────
//
// Un episodio sin `slug` no tiene ruta de material en **ninguna** superficie de
// lectura, sea cual sea la plantilla del perfil (`specs/json-output/spec.md`:
// «`material` … o `null` si el episodio no tiene `slug`»; `specs/tracker-read/
// spec.md`: «Si el episodio no tiene `slug`, SHALL mostrar `—` … NO SHALL
// inventarse ninguna ruta de material»). El texto y el JSON no pueden discrepar.

#[test]
fn get_sin_slug_con_plantilla_numerica_no_inventa_ruta() {
    let h = Harness::new_without_slug("episodio-{n:02d}");

    let out = h.run(&["get", &NO_SLUG_NUMBER.to_string()]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);

    let slug_line = line_with(&out.stdout, "Slug:");
    assert!(slug_line.contains('—'), "Slug: {slug_line:?}");
    let ruta_line = line_with(&out.stdout, "Ruta:");
    assert!(ruta_line.contains('—'), "Ruta: {ruta_line:?}");
    assert!(
        !out.stdout.contains(&numeric_folder()),
        "no se esperaba una ruta inventada en stdout:\n{}",
        out.stdout
    );
}

#[test]
fn get_json_sin_slug_con_plantilla_numerica_deja_material_nulo() {
    let h = Harness::new_without_slug("episodio-{n:02d}");

    let out = h.run(&["get", &NO_SLUG_NUMBER.to_string(), "--json"]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);

    let doc: serde_json::Value = serde_json::from_str(&out.stdout).expect("JSON de get");
    assert_eq!(doc["episode"]["number"], serde_json::json!(NO_SLUG_NUMBER));
    assert!(
        doc["episode"]["slug"].is_null(),
        "slug: {}",
        doc["episode"]["slug"]
    );
    assert!(
        doc["episode"]["material"].is_null(),
        "material: {}",
        doc["episode"]["material"]
    );
}

#[test]
fn status_sin_slug_con_plantilla_numerica_omite_la_ruta() {
    let h = Harness::new_without_slug("episodio-{n:02d}");

    let out = h.run(&["status"]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    assert!(
        out.stdout.contains("Siguiente a publicar"),
        "stdout:\n{}",
        out.stdout
    );
    assert!(
        !out.stdout.lines().any(|line| line.contains("ruta:")),
        "no debe haber línea `ruta:`:\n{}",
        out.stdout
    );
}

#[test]
fn status_json_sin_slug_con_plantilla_numerica_deja_material_nulo() {
    let h = Harness::new_without_slug("episodio-{n:02d}");

    let out = h.run(&["status", "--json"]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);

    let doc: serde_json::Value = serde_json::from_str(&out.stdout).expect("JSON de status");
    let ntp = &doc["next_to_publish"];
    assert_eq!(ntp["number"], serde_json::json!(NO_SLUG_NUMBER));
    assert!(ntp["slug"].is_null(), "slug: {}", ntp["slug"]);
    assert!(ntp["material"].is_null(), "material: {}", ntp["material"]);
}

#[test]
fn where_sin_slug_con_plantilla_numerica_es_caso_tratado() {
    let h = Harness::new_without_slug("episodio-{n:02d}");

    let out = h.run(&["where", &NO_SLUG_NUMBER.to_string()]);
    assert_ne!(out.code, 0, "stdout: {}", out.stdout);
    assert!(
        out.stderr.contains(&NO_SLUG_NUMBER.to_string()),
        "stderr debe mencionar el episodio #{}: {}",
        NO_SLUG_NUMBER,
        out.stderr
    );
    assert!(
        !out.stdout.contains(&numeric_folder()),
        "`where` no debe imprimir la ruta sin slug; stdout:\n{}",
        out.stdout
    );
}

// ── C1. plantilla numérica + episodio con slug pero SIN número ──────────────
//
// Estado que el propio binario produce (`backfill-slugs` asigna slug a episodios
// sin número): con `dir: "episodio-{n:02d}"` la ruta no es resoluble, así que la
// lectura sale con 1 y **nombra el episodio** (por su slug, no por `—`).

#[test]
fn get_con_plantilla_numerica_y_sin_numero_falla_nombrando_el_episodio() {
    let h = Harness::with_tracker("episodio-{n:02d}", TRACKER_YAML_NO_NUMBER);

    let out = h.run(&["get", NO_NUMBER_SLUG]);
    assert_eq!(out.code, 1, "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("no tiene número"),
        "stderr: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains(NO_NUMBER_SLUG),
        "el mensaje debe nombrar el episodio {NO_NUMBER_SLUG:?}: {}",
        out.stderr
    );
    assert!(
        out.stdout.trim().is_empty(),
        "no se inventa ninguna ruta; stdout:\n{}",
        out.stdout
    );
}

#[test]
fn status_con_plantilla_numerica_y_sin_numero_falla_nombrando_el_episodio() {
    let h = Harness::with_tracker("episodio-{n:02d}", TRACKER_YAML_NO_NUMBER);

    let out = h.run(&["status"]);
    assert_eq!(out.code, 1, "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("no tiene número"),
        "stderr: {}",
        out.stderr
    );
    assert!(
        out.stderr.contains(NO_NUMBER_SLUG),
        "el mensaje debe nombrar el episodio {NO_NUMBER_SLUG:?}: {}",
        out.stderr
    );
}

#[test]
fn get_json_con_plantilla_numerica_y_sin_numero_emite_documento_de_error() {
    let h = Harness::with_tracker("episodio-{n:02d}", TRACKER_YAML_NO_NUMBER);

    let out = h.run(&["get", NO_NUMBER_SLUG, "--json"]);
    assert_ne!(out.code, 0, "stdout: {}", out.stdout);
    assert!(
        out.stdout.trim().is_empty(),
        "sin texto en stdout: {}",
        out.stdout
    );
    let doc: serde_json::Value = serde_json::from_str(out.stderr.trim()).expect("stderr es JSON");
    let msg = doc["error"].as_str().expect("clave error");
    assert!(msg.contains("no tiene número"), "error: {msg}");
    assert!(
        msg.contains(NO_NUMBER_SLUG),
        "el error debe nombrar el episodio {NO_NUMBER_SLUG:?}: {msg}"
    );
}

// ── C2. rebase-references con plantilla numérica ────────────────────────────

#[test]
fn rebase_references_con_plantilla_numerica_y_sin_numero_no_aborta() {
    // I-1: un episodio con slug y sin número NO puede abortar la cirugía entera;
    // se salta, como los publicados/sin-slug/carpetas inexistentes.
    let h = Harness::with_tracker("episodio-{n:02d}", TRACKER_YAML_NO_NUMBER);

    let out = h.run(&["rebase-references"]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    assert!(
        out.stderr.trim().is_empty(),
        "sin errores en stderr: {}",
        out.stderr
    );
    assert!(
        out.stdout.contains("Rebase de hooks"),
        "stdout:\n{}",
        out.stdout
    );
}

#[test]
fn rebase_references_con_plantilla_numerica_visita_la_carpeta_renderizada() {
    let h = Harness::new("episodio-{n:02d}");
    // Material en la carpeta **renderizada** (`episodio-07/`), con un hook.
    let folder = h.material(&numeric_folder());
    std::fs::create_dir_all(&folder).expect("mkdir material");
    let hook = folder.join("guion.md");
    std::fs::write(&hook, "## Próximo episodio: 844 — Algo\n").expect("write hook");

    let out = h.run(&["rebase-references"]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);

    let content = std::fs::read_to_string(&hook).expect("read hook");
    assert!(
        content.contains("Próximo episodio: Algo"),
        "el hook debe quedar sin número (se visitó episodio-07/): {content}"
    );
    assert!(!content.contains("844"), "contenido: {content}");
}

// ── C3. `open` hereda el endurecimiento D5 ──────────────────────────────────

#[test]
fn open_con_dir_absoluto_sale_con_dos_y_no_lanza() {
    let scripts = "  scripts:\n    dir: \"/etc\"\n    files:\n      presentacion: \"presentacion-{slug}.pdf\"\n";
    let h = Harness::with_scripts(scripts, TRACKER_YAML);

    let out = h.run(&["open", EP_SLUG, "--dry-run"]);
    assert_eq!(out.code, 2, "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains(SHOW),
        "stderr debe nombrar el perfil {SHOW:?}: {}",
        out.stderr
    );
    assert!(
        out.stdout.trim().is_empty(),
        "no debe resolver ni imprimir ninguna ruta: {}",
        out.stdout
    );
}

// ── C4. migrate-material real: la cirugía escribe donde lee ─────────────────

#[test]
fn migrate_material_real_renombra_a_la_carpeta_renderizada() {
    let h = Harness::new("episodio-{n:02d}");
    h.write_material(&legacy_folder()); // `<raíz>/0007/notas.md`

    let out = h.run(&["migrate-material", "--include-published"]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);

    let dest = h.material(&numeric_folder());
    assert!(
        dest.is_dir(),
        "debe existir la carpeta renderizada {dest:?}"
    );
    assert!(
        dest.join("notas.md").is_file(),
        "el contenido viaja con la carpeta"
    );
    assert!(
        !h.material(&legacy_folder()).exists(),
        "la carpeta legacy 0007/ ya no debe existir"
    );
    assert!(
        !h.material(EP_SLUG).exists(),
        "NO debe crear la carpeta por slug"
    );
}

// ── C5. endurecimiento `..` a nivel CLI ─────────────────────────────────────

#[test]
fn dir_con_parentdir_hace_que_where_salga_con_dos() {
    let h = Harness::new("../../fuera");

    let out = h.run(&["where", EP_SLUG]);
    assert_eq!(out.code, 2, "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains(SHOW),
        "stderr debe nombrar el perfil {SHOW:?}: {}",
        out.stderr
    );
    assert!(
        out.stdout.trim().is_empty(),
        "no debe resolver ninguna ruta fuera de la raíz: {}",
        out.stdout
    );
}
