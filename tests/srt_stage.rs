//! `srt-stage` — clasificación de la etapa de un SRT por contenido, con contraste
//! en vivo contra `srt-stage.py`.

mod common;

use std::path::PathBuf;
use std::process::Command;

use common::Outcome;
use podtrack::srt_stage::{
    classify, first_cue_span, first_cue_texts, load_stages, normalize, srt_files_for_episode,
    srt_has_cues,
};
use tempfile::TempDir;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/srt");

/// Fixture de `srt-stage`: `shows/<slug>.yaml` (de `profiles/`) + un árbol de SRT.
struct Harness {
    dir: TempDir,
    root: PathBuf,
    shows: PathBuf,
    slug: String,
}

impl Harness {
    fn new(profile: &str, slug: &str) -> Self {
        let dir = TempDir::new().expect("tempdir");
        let root = dir.path().join("srt");
        std::fs::create_dir_all(&root).expect("mkdir root");
        let shows = dir.path().join("shows");
        std::fs::create_dir_all(&shows).expect("mkdir shows");
        let template = std::fs::read_to_string(format!("{FIXTURES}/profiles/{profile}.yaml"))
            .expect("profile fixture");
        let content = template.replace("@@ROOT@@", &root.display().to_string());
        std::fs::write(shows.join(format!("{slug}.yaml")), content).expect("write profile");
        Self {
            dir,
            root,
            shows,
            slug: slug.to_string(),
        }
    }

    /// Copia una fixture SRT a `dest` (relativo a la raíz de SRT).
    fn copy(&self, fixture: &str, dest: &str) -> PathBuf {
        let path = self.root.join(dest);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("mkdir");
        }
        std::fs::copy(format!("{FIXTURES}/{fixture}"), &path).expect("copy");
        path
    }

    fn run(&self, args: &[&str]) -> Outcome {
        let output = Command::new(env!("CARGO_BIN_EXE_podtrack"))
            .args(args)
            .env("PODTRACK_SHOWS_DIR", &self.shows)
            .env("PODTRACK_SHOW", &self.slug)
            .env("HOME", self.dir.path())
            .env_remove("XDG_CONFIG_HOME")
            .output()
            .expect("podtrack");
        Outcome {
            code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }

    fn run_ok(&self, args: &[&str]) -> String {
        let outcome = self.run(args);
        assert_eq!(
            outcome.code, 0,
            "se esperaba éxito para {args:?}; stderr: {}",
            outcome.stderr
        );
        outcome.stdout
    }

    fn run_fail(&self, args: &[&str]) -> (i32, String, String) {
        let outcome = self.run(args);
        assert_ne!(outcome.code, 0, "se esperaba fallo para {args:?}");
        (outcome.code, outcome.stdout, outcome.stderr)
    }

    /// Contraste en vivo con el oráculo sobre el mismo fixture.
    ///
    /// Los `args` son los del oráculo (`--show … --srt …`); al binario se le
    /// antepone el subcomando `srt-stage` (el oráculo no tiene subcomandos).
    fn assert_parity(&self, args: &[&str]) {
        let Some(module) = common::require_oracle_srt() else {
            return;
        };
        let harness = self.dir.path().join("srt_harness.py");
        std::fs::write(&harness, common::SRT_MAIN_HARNESS).expect("harness");
        let oracle = common::run_oracle_main(&module, &harness, &self.shows, args);

        let mut full = vec!["srt-stage"];
        full.extend_from_slice(args);
        let lib = self.run(&full);

        assert_eq!(oracle.stdout, lib.stdout, "stdout difiere para {args:?}");
        assert_eq!(oracle.stderr, lib.stderr, "stderr difiere para {args:?}");
        assert_eq!(oracle.code, lib.code, "exit code difiere para {args:?}");
    }
}

/// Fixture completa del perfil `distros` con un árbol de SRT por episodio.
fn distros_harness() -> Harness {
    let h = Harness::new("distros", "distros");
    h.copy("final-intro.srt", "episodio-01/episodio-01.srt");
    h.copy("final-dash.srt", "episodio-02/episodio_02.srt");
    h.copy("raw-conversation.srt", "episodio-02/zzz-bruto.srt");
    h.copy("raw-casual.srt", "episodio-03/a-bruto.srt");
    h.copy("raw-late-intro.srt", "episodio-03/b-bruto.srt");
    h
}

fn srt(yaml: &str) -> serde_yaml::Value {
    serde_yaml::from_str(yaml).expect("yaml")
}

// ─── unitarios ──────────────────────────────────────────────────────

