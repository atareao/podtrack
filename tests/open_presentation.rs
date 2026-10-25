//! `open` — resolución de la **fuente** de la presentación Typst de un episodio y
//! su apertura desacoplada con `typshow` (change `open-presentation`).
//!
//! Todo se prueba con fixtures temporales: **NUNCA** se toca `/data/notas`. La
//! resolución se verifica con `--dry-run`; el lanzamiento, con un `typshow` falso
//! (shim) que deja un marcador.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use tempfile::TempDir;

const SLUG: &str = "la-memoria-de-valet";
const NUMBER: u32 = 843;
const NAME: &str = "atareao con Linux";

/// Bloque `episodes.scripts` de un perfil que **sí** declara la presentación.
const SCRIPTS_WITH_PRESENTATION: &str = r#"  scripts:
    dir: "{slug}"
    files:
      presentacion: "presentacion-{slug}.pdf"
"#;

/// Bloque `episodes.scripts` de un perfil que **no** declara la presentación.
const SCRIPTS_WITHOUT_PRESENTATION: &str = r#"  scripts:
    dir: "{slug}"
    files: {}
"#;

/// Bloque `episodes.scripts` **sin `dir`**: se usa el `{slug}` por defecto.
const SCRIPTS_NO_DIR: &str = r#"  scripts:
    files:
      presentacion: "presentacion-{slug}.pdf"
"#;

/// Bloque `episodes.scripts` con `dir` parametrizado por número (`{n:02d}`).
const SCRIPTS_DIR_NUMBER: &str = r#"  scripts:
    dir: "episodio-{n:02d}"
    files:
      presentacion: "presentacion-{slug}.typ"
"#;

/// Bloque `episodes.scripts` con tipos inesperados: `dir` es un *flow mapping*
/// (`{slug}` sin comillas) y `files.futuro` es una secuencia. La carga del perfil
/// debe ser **laxa** y no romperse.
const SCRIPTS_UNEXPECTED_TYPES: &str = r#"  scripts:
    dir: {slug}
    files:
      presentacion: "presentacion-{slug}.pdf"
      futuro: [a, b]
"#;

/// Tracker de fixture con un episodio `7` (`episodio-siete`).
const TRACKER_SEVEN: &str = "\
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

/// Tracker de fixture con un episodio **sin número** (`sin-numero`).
const TRACKER_NO_NUMBER: &str = "\
season: 1
season_start: 2026-01-01
episode_duration_minutes: 25
publish_days: [Monday]

episodes:
  - number: null
    slug: sin-numero
    season: 1
    date: null
    title: \"Sin número\"
    status: scripted
";

/// Tracker de fixture con un episodio `843` (`la-memoria-de-valet`).
const TRACKER_YAML: &str = "\
season: 9
season_start: 2026-09-01
episode_duration_minutes: 25
publish_days: [Monday, Thursday]

episodes:
  - number: 843
    slug: la-memoria-de-valet
    season: 9
    date: 2026-09-03
    title: \"La memoria de Valet\"
    status: scripted
";

/// Resultado de una ejecución del binario.
struct Outcome {
    code: i32,
    stdout: String,
    stderr: String,
}

/// Fixture de `open`: `shows/<slug>.yaml` + `guiones/` con tracker y material.
struct Harness {
    dir: TempDir,
    root: PathBuf,
    shows: PathBuf,
    slug: String,
}

impl Harness {
    /// Construye la fixture con el bloque `episodes.scripts` indicado.
    fn new(scripts_block: &str) -> Self {
        Self::with_tracker(scripts_block, TRACKER_YAML)
    }

