//! `neighborhood` — la ventana numérica alrededor de un episodio, con la
//! referencia marcada con `►`, la tabla de `list` y los errores de resolución.
//! Sin escrituras y sin ruta de material.

mod common;

use std::fs;

use common::{Fixture, read_tracker, run_fail, run_ok, snapshot};

/// Episodio sintético para construir trackers sin repetir YAML a mano.
struct Ep {
    number: Option<u32>,
    slug: Option<&'static str>,
    date: &'static str,
    status: &'static str,
}

/// Episodio publicado con número.
fn publicado(number: u32, date: &'static str) -> Ep {
    Ep {
        number: Some(number),
        slug: None,
        date,
        status: "published",
    }
}

/// Episodio con número y un estado explícito (no publicado).
fn con_estado(number: u32, date: &'static str, status: &'static str) -> Ep {
    Ep {
        number: Some(number),
        slug: None,
        date,
        status,
    }
}

/// Episodio sin número (solo slug): invisible a la vecindad numérica.
fn sin_numero(slug: &'static str, date: &'static str, status: &'static str) -> Ep {
    Ep {
        number: None,
        slug: Some(slug),
        date,
        status,
    }
}

/// Emite el tracker YAML de una lista de episodios sintéticos.
fn tracker_yaml(eps: &[Ep]) -> String {
    let mut yaml = String::from(
        "season: 8\n\
         season_start: 2026-09-01\n\
         episode_duration_minutes: 25\n\
         publish_days: [Monday, Thursday]\n\
         \n\
         episodes:\n",
    );
    for ep in eps {
        match ep.number {
            Some(number) => yaml.push_str(&format!("  - number: {number}\n")),
            None => yaml.push_str("  - number: null\n"),
        }
        if let Some(slug) = ep.slug {
            yaml.push_str(&format!("    slug: {slug}\n"));
        }
        yaml.push_str(&format!(
            "    season: 8\n    date: {}\n    title: \"{}\"\n    status: {}\n",
            ep.date,
            ep.number
                .map(|n| format!("Episodio {n}"))
                .unwrap_or_else(|| "Sin número".to_string()),
            ep.status
        ));
    }
    yaml
}

/// Fixture temporal con el tracker sintético.
fn fixture(eps: &[Ep]) -> Fixture {
    let yaml = tracker_yaml(eps);
    Fixture::new(&yaml, "demo", "Show de prueba")
}

/// Filas de datos de la tabla: la referencia (`► `) o una fila con número
/// (indentación seguida de un dígito). Excluye cabecera, separador y leyenda.
fn filas(out: &str) -> Vec<&str> {
    out.lines()
        .filter(|line| {
            line.starts_with("► ")
                || (line.starts_with(' ')
                    && line.trim_start().starts_with(|c: char| c.is_ascii_digit()))
        })
        .collect()
}

/// Números de las filas de datos, en el orden mostrado (sin la marca `*`).
fn numeros(out: &str) -> Vec<u32> {
    filas(out)
        .iter()
        .map(|line| {
            let sin_marca = line.trim_start_matches(['►', ' ']);
            sin_marca
                .split_whitespace()
                .next()
                .expect("número en la fila")
                .trim_end_matches('*')
                .parse()
                .expect("número entero")
        })
        .collect()
}

/// Línea de cabecera de la vecindad, sin recortar (conserva la indentación).
fn cabecera(out: &str) -> &str {
    out.lines()
        .find(|line| line.contains("Vecindad de #"))
        .expect("cabecera de vecindad")
}

/// Una parrilla de nueve episodios consecutivos, `674..=682`.
fn parrilla_674_682() -> Vec<Ep> {
    vec![
        publicado(674, "2026-09-01"),
        publicado(675, "2026-09-02"),
        publicado(676, "2026-09-03"),
        publicado(677, "2026-09-04"),
        publicado(678, "2026-09-05"),
        publicado(679, "2026-09-06"),
        publicado(680, "2026-09-07"),
        publicado(681, "2026-09-08"),
        publicado(682, "2026-09-09"),
    ]
}

#[test]
fn ventana_centrada_marca_la_referencia() {
    let fx = fixture(&parrilla_674_682());
    let out = run_ok(&fx, &["neighborhood", "678"]);

    assert_eq!(cabecera(&out), "  Vecindad de #678 (antes: 4, después: 4):");
    let filas = filas(&out);
    assert_eq!(filas.len(), 9, "salida:\n{out}");
    assert_eq!(
        numeros(&out),
        vec![674, 675, 676, 677, 678, 679, 680, 681, 682]
    );
    assert!(filas[4].starts_with("► "), "quinta fila: {:?}", filas[4]);
    for (i, fila) in filas.iter().enumerate() {
        if i != 4 {
            assert!(fila.starts_with("  "), "fila {i}: {fila:?}");
        }
    }
}

