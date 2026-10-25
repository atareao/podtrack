//! `show-profile` — cascada de resolución, show por defecto y fallo alto y claro.

mod common;

use std::path::PathBuf;

use assert_cmd::Command;
use common::{B4_YAML, Fixture, MINI_YAML, MINI_YAML_V2};
use podtrack::profile::ShowResolver;
use tempfile::TempDir;

/// Escribe un perfil `<slug>.yaml` en `dir` con el contenido dado.
fn write_profile(dir: &std::path::Path, slug: &str, content: &str) {
    std::fs::create_dir_all(dir).expect("mkdir");
    std::fs::write(dir.join(format!("{slug}.yaml")), content).expect("write profile");
}

#[test]
fn show_selecciona_otro_tracker() {
    let fx = Fixture::new(MINI_YAML, "demo", "Otro Show");
    let out = common::run_ok(&fx, &["status", "--show", "demo"]);
    assert!(out.contains("Otro Show"), "salida: {out}");
}

#[test]
fn show_aceptado_antes_del_subcomando() {
    let fx = Fixture::new(MINI_YAML, "demo", "Otro Show");
    let out = common::run_ok(&fx, &["--show", "demo", "next"]);
    assert!(!out.contains("Otro Show"), "`next` no imprime el nombre");
    assert!(out.contains("Siguiente episodio"), "salida: {out}");
}

#[test]
fn la_cascada_prefiere_el_directorio_del_entorno() {
    let tmp = TempDir::new().expect("tmp");
    let env_dir = tmp.path().join("env");
    let exe_dir = tmp.path().join("exe");
    let config_dir = tmp.path().join("config");
    write_profile(
        &env_dir,
        "demo",
        "slug: demo\nname: Desde Env\ntracker: /tmp/t.yaml\n",
    );
    write_profile(
        &exe_dir.join("shows"),
        "demo",
        "slug: demo\nname: Desde Exe\ntracker: /tmp/t.yaml\n",
    );
    write_profile(
        &config_dir,
        "demo",
        "slug: demo\nname: Desde Config\ntracker: /tmp/t.yaml\n",
    );

    let resolver = ShowResolver {
        shows_dir: Some(env_dir.clone()),
        exe_dir: Some(exe_dir),
        config_dir: Some(config_dir),
        default_slug: "demo".to_string(),
    };
    let resolved = resolver.resolve(Some("demo")).expect("resolve");
    assert_eq!(resolved.name, "Desde Env");
}

#[test]
fn la_cascada_cae_al_ejecutable_si_no_hay_entorno() {
    let tmp = TempDir::new().expect("tmp");
    let exe_dir = tmp.path().join("exe");
    let config_dir = tmp.path().join("config");
    write_profile(
        &exe_dir.join("shows"),
        "demo",
        "slug: demo\nname: Desde Exe\ntracker: /tmp/t.yaml\n",
    );
    write_profile(
        &config_dir,
        "demo",
        "slug: demo\nname: Desde Config\ntracker: /tmp/t.yaml\n",
    );

    let resolver = ShowResolver {
        shows_dir: None,
        exe_dir: Some(exe_dir),
        config_dir: Some(config_dir),
        default_slug: "demo".to_string(),
    };
    assert_eq!(
        resolver.resolve(Some("demo")).expect("resolve").name,
        "Desde Exe"
    );
}

#[test]
fn la_cascada_cae_al_directorio_de_config() {
    let tmp = TempDir::new().expect("tmp");
    let config_dir = tmp.path().join("config");
    write_profile(
        &config_dir,
        "demo",
        "slug: demo\nname: Desde Config\ntracker: /tmp/t.yaml\n",
    );
    let resolver = ShowResolver {
        shows_dir: None,
        exe_dir: None,
        config_dir: Some(config_dir),
        default_slug: "demo".to_string(),
    };
    assert_eq!(
        resolver.resolve(Some("demo")).expect("resolve").name,
        "Desde Config"
    );
}