#[test]
fn normalize_colapsa_acentos_y_bom() {
    assert_eq!(
        normalize("\u{feff}Bienvenidos  a  La Era"),
        "bienvenidos a la era"
    );
    assert_eq!(normalize("  ÑANDÚ  "), "nandu");
    assert_eq!(normalize("a\t\tb\nc"), "a b c");
}

#[test]
fn first_cue_span_ignora_cues_de_puntuacion_y_calcula_millis() {
    let h = Harness::new("distros", "distros");
    let path = h.copy("final-dash.srt", "x.srt");
    let span = first_cue_span(&path).expect("span").expect("some");
    assert_eq!(span, (0.0, 25.880));

    // Milisegundos de 1-3 dígitos se rellenan a la derecha (`,` + `5` = 500 ms).
    let path2 = h.root.join("short.srt");
    std::fs::write(&path2, "1\n00:00:01,5 --> 00:00:02,25\nalgo\n").expect("write");
    assert_eq!(first_cue_span(&path2).expect("span"), Some((1.5, 2.25)));
}

#[test]
fn first_cue_texts_ignora_el_cue_de_puntuacion() {
    let h = Harness::new("distros", "distros");
    let path = h.copy("final-dash.srt", "x.srt");
    let cues = first_cue_texts(&path, 2).expect("cues");
    assert_eq!(cues.len(), 2);
    assert!(cues[0].starts_with("Te doy la bienvenida"));
}

#[test]
fn srt_has_cues_rechaza_texto_suelto_y_vacio() {
    let h = Harness::new("distros", "distros");
    assert!(!srt_has_cues(&h.copy("no-cues.srt", "a.srt")).expect("read"));
    assert!(!srt_has_cues(&h.copy("empty.srt", "b.srt")).expect("read"));
    assert!(srt_has_cues(&h.copy("final-intro.srt", "c.srt")).expect("read"));
}

#[test]
fn load_stages_rechaza_detect_desconocido_y_sin_etapas() {
    let bad = srt("episodes:\n  srt_stages:\n    order: [x]\n    x:\n      detect: inventado\n");
    let err = load_stages(&bad).expect_err("err");
    assert_eq!(err.exit_code(), 2);
    assert!(
        err.to_string()
            .contains("'detect' desconocido ('inventado')")
    );

    let none = srt("episodes:\n  root: /x\n");
    assert!(
        load_stages(&none)
            .expect_err("err")
            .to_string()
            .contains("no declara 'episodes.srt_stages'")
    );
}

#[test]
fn classify_las_fixtures_del_perfil_distros() {
    let h = distros_harness();
    let profile = srt(&std::fs::read_to_string(h.shows.join("distros.yaml")).expect("read"));
    let stages = load_stages(&profile).expect("stages");

    for (fixture, expected) in [
        ("final-intro.srt", "final"),
        ("final-dash.srt", "final"),
        ("final-split.srt", "final"),
        ("final-marker.srt", "final"),
        ("raw-conversation.srt", "raw"),
        ("raw-short-intro.srt", "raw"),
        ("raw-casual.srt", "raw"),
        ("raw-late-intro.srt", "raw"),
    ] {
        let path = h.copy(fixture, &format!("c-{fixture}"));
        assert_eq!(
            classify(&path, &stages).expect("classify"),
            expected,
            "fixture {fixture}"
        );
    }

    let empty = h.copy("empty.srt", "c-empty.srt");
    assert!(classify(&empty, &stages).is_err());
}

#[test]
fn srt_files_for_episode_compone_el_glob() {
    let h = distros_harness();
    let profile = srt(&std::fs::read_to_string(h.shows.join("distros.yaml")).expect("read"));
    let files = srt_files_for_episode(&profile, 2).expect("glob");
    assert_eq!(files.len(), 2);
    assert!(files[0].ends_with("episodio-02/episodio_02.srt"));
    assert!(files[1].ends_with("episodio-02/zzz-bruto.srt"));
    assert!(
        srt_files_for_episode(&profile, 99)
            .expect("glob")
            .is_empty()
    );
}

// ─── contrato explícito (CLI) ───────────────────────────────────────

#[test]
fn srt_final_e_imprime_ruta_y_etapa() {
    let h = distros_harness();
    let path = h.copy("final-intro.srt", "solo.srt");
    let out = h.run_ok(&[
        "srt-stage",
        "--show",
        "distros",
        "--srt",
        &path.display().to_string(),
    ]);
    assert_eq!(out, format!("{}\tfinal\n", path.display()));
}