    /// Construye la fixture con el bloque `episodes.scripts` y el tracker dados.
    fn with_tracker(scripts_block: &str, tracker_yaml: &str) -> Self {
        let dir = TempDir::new().expect("tempdir");
        let root = dir.path().join("guiones");
        std::fs::create_dir_all(&root).expect("mkdir guiones");
        let tracker = root.join("podcast-tracker.yaml");
        std::fs::write(&tracker, tracker_yaml).expect("write tracker");

        let shows = dir.path().join("shows");
        std::fs::create_dir_all(&shows).expect("mkdir shows");
        let profile = format!(
            "slug: {SLUG}\nname: {NAME}\ntracker: {tracker}\nepisodes:\n{scripts_block}",
            tracker = tracker.display(),
        );
        std::fs::write(shows.join(format!("{SLUG}.yaml")), profile).expect("write profile");

        Self {
            dir,
            root,
            shows,
            slug: SLUG.to_string(),
        }
    }

    /// Ruta esperada de la fuente `.typ` (sin crearla).
    fn expected_typ(&self) -> PathBuf {
        self.root
            .join(&self.slug)
            .join(format!("presentacion-{}.typ", self.slug))
    }

    /// Crea el `.typ` de la presentación y devuelve su ruta.
    fn write_typ(&self) -> PathBuf {
        let path = self.expected_typ();
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir material");
        std::fs::write(&path, "- hola\n").expect("write typ");
        path
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

    /// Ejecuta con un `typshow` falso en el `PATH`: escribe su primer argumento en
    /// `marker` (`$PODTRACK_TEST_MARKER`).
    fn run_with_shim(&self, args: &[&str], marker: &Path) -> Outcome {
        let bin = self.dir.path().join("bin");
        std::fs::create_dir_all(&bin).expect("mkdir bin");
        let shim = bin.join("typshow");
        std::fs::write(
            &shim,
            "#!/bin/sh\nprintf '%s' \"$1\" > \"$PODTRACK_TEST_MARKER\"\n",
        )
        .expect("write shim");
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&shim).expect("meta").permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&shim, perms).expect("chmod");
        }

        let path = std::env::var("PATH").unwrap_or_default();
        let output = self
            .cmd()
            .args(args)
            .env("PATH", format!("{}:{path}", bin.display()))
            .env("PODTRACK_TEST_MARKER", marker)
            .output()
            .expect("podtrack");
        Outcome {
            code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }

    /// Ejecuta con un `PATH` **exacto** (sin heredar el del proceso): permite
    /// simular la ausencia del lanzador apuntando a un directorio vacío.
    fn run_with_path(&self, args: &[&str], path: &Path) -> Outcome {
        let output = self
            .cmd()
            .args(args)
            .env("PATH", path)
            .output()
            .expect("podtrack");
        Outcome {
            code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }
}

