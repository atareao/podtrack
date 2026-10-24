//! Utilidades compartidas por la suite de paridad (fixtures temporales).
//!
//! **Ningún test toca el árbol real de notas**: cada fixture vive en un `tempfile`.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use assert_cmd::Command;
use tempfile::TempDir;

/// Tracker mínimo del oráculo (`MINI_YAML`), sin slugs.
pub const MINI_YAML: &str = "\
# podcast-tracker.yaml
# Estado del podcast \"Show de prueba\"
# Temporadas: empiezan el 1 de septiembre
# Formato de episodio: XXXX (4 dígitos)

season: 8
season_start: 2026-09-01
episode_duration_minutes: 25
publish_days: [Monday, Thursday]

episodes:
  - number: 100
    season: 8
    date: 2026-09-03
    title: \"Primero\"
    status: published
  - number: 101
    season: 8
    date: 2026-09-07
    title: \"Segundo\"
    status: scripted
  - number: 102
    season: 8
    date: 2026-09-10
    title: \"Tercero\"
    status: scripted
";

/// Tracker tolerante (`MINI_YAML_V2`): `number: null` y `slug` opcional.
pub const MINI_YAML_V2: &str = "\
season: 9
season_start: 2026-09-03
episode_duration_minutes: 25
publish_days: [Monday, Thursday]

episodes:
  - number: 200
    slug: primer-episodio
    season: 8
    date: 2025-09-04
    title: \"Primero\"
    status: published
  - number: null
    slug: sin-numero
    season: 9
    date: 2026-09-07
    title: \"Sin número\"
    status: scripted
  - number: 202
    season: 9
    date: 2026-09-10
    title: \"Tercero\"
    status: recorded
";

/// Tracker con un publicado y dos provisionales con slug (`B4_YAML`).
pub const B4_YAML: &str = "\
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
    slug: provisional-uno
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

/// Fixture: un `guiones/` con tracker y un `shows/<slug>.yaml` apuntándolo.
pub struct Fixture {
    /// Directorio temporal raíz (se borra al soltar).
    pub dir: TempDir,
    /// Raíz de guiones (directorio padre del tracker).
    pub root: PathBuf,
    /// Directorio de perfiles.
    pub shows: PathBuf,
    /// Ruta del tracker.
    pub tracker: PathBuf,
    /// Slug del show.
    pub slug: String,
}

impl Fixture {
    /// Construye la fixture con un perfil válido.
    pub fn new(yaml: &str, slug: &str, name: &str) -> Self {
        let dir = TempDir::new().expect("tempdir");
        let root = dir.path().join("guiones");
        std::fs::create_dir_all(&root).expect("mkdir guiones");
        let tracker = root.join("podcast-tracker.yaml");
        std::fs::write(&tracker, yaml).expect("write tracker");
        let shows = dir.path().join("shows");
        std::fs::create_dir_all(&shows).expect("mkdir shows");
        std::fs::write(
            shows.join(format!("{slug}.yaml")),
            format!(
                "slug: {slug}\nname: {name}\ntracker: {}\n",
                tracker.display()
            ),
        )
        .expect("write profile");
        Self {
            dir,
            root,
            shows,
            tracker,
            slug: slug.to_string(),
        }
    }

    /// Construye la fixture con el perfil `atareao-con-linux` (show por defecto).
    pub fn default_show(yaml: &str) -> Self {
        Self::new(yaml, "atareao-con-linux", "atareao con Linux")
    }

    /// Un `Command` del binario apuntando a esta fixture.
    pub fn cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("podtrack").expect("bin podtrack");
        cmd.env("PODTRACK_SHOWS_DIR", &self.shows);
        cmd.env("PODTRACK_SHOW", &self.slug);
        cmd.env("HOME", self.dir.path());
        cmd.env_remove("XDG_CONFIG_HOME");
        cmd
    }
}

