//! `show-profile` — cascada de resolución, show por defecto y fallo alto y claro.

mod common;

use assert_cmd::Command;
use common::{Fixture, MINI_YAML};
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