/// Sondea que aparezca `marker` hasta 3 s (el visor se lanza en segundo plano).
fn poll_marker(marker: &Path) -> Option<String> {
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if let Ok(content) = std::fs::read_to_string(marker) {
            return Some(content);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    std::fs::read_to_string(marker).ok()
}

// ── 1.1 resolución (por slug y por número) ───────────────────────────────────

#[test]
fn open_por_slug_dry_run_json_resuelve_y_no_lanza() {
    let h = Harness::new(SCRIPTS_WITH_PRESENTATION);
    let typ = h.write_typ();

    let out = h.run(&["open", SLUG, "--dry-run", "--json"]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);

    let doc: serde_json::Value = serde_json::from_str(&out.stdout).expect("JSON de open");
    let presentation = doc["presentation"].as_str().expect("clave presentation");
    assert!(
        presentation.ends_with(&format!("/{SLUG}/presentacion-{SLUG}.typ")),
        "presentation: {presentation}"
    );
    assert_eq!(presentation, typ.display().to_string());
    assert_eq!(doc["launched"], serde_json::json!(false));
}

#[test]
fn open_por_numero_da_la_misma_ruta_en_texto() {
    let h = Harness::new(SCRIPTS_WITH_PRESENTATION);
    let typ = h.write_typ();

    let target = NUMBER.to_string();
    let out = h.run(&["open", target.as_str(), "--dry-run"]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    assert!(
        out.stdout.contains(&typ.display().to_string()),
        "stdout: {}",
        out.stdout
    );
    assert!(out.stdout.contains("DRY-RUN"), "stdout: {}", out.stdout);
}

// ── 1.2 sin presentación declarada ───────────────────────────────────────────

#[test]
fn sin_presentacion_declarada_sale_con_dos() {
    let h = Harness::new(SCRIPTS_WITHOUT_PRESENTATION);
    h.write_typ(); // el material existe; el fallo es del perfil, no del disco

    let out = h.run(&["open", SLUG, "--dry-run"]);
    assert_eq!(out.code, 2, "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("no declara una presentación"),
        "stderr: {}",
        out.stderr
    );
}

// ── 1.3 `.typ` ausente ───────────────────────────────────────────────────────

#[test]
fn typ_ausente_sale_con_uno_y_nombra_episodio_y_ruta() {
    let h = Harness::new(SCRIPTS_WITH_PRESENTATION);
    // No se escribe el `.typ`.
    let expected = h.expected_typ();

    let out = h.run(&["open", SLUG]);
    assert_eq!(out.code, 1, "stderr: {}", out.stderr);
    assert!(out.stderr.contains(SLUG), "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains(&expected.display().to_string()),
        "stderr: {}",
        out.stderr
    );
}

// ── 1.3bis no lanza si falla; SÍ lanza si va bien (shim) ─────────────────────

#[test]
fn no_lanza_si_falla_y_si_lanza_si_va_bien() {
    // Caso fallo: `.typ` ausente → typshow NO debe ejecutarse.
    let fail = Harness::new(SCRIPTS_WITH_PRESENTATION);
    let marker_fail = fail.dir.path().join("marker-fail");
    let out = fail.run_with_shim(&["open", SLUG], &marker_fail);
    assert_eq!(out.code, 1, "stderr: {}", out.stderr);
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        !marker_fail.exists(),
        "typshow NO debe lanzarse cuando la resolución falla"
    );

    // Caso éxito: `.typ` presente → typshow se lanza con la ruta y deja el marcador.
    let ok = Harness::new(SCRIPTS_WITH_PRESENTATION);
    let typ = ok.write_typ();
    let marker_ok = ok.dir.path().join("marker-ok");
    let out = ok.run_with_shim(&["open", SLUG], &marker_ok);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);

    let content = poll_marker(&marker_ok)
        .expect("typshow debía lanzarse y escribir el marcador (sondeo 3 s)");
    assert!(
        content.contains(&typ.display().to_string()),
        "marcador: {content}"
    );
}

// ── B1. sin lanzador en el PATH ──────────────────────────────────────────────

#[test]
fn sin_typshow_en_el_path_sale_con_launcher_not_found() {
    let h = Harness::new(SCRIPTS_WITH_PRESENTATION);
    h.write_typ();

    let empty = h.dir.path().join("path-vacio");
    std::fs::create_dir_all(&empty).expect("mkdir path-vacio");
    let out = h.run_with_path(&["open", SLUG], &empty);

    assert_eq!(out.code, 1, "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("No se encuentra el lanzador"),
        "stderr: {}",
        out.stderr
    );
    assert!(out.stderr.contains("typshow"), "stderr: {}", out.stderr);
}

// ── B2. sin `dir` declarado: `{slug}` por defecto ────────────────────────────

