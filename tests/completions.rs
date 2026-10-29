//! Contrato del autocompletado de shell (capacidad `shell-completions`).
//!
//! Dos caminos emiten el **mismo** script de registro —`podtrack completions <shell>` y el
//! protocolo `COMPLETE=<shell> podtrack --`— y en ambos el script es determinista y nombra al
//! binario como `podtrack`, nunca por su ruta. Cuando el shell llama al binario, el completado lo
//! resuelve el propio binario: estático (subcomandos, flags y los valores de `value_parser`) y
//! dinámico (el `target` de los siete comandos que lo aceptan y el valor de `--show`).
//!
//! Nada de esto escribe en `tests/golden/` ni toca `/data/notas`.

mod common;

use std::path::Path;
use std::process::Command;

use assert_cmd::Command as Bin;
use common::{Fixture, MINI_YAML_V2};
use tempfile::TempDir;

/// Ruta del binario bajo prueba (la fija `cargo` para los tests de integración).
const EXE: &str = env!("CARGO_BIN_EXE_podtrack");

/// Los siete subcomandos que aceptan un `target` por número o slug.
const CON_TARGET: [&str; 7] = [
    "get",
    "where",
    "open",
    "neighborhood",
    "update",
    "publish",
    "move",
];

/// Comando del binario **sin ningún show resoluble** en el entorno.
fn sin_show(dir: &Path) -> Bin {
    let mut cmd = Bin::cargo_bin("podtrack").expect("bin podtrack");
    cmd.current_dir(dir);
    cmd.env("HOME", dir);
    cmd.env_remove("XDG_CONFIG_HOME");
    cmd.env_remove("PODTRACK_SHOWS_DIR");
    cmd.env_remove("PODTRACK_SHOW");
    cmd.env_remove("COMPLETE");
    cmd
}

/// Ejecuta el protocolo de completado: `COMPLETE=<shell>` + `podtrack -- <palabras…>`.
///
/// `palabras` empieza por el nombre del binario y termina en la palabra que se está completando
/// (normalmente vacía), que es la del índice.
fn completar(fixture: &Fixture, shell: &str, palabras: &[&str]) -> std::process::Output {
    let mut cmd = fixture.cmd();
    cmd.env_remove("COMPLETE");
    cmd.env_remove("_CLAP_COMPLETE_INDEX");
    cmd.env_remove("_CLAP_IFS");
    cmd.env_remove("_CLAP_COMPLETE_COMP_TYPE");
    cmd.env_remove("_CLAP_COMPLETE_SPACE");
    cmd.env("COMPLETE", shell);
    cmd.env("_CLAP_COMPLETE_INDEX", (palabras.len() - 1).to_string());
    cmd.arg("--");
    cmd.args(palabras);
    cmd.output().expect("ejecutar podtrack")
}

/// Candidatos de una salida de completado: las líneas que no son flags.
fn candidatos(out: &std::process::Output) -> Vec<String> {
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('-'))
        .map(str::to_string)
        .collect()
}