#[test]
fn el_slug_por_defecto_se_usa_sin_show() {
    let tmp = TempDir::new().expect("tmp");
    let env_dir = tmp.path().join("env");
    write_profile(
        &env_dir,
        "mi-podcast",
        "slug: x\nname: Mi Podcast\ntracker: /tmp/t.yaml\n",
    );
    let resolver = ShowResolver {
        shows_dir: Some(env_dir),
        exe_dir: None,
        config_dir: None,
        default_slug: "mi-podcast".to_string(),
    };
    assert_eq!(resolver.resolve(None).expect("resolve").name, "Mi Podcast");
}

#[test]
fn perfil_ausente_falla_nombrando_el_slug() {
    let fx = Fixture::new(MINI_YAML, "demo", "Show de prueba");
    let (code, _out, err) = common::run_fail(&fx, &["status", "--show", "no-existe"]);
    assert_eq!(code, 2, "perfil explícito inexistente → exit 2");
    assert!(err.contains("no-existe"), "el error nombra el slug: {err}");
}

#[test]
fn perfil_sin_clave_tracker_falla_nombrando_la_causa() {
    let tmp = TempDir::new().expect("tmp");
    let shows = tmp.path().join("shows");
    write_profile(&shows, "sin-clave", "slug: sin-clave\nname: Sin Clave\n");
    let output = Command::cargo_bin("podtrack")
        .expect("bin")
        .args(["status", "--show", "sin-clave"])
        .env("PODTRACK_SHOWS_DIR", &shows)
        .env("HOME", tmp.path())
        .env_remove("XDG_CONFIG_HOME")
        .output()
        .expect("run");
    assert_ne!(output.status.code(), Some(0));
    let err = String::from_utf8(output.stderr).expect("utf8");
    assert!(
        err.contains("'tracker'"),
        "nombra la clave que falta: {err}"
    );
    assert!(err.contains("sin-clave"), "nombra el perfil: {err}");
}

#[test]
fn perfil_invalido_falla_nombrando_la_causa() {
    let tmp = TempDir::new().expect("tmp");
    let shows = tmp.path().join("shows");
    write_profile(&shows, "roto", "slug: roto\nname: \"sin cerrar\n");
    let output = Command::cargo_bin("podtrack")
        .expect("bin")
        .args(["status", "--show", "roto"])
        .env("PODTRACK_SHOWS_DIR", &shows)
        .env("HOME", tmp.path())
        .env_remove("XDG_CONFIG_HOME")
        .output()
        .expect("run");
    assert_eq!(output.status.code(), Some(2));
    let err = String::from_utf8(output.stderr).expect("utf8");
    assert!(err.contains("roto"), "nombra el perfil: {err}");
}

#[test]
fn perfil_con_tracker_none_sale_con_codigo_tres() {
    let tmp = TempDir::new().expect("tmp");
    let shows = tmp.path().join("shows");
    write_profile(
        &shows,
        "ninguno",
        "slug: ninguno\nname: Ninguno\ntracker: none\n",
    );
    let output = Command::cargo_bin("podtrack")
        .expect("bin")
        .args(["status", "--show", "ninguno"])
        .env("PODTRACK_SHOWS_DIR", &shows)
        .env("HOME", tmp.path())
        .env_remove("XDG_CONFIG_HOME")
        .output()
        .expect("run");
    assert_eq!(output.status.code(), Some(3));
    let err = String::from_utf8(output.stderr).expect("utf8");
    assert!(err.contains("no usa tracker"), "salida: {err}");
}

#[test]
fn sin_show_ni_entorno_se_usa_el_defecto_compilado() {
    let fx = Fixture::default_show(MINI_YAML);
    // Sin `--show` y sin `PODTRACK_SHOW`: debe caer al slug compilado.
    let output = fx
        .cmd()
        .env_remove("PODTRACK_SHOW")
        .arg("status")
        .output()
        .expect("run");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let out = String::from_utf8(output.stdout).expect("utf8");
    assert!(out.contains("atareao con Linux"), "salida: {out}");
}

#[test]
fn sin_show_se_usa_podtrack_show_del_entorno() {
    let fx = Fixture::new(MINI_YAML, "mi-podcast", "Mi Podcast");
    // `cmd()` fija `PODTRACK_SHOW`; sin `--show` debe cargar ese perfil.
    let out = common::run_ok(&fx, &["status"]);
    assert!(out.contains("Mi Podcast"), "salida: {out}");
}