#[test]
fn sin_dir_declarado_usa_el_slug_por_defecto() {
    let h = Harness::new(SCRIPTS_NO_DIR);
    let typ = h.write_typ();

    let out = h.run(&["open", "843", "--dry-run"]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    assert_eq!(typ, h.expected_typ(), "ruta esperada inesperada");
    assert!(
        out.stdout.contains(&typ.display().to_string()),
        "stdout: {}",
        out.stdout
    );
}

// ── B3. JSON completo de `open` ───────────────────────────────────────────────

#[test]
fn json_completo_de_open() {
    let h = Harness::new(SCRIPTS_WITH_PRESENTATION);
    let typ = h.write_typ();

    let out = h.run(&["open", SLUG, "--dry-run", "--json"]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);

    let doc: serde_json::Value = serde_json::from_str(&out.stdout).expect("JSON de open");
    assert_eq!(doc["schema_version"], serde_json::json!(1));
    assert_eq!(doc["show"], serde_json::json!(SLUG));
    assert_eq!(doc["episode"]["number"], serde_json::json!(NUMBER));
    assert_eq!(doc["episode"]["slug"], serde_json::json!(SLUG));
    assert_eq!(doc["launcher"], serde_json::json!("typshow"));
    assert_eq!(
        doc["presentation"],
        serde_json::json!(typ.display().to_string())
    );
    assert_eq!(doc["launched"], serde_json::json!(false));
}

// ── B4. episodio inexistente ─────────────────────────────────────────────────

#[test]
fn open_de_episodio_inexistente_sale_con_uno() {
    let h = Harness::new(SCRIPTS_WITH_PRESENTATION);
    h.write_typ();

    let out = h.run(&["open", "999", "--dry-run"]);
    assert_eq!(out.code, 1, "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("no encontrado"),
        "stderr: {}",
        out.stderr
    );
}

// ── B5. `dir` parametrizado por número ───────────────────────────────────────

#[test]
fn dir_parametrizado_por_numero_resuelve_la_carpeta() {
    let h = Harness::with_tracker(SCRIPTS_DIR_NUMBER, TRACKER_SEVEN);
    let target = h
        .root
        .join("episodio-07")
        .join("presentacion-episodio-siete.typ");
    std::fs::create_dir_all(target.parent().expect("parent")).expect("mkdir material");
    std::fs::write(&target, "- hola\n").expect("write typ");

    let out = h.run(&["open", "7", "--dry-run"]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    assert!(
        out.stdout.contains(&target.display().to_string()),
        "stdout: {}",
        out.stdout
    );
}

// ── B6. `dir` por número con episodio sin número ─────────────────────────────

#[test]
fn episodio_sin_numero_con_dir_por_numero_sale_con_uno() {
    let h = Harness::with_tracker(SCRIPTS_DIR_NUMBER, TRACKER_NO_NUMBER);

    let out = h.run(&["open", "sin-numero", "--dry-run"]);
    assert_eq!(out.code, 1, "stderr: {}", out.stderr);
    assert!(
        out.stderr.contains("no tiene número"),
        "stderr: {}",
        out.stderr
    );
}

// ── B7. tipos inesperados en el perfil no rompen la carga ────────────────────

#[test]
fn claves_extra_y_tipos_inesperados_no_rompen_la_carga() {
    let h = Harness::new(SCRIPTS_UNEXPECTED_TYPES);
    let typ = h.write_typ();

    // (a) Un comando ajeno a `open` carga el perfil sin romperse.
    let status = h.run(&["status"]);
    assert_eq!(status.code, 0, "stderr: {}", status.stderr);

    // (b) `open` usa el `{slug}` por defecto (el `dir` no escalar se ignora).
    let out = h.run(&["open", "843", "--dry-run"]);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    assert!(
        out.stdout.contains(&typ.display().to_string()),
        "stdout: {}",
        out.stdout
    );
}

// ── B8. `--dry-run` no lanza aunque el lanzador exista ───────────────────────

#[test]
fn dry_run_no_lanza_aunque_el_lanzador_exista() {
    let h = Harness::new(SCRIPTS_WITH_PRESENTATION);
    h.write_typ();
    let marker = h.dir.path().join("marker-dry-run");

    let out = h.run_with_shim(&["open", SLUG, "--dry-run"], &marker);
    assert_eq!(out.code, 0, "stderr: {}", out.stderr);
    std::thread::sleep(Duration::from_millis(300));
    assert!(!marker.exists(), "--dry-run NO debe lanzar typshow");
}
