//! Contrato del comparador del golden: el orden dentro de una tirada del plan de renombrado no
//! es contrato (lo decide el sistema de ficheros), pero su contenido y su agrupación por
//! profundidad sí.
//!
//! `collect_descendants` (`src/material.rs`) recorre el material con `read_dir` y ordena
//! **estable** por profundidad: los hermanos de una misma profundidad quedan en el orden que
//! devuelva el sistema de ficheros. El golden los congeló tal como los emitió el oráculo, así que
//! compararlos en orden exacto hacía la suite intermitente (el mismo árbol dio dos resultados:
//! run `40655350954` verde en `development` y run `40655346111` rojo en `main`).
//!
//! Estos tests fijan las dos caras del contrato: permutar hermanos **no** falla, y cualquier
//! cambio de contenido —una línea de más, de menos, alterada o movida de grupo— **sí**.
//!
//! El material sintético es un golden real: se reconstruye su propia salida (`lines` es la inversa
//! de `join("\n")`), se muta el caso **actual** y se compara contra el golden **prístino**. Nada de
//! esto escribe en `tests/golden/`.

mod common;

use std::path::Path;

use common::golden::{self, GoldenCase};

/// Raíz sintética: el golden no guarda rutas temporales (usa el marcador `<FIXTURE>`), así que
/// `normalize` no sustituye nada y cualquier ruta sirve.
const RAIZ: &str = "/tmp/podtrack-golden-listing-order";

/// Tira del plan de renombrado: `      · <origen>  →  <destino>`.
///
/// Predicado **explícito**: reformula aquí el formato en lugar de reutilizar el del comparador, para
/// que el test se lea solo. No es una implementación independiente (comparte el formato
/// documentado), así que la cobertura real la dan las mutaciones de contenido y
/// `permutar_lo_que_no_es_el_plan_falla`, no la duplicación del parseo.
fn plan(linea: &str) -> Option<&str> {
    let resto = linea.strip_prefix("      · ")?;
    let (origen, destino) = resto.split_once("  →  ")?;
    if origen.is_empty() || destino.is_empty() {
        return None;
    }
    Some(origen)
}

/// Profundidad del origen: un componente de ruta por cada `/`.
fn profundidad(origen: &str) -> usize {
    origen.matches('/').count()
}

/// Índices de las líneas del plan, agrupados por tirada máxima.
fn tiradas(lineas: &[String]) -> Vec<Vec<usize>> {
    let mut out: Vec<Vec<usize>> = Vec::new();
    let mut actual: Vec<usize> = Vec::new();
    for (i, linea) in lineas.iter().enumerate() {
        if plan(linea).is_some() {
            actual.push(i);
        } else if !actual.is_empty() {
            out.push(std::mem::take(&mut actual));
        }
    }
    if !actual.is_empty() {
        out.push(actual);
    }
    out
}

/// Reconstruye la salida de un caso desde su propio golden.
fn reproducir(case: &GoldenCase) -> (String, String) {
    (case.stdout.join("\n"), case.stderr.join("\n"))
}

/// Compara la salida **mutada** (`actual`) contra el golden **prístino**.
///
/// Con esto, una mutación que rompa el contrato del comparador hace fallar la comparación; una que
/// solo permute hermanos de una misma profundidad, no.
///
/// El `code` se toma de `actual`, que es un clon de `pristine`: el arnés solo ejerce `stdout` y
/// `stderr`, y la comprobación del código de salida es vacua a propósito.
fn comprobar(pristine: &GoldenCase, actual: &GoldenCase) {
    let (stdout, stderr) = reproducir(actual);
    pristine.assert_outcome(Path::new(RAIZ), actual.code, &stdout, &stderr);
}

/// Aplica `f` a la salida del caso **actual** y exige que algo cambie (si no, el test no probaría
/// nada).
fn mutar(actual: &mut GoldenCase, f: impl FnOnce(&mut Vec<String>)) {
    let antes = actual.stdout.clone();
    f(&mut actual.stdout);
    assert_ne!(antes, actual.stdout, "la mutación no cambió la salida");
}