/// `stdout` de una salida como texto.
fn salida(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// `stderr` de una salida como texto.
fn errores(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// Directorio temporal suelto (para los casos que no necesitan un show).
fn tmp() -> TempDir {
    TempDir::new().expect("tempdir")
}

// ── El script de registro ─────────────────────────────────────────────────────

#[test]
fn los_tres_shells_emiten_un_script_sin_necesitar_un_show() {
    let dir = tmp();
    for shell in ["bash", "zsh", "fish"] {
        let out = sin_show(dir.path())
            .args(["completions", shell])
            .output()
            .expect("ejecutar podtrack");
        assert_eq!(out.status.code(), Some(0), "shell {shell}");
        assert!(!salida(&out).is_empty(), "shell {shell}: script vacío");
        assert!(errores(&out).is_empty(), "shell {shell}: {}", errores(&out));
    }
}

#[test]
fn el_script_es_determinista_y_no_lleva_la_ruta_del_binario() {
    let uno = tmp();
    let otro = tmp();
    let a = sin_show(uno.path())
        .args(["completions", "bash"])
        .output()
        .expect("ejecutar podtrack");
    let b = sin_show(otro.path())
        .args(["completions", "bash"])
        .output()
        .expect("ejecutar podtrack");
    let script = salida(&a);
    assert!(!script.is_empty(), "script vacío");
    assert_eq!(script, salida(&b), "el script debe ser determinista");
    assert!(
        script.contains("podtrack"),
        "el script debe nombrar el binario"
    );
    assert!(
        !script.contains(EXE),
        "el script no debe llevar la ruta del binario"
    );
}

#[test]
fn el_subcomando_y_el_protocolo_de_entorno_emiten_el_mismo_script() {
    let dir = tmp();
    let sub = sin_show(dir.path())
        .args(["completions", "bash"])
        .output()
        .expect("ejecutar podtrack");
    let env = sin_show(dir.path())
        .arg("--")
        .env("COMPLETE", "bash")
        .output()
        .expect("ejecutar podtrack");
    assert!(!salida(&sub).is_empty(), "script del subcomando vacío");
    assert!(!salida(&env).is_empty(), "script del protocolo vacío");
    assert_eq!(salida(&sub), salida(&env));
}

#[test]
fn el_script_se_evalua_en_bash_y_registra_el_completado() {
    let dir = tmp();
    let script = format!("source <('{EXE}' completions bash) && complete -p podtrack");
    let out = Command::new("bash")
        .arg("-c")
        .arg(script)
        .current_dir(dir.path())
        .output()
        .expect("bash");
    assert!(
        out.status.success(),
        "bash falló: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("podtrack"));
}

#[test]
fn un_shell_no_soportado_es_un_error_de_uso() {
    let dir = tmp();
    let out = sin_show(dir.path())
        .args(["completions", "tcsh"])
        .output()
        .expect("ejecutar podtrack");
    assert_eq!(out.status.code(), Some(2));
    assert!(salida(&out).is_empty(), "no debe haber script");
    let err = errores(&out);
    assert!(err.contains("tcsh"), "nombra el valor rechazado: {err}");
    assert!(
        err.contains("bash") && err.contains("fish"),
        "lista los shells válidos: {err}"
    );
}

/// El protocolo de entorno sirve exactamente los mismos shells que acepta el subcomando.
#[test]
fn el_protocolo_de_entorno_rechaza_un_shell_no_soportado() {
    let dir = tmp();
    let out = sin_show(dir.path())
        .arg("--")
        .env("COMPLETE", "elvish")
        .output()
        .expect("ejecutar podtrack");
    assert_eq!(
        out.status.code(),
        Some(2),
        "un shell no soportado es un error de uso"
    );
    assert!(salida(&out).is_empty(), "no debe emitir script");
    let err = errores(&out);
    assert!(
        !err.contains("powershell"),
        "solo se anuncian los tres shells soportados: {err}"
    );
}

// ── El completado dinámico ────────────────────────────────────────────────────

#[test]
fn el_target_de_get_se_completa_con_los_numeros_y_los_slugs() {
    let fx = Fixture::default_show(MINI_YAML_V2);
    let out = completar(&fx, "bash", &["podtrack", "get", ""]);
    assert_eq!(out.status.code(), Some(0));
    assert!(errores(&out).is_empty(), "stderr: {}", errores(&out));
    assert_eq!(
        candidatos(&out),
        vec!["200", "primer-episodio", "sin-numero", "202"]
    );
}

#[test]
fn el_prefijo_filtra_los_candidatos() {
    let fx = Fixture::default_show(MINI_YAML_V2);
    let out = completar(&fx, "bash", &["podtrack", "get", "primer"]);
    assert_eq!(candidatos(&out), vec!["primer-episodio"]);
}

#[test]
fn todos_los_comandos_con_target_se_completan() {
    let fx = Fixture::default_show(MINI_YAML_V2);
    for sub in CON_TARGET {
        let out = completar(&fx, "bash", &["podtrack", sub, ""]);
        let c = candidatos(&out);
        assert!(c.contains(&"200".to_string()), "{sub}: {c:?}");
        assert!(c.contains(&"primer-episodio".to_string()), "{sub}: {c:?}");
    }
}

#[test]
fn sin_show_resoluble_el_completado_queda_vacio_y_en_silencio() {
    let fx = Fixture::default_show(MINI_YAML_V2);
    let vacio = fx.dir.path().join("vacio");
    std::fs::create_dir_all(&vacio).expect("mkdir");
    let mut cmd = fx.cmd();
    cmd.env_remove("COMPLETE");
    cmd.env("PODTRACK_SHOWS_DIR", &vacio);
    cmd.env("COMPLETE", "bash");
    cmd.env("_CLAP_COMPLETE_INDEX", "2");
    cmd.arg("--");
    cmd.args(["podtrack", "get", ""]);
    let out = cmd.output().expect("ejecutar podtrack");
    assert_eq!(out.status.code(), Some(0));
    assert!(candidatos(&out).is_empty(), "no debe haber candidatos");
    assert!(errores(&out).is_empty(), "stderr: {}", errores(&out));
}

#[test]
fn el_valor_de_show_se_completa_con_los_perfiles_de_la_cascada() {
    let fx = Fixture::default_show(MINI_YAML_V2);
    std::fs::write(fx.shows.join("otro.yaml"), "name: Otro\ntracker: x.yaml\n").expect("write");
    let out = completar(&fx, "bash", &["podtrack", "--show", ""]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(candidatos(&out), vec!["atareao-con-linux", "otro"]);
}

#[test]
fn los_slugs_duplicados_se_colapsan() {
    let fx = Fixture::default_show(MINI_YAML_V2);
    let config = fx.dir.path().join(".config/podtrack/shows");
    std::fs::create_dir_all(&config).expect("mkdir");
    std::fs::write(
        config.join("atareao-con-linux.yaml"),
        "name: Duplicado\ntracker: y.yaml\n",
    )
    .expect("write");
    std::fs::write(config.join("otro.yaml"), "name: Otro\ntracker: z.yaml\n").expect("write");
    let out = completar(&fx, "bash", &["podtrack", "--show", ""]);
    assert_eq!(candidatos(&out), vec!["atareao-con-linux", "otro"]);
}

// ── No interferencia ──────────────────────────────────────────────────────────

#[test]
fn un_complete_vacio_o_cero_no_completa_y_deja_correr_el_comando() {
    let fx = Fixture::default_show(MINI_YAML_V2);
    let referencia = fx
        .cmd()
        .env_remove("COMPLETE")
        .args(["list"])
        .output()
        .expect("ejecutar podtrack");
    assert_eq!(referencia.status.code(), Some(0));
    for valor in ["", "0"] {
        let out = fx
            .cmd()
            .env("COMPLETE", valor)
            .args(["list"])
            .output()
            .expect("ejecutar podtrack");
        assert_eq!(out.status.code(), Some(0), "COMPLETE={valor:?}");
        assert_eq!(salida(&out), salida(&referencia), "COMPLETE={valor:?}");
    }
}