#[test]
fn srt_raw() {
    let h = distros_harness();
    let path = h.copy("raw-conversation.srt", "solo.srt");
    let out = h.run_ok(&[
        "srt-stage",
        "--show",
        "distros",
        "--srt",
        &path.display().to_string(),
    ]);
    assert_eq!(out, format!("{}\traw\n", path.display()));
}

#[test]
fn expect_presente_ok() {
    let h = distros_harness();
    let path = h.copy("final-intro.srt", "solo.srt");
    let out = h.run_ok(&[
        "srt-stage",
        "--show",
        "distros",
        "--srt",
        &path.display().to_string(),
        "--expect",
        "final",
    ]);
    assert!(out.ends_with("OK\n"));
}

#[test]
fn expect_fallida_con_srt_sale_5() {
    let h = distros_harness();
    let path = h.copy("raw-conversation.srt", "solo.srt");
    let (code, out, err) = h.run_fail(&[
        "srt-stage",
        "--show",
        "distros",
        "--srt",
        &path.display().to_string(),
        "--expect",
        "final",
    ]);
    assert_eq!(code, 5);
    assert!(out.ends_with("FALLO\n"));
    assert!(err.contains("Se esperaba la etapa 'final' pero el SRT es de la etapa 'raw'."));
    assert!(err.contains("No generes entregables de una etapa que el SRT no es"));
}

#[test]
fn episode_multiples_srt() {
    let h = distros_harness();
    let out = h.run_ok(&["srt-stage", "--show", "distros", "--episode", "2"]);
    assert_eq!(out.lines().count(), 2);
    assert!(out.contains("\tfinal"));
    assert!(out.contains("\traw"));
}

#[test]
fn episode_sin_srt_sale_2() {
    let h = distros_harness();
    let (code, _o, err) = h.run_fail(&["srt-stage", "--show", "distros", "--episode", "99"]);
    assert_eq!(code, 2);
    assert!(err.contains("No se encontró ningún SRT del episodio 99"));
}

#[test]
fn episode_expect_ausente_sale_4() {
    let h = distros_harness();
    let (code, out, err) = h.run_fail(&[
        "srt-stage",
        "--show",
        "distros",
        "--episode",
        "3",
        "--expect",
        "final",
    ]);
    assert_eq!(code, 4);
    assert!(out.ends_with("FALLO\n"));
    assert!(err.contains("Ningún SRT del episodio 3 es de la etapa 'final'."));
}

#[test]
fn sin_srt_ni_episode_sale_2() {
    let h = distros_harness();
    let (code, _o, err) = h.run_fail(&["srt-stage", "--show", "distros"]);
    assert_eq!(code, 2);
    assert!(err.contains("Indica --srt <ruta> o --episode NN."));
}

#[test]
fn srt_y_episode_a_la_vez_sale_2() {
    let h = distros_harness();
    let (code, _o, err) = h.run_fail(&[
        "srt-stage",
        "--show",
        "distros",
        "--srt",
        "a.srt",
        "--episode",
        "1",
    ]);
    assert_eq!(code, 2);
    assert!(err.contains("--srt y --episode son excluyentes."));
}

#[test]
fn expect_no_declarada_sale_2() {
    let h = Harness::new("one-stage", "one");
    let path = h.copy("final-intro.srt", "solo.srt");
    let (code, _o, err) = h.run_fail(&[
        "srt-stage",
        "--show",
        "one",
        "--srt",
        &path.display().to_string(),
        "--expect",
        "raw",
    ]);
    assert_eq!(code, 2);
    assert!(err.contains("La etapa 'raw' no existe en el perfil 'one'."));
    assert!(err.contains("Etapas declaradas: final"));
}

#[test]
fn fichero_sin_cues_sale_2() {
    let h = distros_harness();
    let path = h.copy("no-cues.srt", "solo.srt");
    let (code, _o, err) = h.run_fail(&[
        "srt-stage",
        "--show",
        "distros",
        "--srt",
        &path.display().to_string(),
    ]);
    assert_eq!(code, 2);
    assert!(err.contains(&format!(
        "El fichero {} no parece un SRT: sin cues.",
        path.display()
    )));
}

#[test]
fn perfil_inexistente_sale_2() {
    let h = distros_harness();
    let path = h.copy("final-intro.srt", "solo.srt");
    let (code, _o, err) = h.run_fail(&[
        "srt-stage",
        "--show",
        "noexiste",
        "--srt",
        &path.display().to_string(),
    ]);
    assert_eq!(code, 2);
    assert!(err.contains("No existe el perfil de podcast 'noexiste'."));
    assert!(err.contains("Perfiles disponibles: distros"));
}