/// Permuta dos líneas distintas dentro de `candidatos`.
fn permutar_hermanos(lineas: &mut [String], candidatos: &[usize]) {
    for (n, &i) in candidatos.iter().enumerate() {
        if let Some(&j) = candidatos[n + 1..]
            .iter()
            .find(|&&j| lineas[j] != lineas[i])
        {
            lineas.swap(i, j);
            return;
        }
    }
}

/// El caso de `rename-material` con dos episodios y tiradas de dos profundidades.
fn caso_rename() -> GoldenCase {
    golden::case("parity_surgery/oraculo_rename_material", 0)
}

#[test]
fn permutar_hermanos_de_una_tirada_no_falla() {
    let pristine = caso_rename();
    let mut actual = pristine.clone();
    mutar(&mut actual, |lineas| {
        for tirada in tiradas(lineas) {
            let mut inicio = 0;
            while inicio < tirada.len() {
                let Some(origen) = plan(&lineas[tirada[inicio]]) else {
                    break;
                };
                let depth = profundidad(origen);
                let mut fin = inicio + 1;
                while fin < tirada.len()
                    && plan(&lineas[tirada[fin]]).is_some_and(|o| profundidad(o) == depth)
                {
                    fin += 1;
                }
                permutar_hermanos(lineas, &tirada[inicio..fin]);
                inicio = fin;
            }
        }
    });
    comprobar(&pristine, &actual);
}

#[test]
#[should_panic(expected = "stdout difiere")]
fn duplicar_una_linea_del_plan_falla() {
    let pristine = caso_rename();
    let mut actual = pristine.clone();
    mutar(&mut actual, |lineas| {
        let i = tiradas(lineas)[0][0];
        let copia = lineas[i].clone();
        lineas.insert(i, copia);
    });
    comprobar(&pristine, &actual);
}

#[test]
#[should_panic(expected = "stdout difiere")]
fn quitar_una_linea_del_plan_falla() {
    let pristine = caso_rename();
    let mut actual = pristine.clone();
    mutar(&mut actual, |lineas| {
        let i = tiradas(lineas)[0][0];
        lineas.remove(i);
    });
    comprobar(&pristine, &actual);
}

#[test]
#[should_panic(expected = "stdout difiere")]
fn alterar_una_linea_del_plan_falla() {
    let pristine = caso_rename();
    let mut actual = pristine.clone();
    mutar(&mut actual, |lineas| {
        let i = tiradas(lineas)[0][0];
        let alterada = lineas[i].replacen("  →  ", "  →  extra-", 1);
        lineas[i] = alterada;
    });
    comprobar(&pristine, &actual);
}

#[test]
#[should_panic(expected = "stdout difiere")]
fn cambiar_una_linea_de_grupo_falla() {
    let pristine = caso_rename();
    let mut actual = pristine.clone();
    mutar(&mut actual, |lineas| {
        let tirada = tiradas(lineas).remove(0);
        let primero = tirada[0];
        let depth = plan(&lineas[primero]).map(profundidad).unwrap_or_default();
        let otro = tirada
            .iter()
            .copied()
            .find(|&i| plan(&lineas[i]).map(profundidad) != Some(depth))
            .expect("la tirada debe tener dos grupos de profundidad");
        lineas.swap(primero, otro);
    });
    comprobar(&pristine, &actual);
}

#[test]
#[should_panic(expected = "stdout difiere")]
fn permutar_lo_que_no_es_el_plan_falla() {
    let pristine = golden::case("rebase_references/oraculo_rebase_references", 0);
    let mut actual = pristine.clone();
    mutar(&mut actual, |lineas| {
        let candidatos: Vec<usize> = lineas
            .iter()
            .enumerate()
            .filter(|(_, l)| plan(l).is_none())
            .map(|(i, _)| i)
            .collect();
        permutar_hermanos(lineas, &candidatos);
    });
    comprobar(&pristine, &actual);
}