#[test]
fn show_acepta_una_ruta_absoluta_al_perfil() {
    let tmp = TempDir::new().expect("tmp");
    let guiones = tmp.path().join("guiones");
    std::fs::create_dir_all(&guiones).expect("mkdir");
    let tracker = guiones.join("podcast-tracker.yaml");
    std::fs::write(&tracker, MINI_YAML).expect("tracker");
    let profile = tmp.path().join("absoluto.yaml");
    std::fs::write(
        &profile,
        format!(
            "slug: absoluto\nname: Perfil Absoluto\ntracker: {}\n",
            tracker.display()
        ),
    )
    .expect("profile");

    let output = Command::cargo_bin("podtrack")
        .expect("bin")
        .args(["status", "--show", profile.to_str().expect("utf8")])
        .env("HOME", tmp.path())
        .env_remove("XDG_CONFIG_HOME")
        .output()
        .expect("run");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let out = String::from_utf8(output.stdout).expect("utf8");
    assert!(out.contains("Perfil Absoluto"), "salida: {out}");
}

// ── `profile-absolute-paths`: el tracker del perfil se resuelve a absoluto ──

/// Declaración de `tracker:` **relativa** al directorio del propio perfil.
const REL_TRACKER: &str = "../guiones/podcast-tracker.yaml";

/// Fixture con un perfil que declara `tracker:` relativo. Si `tracker_yaml` es
/// `Some`, se crea además el fichero apuntado en `guiones/`.
struct RelFixture {
    dir: TempDir,
    shows: PathBuf,
}

impl RelFixture {
    fn new(tracker_decl: &str, tracker_yaml: Option<&str>) -> Self {
        let dir = TempDir::new().expect("tmp");
        let shows = dir.path().join("shows");
        std::fs::create_dir_all(&shows).expect("mkdir shows");
        std::fs::write(
            shows.join("demo.yaml"),
            format!("slug: demo\nname: Show Demo\ntracker: {tracker_decl}\n"),
        )
        .expect("write profile");
        if let Some(yaml) = tracker_yaml {
            let guiones = dir.path().join("guiones");
            std::fs::create_dir_all(&guiones).expect("mkdir guiones");
            std::fs::write(guiones.join("podcast-tracker.yaml"), yaml).expect("write tracker");
        }
        Self { dir, shows }
    }

    fn cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("podtrack").expect("bin podtrack");
        cmd.env("PODTRACK_SHOWS_DIR", &self.shows);
        cmd.env("PODTRACK_SHOW", "demo");
        cmd.env("HOME", self.dir.path());
        cmd.env_remove("XDG_CONFIG_HOME");
        cmd
    }

    /// Raíz de guiones: `<tmp>/guiones` (padre absoluto del tracker relativo).
    fn guiones(&self) -> PathBuf {
        self.dir.path().join("guiones")
    }

    fn run_ok(&self, args: &[&str]) -> String {
        let output = self.cmd().args(args).output().expect("run");
        assert!(
            output.status.success(),
            "se esperaba éxito para {args:?}; stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).expect("utf8")
    }
}

#[test]
fn where_con_tracker_relativo_imprime_ruta_absoluta() {
    let fx = RelFixture::new(REL_TRACKER, Some(MINI_YAML_V2));
    let out = fx.run_ok(&["where", "200"]);
    let esperado = fx.guiones().join("primer-episodio");
    assert_eq!(
        out.trim(),
        format!("{} no existe", esperado.display()),
        "salida: {out}"
    );
}

#[test]
fn get_con_tracker_relativo_imprime_ruta_absoluta() {
    let fx = RelFixture::new(REL_TRACKER, Some(MINI_YAML_V2));
    let out = fx.run_ok(&["get", "200"]);
    let esperado = format!(
        "  {:<10} {}",
        "Ruta:",
        fx.guiones().join("primer-episodio").display()
    );
    assert!(
        out.lines().any(|l| l == esperado.as_str()),
        "falta `Ruta:` absoluta; salida:\n{out}"
    );
}