#[test]
fn perfil_de_una_sola_etapa_any() {
    let h = Harness::new("one-stage", "one");
    let path = h.copy("raw-conversation.srt", "solo.srt");
    let out = h.run_ok(&[
        "srt-stage",
        "--show",
        "one",
        "--srt",
        &path.display().to_string(),
    ]);
    assert_eq!(out, format!("{}\tfinal\n", path.display()));
}

#[test]
fn json_con_expect() {
    let h = distros_harness();
    let path = h.copy("final-intro.srt", "solo.srt");
    let out = h.run_ok(&[
        "srt-stage",
        "--show",
        "distros",
        "--srt",
        &path.display().to_string(),
        "--expect",
        "final",
        "--json",
    ]);
    let doc: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert_eq!(doc["schema_version"], 1);
    assert_eq!(doc["show"], "distros");
    assert_eq!(doc["results"][0]["stage"], "final");
    assert_eq!(doc["expect"], "final");
    assert_eq!(doc["ok"], true);
}

#[test]
fn json_sin_expect_no_incluye_ok() {
    let h = distros_harness();
    let path = h.copy("raw-conversation.srt", "solo.srt");
    let out = h.run_ok(&[
        "srt-stage",
        "--show",
        "distros",
        "--srt",
        &path.display().to_string(),
        "--json",
    ]);
    let doc: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert_eq!(doc["results"][0]["stage"], "raw");
    assert!(doc.get("ok").is_none());
}

#[test]
fn json_expect_fallida_mantiene_exit_y_vacio_stderr() {
    let h = distros_harness();
    let path = h.copy("raw-conversation.srt", "solo.srt");
    let outcome = h.run(&[
        "srt-stage",
        "--show",
        "distros",
        "--srt",
        &path.display().to_string(),
        "--expect",
        "final",
        "--json",
    ]);
    assert_eq!(outcome.code, 5);
    let doc: serde_json::Value = serde_json::from_str(&outcome.stdout).expect("json");
    assert_eq!(doc["ok"], false);
    assert_eq!(outcome.stderr, "");
}

#[test]
fn sin_json_no_hay_json() {
    let h = distros_harness();
    let path = h.copy("final-intro.srt", "solo.srt");
    let out = h.run_ok(&[
        "srt-stage",
        "--show",
        "distros",
        "--srt",
        &path.display().to_string(),
    ]);
    assert!(!out.trim_start().starts_with('{'));
}

// ─── contraste en vivo con el oráculo ───────────────────────────────

#[test]
fn oraculo_srt_por_contenido() {
    let h = distros_harness();
    for fixture in [
        "final-intro.srt",
        "final-dash.srt",
        "final-split.srt",
        "final-marker.srt",
        "raw-conversation.srt",
        "raw-short-intro.srt",
        "raw-casual.srt",
        "raw-late-intro.srt",
    ] {
        let path = h.copy(fixture, &format!("o-{fixture}"));
        h.assert_parity(&["--show", "distros", "--srt", &path.display().to_string()]);
        h.assert_parity(&[
            "--show",
            "distros",
            "--srt",
            &path.display().to_string(),
            "--expect",
            "raw",
        ]);
        h.assert_parity(&[
            "--show",
            "distros",
            "--srt",
            &path.display().to_string(),
            "--expect",
            "final",
        ]);
    }
}

#[test]
fn oraculo_episode() {
    let h = distros_harness();
    for episode in ["1", "2", "3", "99"] {
        h.assert_parity(&["--show", "distros", "--episode", episode]);
        h.assert_parity(&[
            "--show",
            "distros",
            "--episode",
            episode,
            "--expect",
            "final",
        ]);
        h.assert_parity(&["--show", "distros", "--episode", episode, "--expect", "raw"]);
    }
}

#[test]
fn oraculo_una_sola_etapa() {
    let h = Harness::new("one-stage", "one");
    let path = h.copy("raw-conversation.srt", "solo.srt");
    h.assert_parity(&["--show", "one", "--srt", &path.display().to_string()]);
    h.assert_parity(&[
        "--show",
        "one",
        "--srt",
        &path.display().to_string(),
        "--expect",
        "final",
    ]);
    h.assert_parity(&[
        "--show",
        "one",
        "--srt",
        &path.display().to_string(),
        "--expect",
        "raw",
    ]);
}