#[test]
fn resolucion_por_slug_declara_el_numero() {
    let mut eps = parrilla_674_682();
    eps[4].slug = Some("la-memoria-de-valet");
    let fx = fixture(&eps);

    let out = run_ok(&fx, &["neighborhood", "la-memoria-de-valet"]);

    assert_eq!(cabecera(&out), "  Vecindad de #678 (antes: 4, después: 4):");
    assert_eq!(
        numeros(&out),
        vec![674, 675, 676, 677, 678, 679, 680, 681, 682]
    );
}

#[test]
fn huecos_se_rellenan_con_los_existentes_mas_cercanos() {
    let fx = fixture(&[
        publicado(670, "2026-09-01"),
        publicado(672, "2026-09-03"),
        publicado(678, "2026-09-09"),
        publicado(681, "2026-09-12"),
        publicado(690, "2026-09-21"),
    ]);

    let out = run_ok(&fx, &["neighborhood", "678"]);

    assert_eq!(cabecera(&out), "  Vecindad de #678 (antes: 2, después: 2):");
    assert_eq!(filas(&out).len(), 5);
    assert_eq!(numeros(&out), vec![670, 672, 678, 681, 690]);
    let mostrados = numeros(&out);
    for ausente in [673u32, 674, 675, 676, 677, 679, 680] {
        assert!(
            !mostrados.contains(&ausente),
            "no debe haber fila para {ausente}: {out}"
        );
    }
}

#[test]
fn extremo_de_la_parrilla_muestra_lo_que_haya() {
    let fx = fixture(&[
        publicado(100, "2026-09-01"),
        publicado(101, "2026-09-02"),
        publicado(102, "2026-09-03"),
        publicado(103, "2026-09-04"),
        publicado(104, "2026-09-05"),
        publicado(105, "2026-09-06"),
    ]);

    let out = run_ok(&fx, &["neighborhood", "101"]);

    assert_eq!(cabecera(&out), "  Vecindad de #101 (antes: 1, después: 4):");
    assert_eq!(filas(&out).len(), 6);
    assert_eq!(numeros(&out), vec![100, 101, 102, 103, 104, 105]);
}

#[test]
fn before_cero_deja_la_ventana_a_un_lado() {
    let fx = fixture(&parrilla_674_682());

    let out = run_ok(&fx, &["neighborhood", "678", "--before", "0"]);

    assert_eq!(cabecera(&out), "  Vecindad de #678 (antes: 0, después: 4):");
    let filas = filas(&out);
    assert_eq!(filas.len(), 5);
    assert_eq!(numeros(&out), vec![678, 679, 680, 681, 682]);
    assert!(filas[0].starts_with("► "), "primera fila: {:?}", filas[0]);
}

#[test]
fn episodio_sin_numero_no_entra_en_la_ventana() {
    let fx = fixture(&[
        publicado(677, "2026-09-01"),
        sin_numero("en-medio", "2026-09-02", "scripted"),
        publicado(678, "2026-09-03"),
        publicado(679, "2026-09-04"),
    ]);

    let out = run_ok(&fx, &["neighborhood", "678"]);

    assert_eq!(filas(&out).len(), 3);
    assert_eq!(numeros(&out), vec![677, 678, 679]);
    assert!(
        !out.contains("Sin número"),
        "el episodio sin número no debe aparecer:\n{out}"
    );
}

#[test]
fn vecindad_numerica_no_de_parrilla() {
    // Fechas invertidas respecto del número: la ventana manda por número.
    let fx = fixture(&[
        publicado(677, "2026-10-21"),
        publicado(679, "2026-10-20"),
        publicado(678, "2026-10-23"),
    ]);

    let out = run_ok(&fx, &["neighborhood", "678"]);

    assert_eq!(filas(&out).len(), 3);
    assert_eq!(numeros(&out), vec![677, 678, 679]);
    assert_eq!(cabecera(&out), "  Vecindad de #678 (antes: 1, después: 1):");
}

#[test]
fn no_publicados_llevan_marca_provisional() {
    let fx = fixture(&[
        publicado(676, "2026-09-03"),
        publicado(677, "2026-09-04"),
        publicado(678, "2026-09-05"),
        con_estado(679, "2026-09-06", "scripted"),
        publicado(680, "2026-09-07"),
    ]);

    let out = run_ok(&fx, &["neighborhood", "678"]);

    assert!(
        filas(&out).iter().any(|line| line.contains("679*")),
        "la fila del 679 debe llevar `*`:\n{out}"
    );
    assert!(
        out.contains("* número provisional (se fija al publicar)"),
        "falta la leyenda:\n{out}"
    );
}