/// Ejecuta el binario esperando éxito y devuelve su `stdout`.
pub fn run_ok(fx: &Fixture, args: &[&str]) -> String {
    let output = fx.cmd().args(args).output().expect("run");
    assert!(
        output.status.success(),
        "se esperaba éxito para {args:?}; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("utf8")
}

/// Ejecuta el binario esperando fallo y devuelve `(código, stdout, stderr)`.
pub fn run_fail(fx: &Fixture, args: &[&str]) -> (i32, String, String) {
    let output = fx.cmd().args(args).output().expect("run");
    let code = output.status.code().expect("exit code");
    assert_ne!(code, 0, "se esperaba fallo para {args:?}");
    (
        code,
        String::from_utf8(output.stdout).expect("utf8"),
        String::from_utf8(output.stderr).expect("utf8"),
    )
}

/// Contenido textual del tracker.
pub fn read_tracker(fx: &Fixture) -> String {
    std::fs::read_to_string(&fx.tracker).expect("read tracker")
}

/// Instantánea ruta-relativa → bytes de todo el árbol de `root`.
pub fn snapshot(root: &PathBuf) -> std::collections::BTreeMap<String, Vec<u8>> {
    let mut map = std::collections::BTreeMap::new();
    for entry in walk(root) {
        if entry.is_file() {
            let rel = entry
                .strip_prefix(root)
                .expect("rel")
                .to_string_lossy()
                .to_string();
            map.insert(rel, std::fs::read(&entry).expect("read"));
        }
    }
    map
}

fn walk(root: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path.clone());
            }
            out.push(path);
        }
    }
    out.sort();
    out
}

// ── Guardián del oráculo (paridad en vivo) ──────────────────────────────────
//
// Los tests que comparan contra `podcast-tracker.py` necesitan Python + PyYAML y
// el propio script. Cuando no están, el patrón por defecto es **saltar** el test
// (para no romper el CI, donde no existe la skill). Pero un test que se salta en
// silencio puede pasar sin comprobar nada: por eso, si `PODTRACK_REQUIRE_ORACLE`
// está definida y no vacía, el test **falla** con un mensaje claro en vez de
// saltar. Así el operador exige el guardián con `PODTRACK_REQUIRE_ORACLE=1 cargo test`
// y obtiene certeza de que la comparación se ejecutó de verdad.

/// ¿Se exige el oráculo (variable `PODTRACK_REQUIRE_ORACLE` no vacía)?
fn require_oracle_env() -> bool {
    std::env::var_os("PODTRACK_REQUIRE_ORACLE").is_some_and(|v| !v.is_empty())
}

/// Ruta del oráculo: `PODTRACK_ORACLE` o el hogar habitual.
pub fn oracle_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("PODTRACK_ORACLE") {
        let path = PathBuf::from(path);
        return path.is_file().then_some(path);
    }
    let home = std::env::var_os("HOME")?;
    let path = PathBuf::from(home).join(".agents/skills/podcast-srt/podcast-tracker.py");
    path.is_file().then_some(path)
}

/// Comprueba que Python y PyYAML están operativos.
pub fn python_ready() -> bool {
    ProcessCommand::new("python3")
        .args(["-c", "import yaml"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Resuelve el oráculo si está disponible; si no, **salta** el test (imprimiendo
/// el motivo) salvo que `PODTRACK_REQUIRE_ORACLE` lo exija, en cuyo caso falla.
pub fn require_oracle() -> Option<PathBuf> {
    let (available, reason) = match oracle_path() {
        Some(module) if python_ready() => (Some(module), String::new()),
        Some(_) => (
            None,
            "python3 + PyYAML no disponibles".to_string(),
        ),
        None => (
            None,
            "el oráculo Python no está disponible (PODTRACK_ORACLE o ~/.agents/skills/podcast-srt/podcast-tracker.py)"
                .to_string(),
        ),
    };

    if available.is_none() {
        if require_oracle_env() {
            panic!("PODTRACK_REQUIRE_ORACLE está activo pero {reason}");
        }
        eprintln!("skip: {reason}");
    }
    available
}

/// Cuerpo del harness Python que carga el módulo del oráculo por ruta y deja
/// fijados `TRACKER_PATH` y `SHOW_NAME` para invocar sus funciones sin `main()`.
pub const ORACLE_PREAMBLE: &str = r#"import importlib.util, sys
from pathlib import Path
spec = importlib.util.spec_from_file_location("pt", sys.argv[1])
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
m.TRACKER_PATH = Path(sys.argv[2])
m.SHOW_NAME = sys.argv[3]
"#;

/// Ejecuta un harness Python (fichero) con el módulo, tracker y nombre del show.
pub fn run_python_harness(
    harness: &Path,
    module: &Path,
    tracker: &Path,
    name: &str,
) -> std::process::Output {
    ProcessCommand::new("python3")
        .arg(harness)
        .arg(module)
        .arg(tracker)
        .arg(name)
        .output()
        .expect("python3")
}