#[test]
fn get_json_con_tracker_relativo_lleva_material_absoluto() {
    let fx = RelFixture::new(REL_TRACKER, Some(MINI_YAML_V2));
    let out = fx.run_ok(&["get", "200", "--json"]);
    let doc: serde_json::Value = serde_json::from_str(&out).expect("json");
    let material = doc["episode"]["material"].as_str().expect("material");
    assert_eq!(
        material,
        fx.guiones().join("primer-episodio").display().to_string()
    );
}

#[test]
fn status_con_tracker_relativo_imprime_ruta_absoluta() {
    let fx = RelFixture::new(REL_TRACKER, Some(B4_YAML));
    let out = fx.run_ok(&["status"]);
    let esperado = format!("ruta:  {}", fx.guiones().join("provisional-uno").display());
    assert!(out.contains(&esperado), "salida:\n{out}");
}

#[test]
fn tracker_relativo_inexistente_nombra_la_ruta_absoluta() {
    let fx = RelFixture::new("no/existe.yaml", None);
    let output = fx.cmd().args(["status"]).output().expect("run");
    assert_eq!(output.status.code(), Some(1), "tracker ausente → exit 1");
    let err = String::from_utf8(output.stderr).expect("utf8");
    let absoluta = fx.shows.join("no/existe.yaml");
    assert!(
        err.contains(&absoluta.display().to_string()),
        "el error debe nombrar la ruta absoluta ({absoluta:?}): {err}"
    );
    // La resolución es puramente léxica: NO crea nada (ni el directorio del
    // `no/`, ni `guiones/`, que en esta fixture no existe de partida).
    assert!(
        !fx.shows.join("no").exists(),
        "no debe crearse el directorio `no/`"
    );
    assert!(!fx.guiones().exists(), "no debe crearse `guiones/`");
}

#[test]
fn scripts_root_con_tracker_sin_directorio_no_cae_al_directorio_actual() {
    // Scenario «`scripts_root` nunca cae al directorio actual»: el GIVEN es un
    // `tracker:` sin componente de directorio. La base es el directorio del
    // perfil, así que el tracker vive en el propio `shows/`.
    let fx = RelFixture::new("podcast-tracker.yaml", None);
    std::fs::write(fx.shows.join("podcast-tracker.yaml"), MINI_YAML_V2).expect("write tracker");

    let out = fx.run_ok(&["where", "200"]);
    let ruta = out
        .trim()
        .strip_suffix(" no existe")
        .expect("sufijo « no existe»");
    let esperada = fx.shows.join("primer-episodio");

    assert!(
        ruta.starts_with('/'),
        "debe ser absoluta (`/` inicial): {ruta}"
    );
    assert!(!ruta.starts_with("./"), "no debe empezar por `./`: {ruta}");
    assert!(!ruta.contains("/./"), "sin componentes `.`: {ruta}");
    assert!(!ruta.contains("shows/../"), "sin `..` de escape: {ruta}");
    assert_eq!(ruta, esperada.display().to_string(), "salida: {out}");
    assert!(
        !ruta.contains(env!("CARGO_MANIFEST_DIR")),
        "no debe ser el directorio de trabajo del repo: {ruta}"
    );
}

#[test]
fn tracker_absoluto_limpio_se_conserva_identico() {
    // Control de no-regresión: la normalización de un absoluto limpio es la
    // identidad, así que la salida no cambia un byte respecto de la declaración.
    let fx = RelFixture::new(REL_TRACKER, Some(MINI_YAML_V2));
    let tracker = fx.guiones().join("podcast-tracker.yaml");
    std::fs::write(
        fx.shows.join("demo.yaml"),
        format!(
            "slug: demo\nname: Show Demo\ntracker: {}\n",
            tracker.display()
        ),
    )
    .expect("rewrite profile");
    let out = fx.run_ok(&["where", "200"]);
    assert_eq!(
        out.trim(),
        format!(
            "{} no existe",
            fx.guiones().join("primer-episodio").display()
        )
    );
}