#[test]
fn target_inexistente_falla() {
    let fx = fixture(&parrilla_674_682());

    let (_code, out, err) = run_fail(&fx, &["neighborhood", "99999"]);

    assert_eq!(err.trim(), "ERROR: Episodio '99999' no encontrado.");
    assert!(out.trim().is_empty(), "stdout no debe tener texto: {out:?}");
}

#[test]
fn target_sin_numero_falla() {
    let fx = fixture(&[sin_numero("recien-anadido", "2026-09-02", "scripted")]);

    let (_code, out, err) = run_fail(&fx, &["neighborhood", "recien-anadido"]);

    assert!(
        err.to_lowercase().contains("no tiene número"),
        "el error debe explicar que falta el número: {err:?}"
    );
    assert!(out.trim().is_empty(), "stdout no debe tener texto: {out:?}");
}

#[test]
fn no_escribe_nada() {
    let fx = fixture(&parrilla_674_682());
    let antes_texto = read_tracker(&fx);
    let antes_arbol = snapshot(&fx.root);
    let antes_mtime = fs::metadata(&fx.tracker)
        .expect("metadata tracker")
        .modified()
        .expect("mtime tracker");

    let _ = run_ok(&fx, &["neighborhood", "678"]);

    assert_eq!(read_tracker(&fx), antes_texto, "tracker alterado");
    assert_eq!(snapshot(&fx.root), antes_arbol, "árbol alterado");
    assert_eq!(
        fs::metadata(&fx.tracker)
            .expect("metadata tracker")
            .modified()
            .expect("mtime tracker"),
        antes_mtime,
        "mtime del tracker alterado"
    );
    let bak = fs::read_dir(&fx.root)
        .expect("read_dir root")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .any(|name| name.ends_with(".bak"));
    assert!(!bak, "no debe crearse ningún .bak en {:?}", fx.root);
}

#[test]
fn after_cero_deja_la_ventana_a_un_lado() {
    let fx = fixture(&parrilla_674_682());

    let out = run_ok(&fx, &["neighborhood", "678", "--after", "0"]);

    assert_eq!(cabecera(&out), "  Vecindad de #678 (antes: 4, después: 0):");
    let filas = filas(&out);
    assert_eq!(filas.len(), 5);
    assert_eq!(numeros(&out), vec![674, 675, 676, 677, 678]);
    assert!(filas[4].starts_with("► "), "última fila: {:?}", filas[4]);
}

/// Tracker con el número `11` repetido tres veces (anomalía de datos que el
/// cargador no rechaza) y vecinos a ambos lados.
fn parrilla_con_duplicados() -> Vec<Ep> {
    vec![
        publicado(10, "2026-09-01"),
        publicado(11, "2026-09-02"),
        publicado(11, "2026-09-03"),
        publicado(11, "2026-09-04"),
        publicado(12, "2026-09-05"),
        publicado(13, "2026-09-06"),
        publicado(14, "2026-09-07"),
        publicado(15, "2026-09-08"),
    ]
}

#[test]
fn marcador_una_sola_vez_con_numeros_repetidos() {
    let fx = fixture(&parrilla_con_duplicados());

    let out = run_ok(&fx, &["neighborhood", "11"]);

    assert_eq!(
        out.matches('►').count(),
        1,
        "el marcador debe salir una sola vez:\n{out}"
    );
}

#[test]
fn duplicados_no_consumen_el_presupuesto_de_after() {
    let fx = fixture(&parrilla_con_duplicados());

    let out = run_ok(&fx, &["neighborhood", "11"]);

    assert_eq!(cabecera(&out), "  Vecindad de #11 (antes: 1, después: 4):");
    assert_eq!(numeros(&out), vec![10, 11, 12, 13, 14, 15]);
}

#[test]
fn la_tabla_y_el_json_muestran_los_mismos_episodios() {
    use serde_json::Value;

    let fx = fixture(&parrilla_con_duplicados());

    let texto = run_ok(&fx, &["neighborhood", "11"]);
    let json = run_ok(&fx, &["neighborhood", "11", "--json"]);
    let doc: Value = serde_json::from_str(&json).expect("JSON válido");

    let numeros_json = |clave: &str| -> Vec<u32> {
        doc[clave]
            .as_array()
            .unwrap_or_else(|| panic!("{clave} debe ser una lista"))
            .iter()
            .map(|ep| {
                u32::try_from(ep["number"].as_u64().expect("number entero"))
                    .expect("number cabe en u32")
            })
            .collect()
    };

    let mut del_json = numeros_json("before");
    del_json.push(
        u32::try_from(doc["episode"]["number"].as_u64().expect("number entero"))
            .expect("number cabe en u32"),
    );
    del_json.extend(numeros_json("after"));

    assert_eq!(
        del_json,
        numeros(&texto),
        "tabla y JSON deben coincidir:\n{texto}"
    );
    assert_eq!(del_json, vec![10, 11, 12, 13, 14, 15]);
}