#[test]
fn oraculo_errores() {
    let h = distros_harness();
    let nocues = h.copy("no-cues.srt", "n.srt");
    let empty = h.copy("empty.srt", "e.srt");
    let good = h.copy("final-intro.srt", "g.srt");
    h.assert_parity(&["--show", "distros", "--srt", &nocues.display().to_string()]);
    h.assert_parity(&["--show", "distros", "--srt", &empty.display().to_string()]);
    h.assert_parity(&["--show", "distros", "--srt", "/no/existe.srt"]);
    h.assert_parity(&["--show", "noexiste", "--srt", &good.display().to_string()]);
    h.assert_parity(&["--show", "distros"]);
    h.assert_parity(&[
        "--show",
        "distros",
        "--srt",
        &good.display().to_string(),
        "--episode",
        "1",
    ]);
}

#[test]
fn oraculo_perfiles_rotos() {
    let bad = Harness::new("bad-detect", "bad");
    let path = bad.copy("final-intro.srt", "solo.srt");
    bad.assert_parity(&["--show", "bad", "--srt", &path.display().to_string()]);

    let none = Harness::new("no-stages", "nostages");
    let path2 = none.copy("final-intro.srt", "solo.srt");
    none.assert_parity(&["--show", "nostages", "--srt", &path2.display().to_string()]);
}

// ─── regresión: anclaje por caracteres (no bytes) ───────────────────

#[test]
fn anclaje_cuenta_caracteres_no_bytes() {
    let h = Harness::new("anchor", "anchor");
    // `¿` ocupa 2 bytes y `—` 3; con `intro_anchor_chars: 2` la coincidencia
    // empieza en el carácter 1, así que la señal de texto debe dispararse.
    for fixture in ["final-multibyte-2b.srt", "final-multibyte-3b.srt"] {
        let path = h.copy(fixture, &format!("m-{fixture}"));
        let out = h.run_ok(&[
            "srt-stage",
            "--show",
            "anchor",
            "--srt",
            &path.display().to_string(),
        ]);
        assert_eq!(
            out,
            format!("{}\tfinal\n", path.display()),
            "fixture {fixture}"
        );
    }
}

#[test]
fn oraculo_anclaje_multibyte() {
    let h = Harness::new("anchor", "anchor");
    for fixture in ["final-multibyte-2b.srt", "final-multibyte-3b.srt"] {
        let path = h.copy(fixture, &format!("m-{fixture}"));
        let file = path.display().to_string();
        h.assert_parity(&["--show", "anchor", "--srt", &file]);
        h.assert_parity(&["--show", "anchor", "--srt", &file, "--expect", "final"]);
        h.assert_parity(&["--show", "anchor", "--srt", &file, "--expect", "raw"]);
    }
}

// ─── regresión: cadenas vacías cuentan como ausentes ────────────────

#[test]
fn intro_vacio_cuenta_como_ausente() {
    let h = Harness::new("empty-intro", "emptyintro");
    let path = h.copy("final-intro.srt", "solo.srt");
    let (code, _out, err) = h.run_fail(&[
        "srt-stage",
        "--show",
        "emptyintro",
        "--srt",
        &path.display().to_string(),
    ]);
    assert_eq!(code, 2);
    assert!(err.contains("no declara 'intro_pattern' ni 'intro_marker'"));
}

#[test]
fn oraculo_intro_vacio() {
    let h = Harness::new("empty-intro", "emptyintro");
    let path = h.copy("final-intro.srt", "solo.srt");
    h.assert_parity(&["--show", "emptyintro", "--srt", &path.display().to_string()]);
}

#[test]
fn load_stages_trata_cadenas_vacias_como_ausentes() {
    let both_empty = srt(
        "episodes:\n  srt_stages:\n    order: [x]\n    x:\n      detect: starts_with_intro\n      intro_pattern: \"\"\n      intro_marker: \"\"\n",
    );
    let err = load_stages(&both_empty).expect_err("err");
    assert_eq!(err.exit_code(), 2);
    assert!(
        err.to_string()
            .contains("no declara 'intro_pattern' ni 'intro_marker'")
    );

    // Patrón vacío + marcador presente: válido, y el patrón no se compila.
    let marker_only = srt(
        "episodes:\n  srt_stages:\n    order: [x]\n    x:\n      detect: starts_with_intro\n      intro_pattern: \"\"\n      intro_marker: \"INTRO\"\n      intro_anchor_chars: 12\n",
    );
    let stages = load_stages(&marker_only).expect("stages");
    assert!(stages[0].intro_re.is_none());
    assert_eq!(stages[0].intro_marker.as_deref(), Some("INTRO"));
}
