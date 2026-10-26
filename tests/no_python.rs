//! Guardián mecánico: la suite no puede reintroducir el arnés retirado.
//!
//! `close-the-port` retiró el oráculo Python y los modos temporales de grabación y auditoría. Este
//! test es el candado que impide que vuelvan por la puerta de atrás: recorre `tests/` y falla si
//! reaparece alguna de sus referencias, si aparece un fichero o directorio terminado en la
//! extensión del oráculo, o si el golden deja de tener sus rutas temporales normalizadas.
//!
//! **Qué se busca y dónde.** Las agujas viven en dos familias, con ámbitos distintos a propósito:
//!
//! - **Referencias inequívocas** —el importador dinámico del intérprete y cuatro variables de
//!   entorno del oráculo y de los dos modos retirados— se buscan en el **texto de todo fichero
//!   bajo `tests/`**, con cualquier nombre y cualquier extensión (`.sh`, `.md`, `.toml`, `.json`,
//!   sin extensión…), recorriendo recursivamente. Ninguna de esas cadenas puede aparecer
//!   legítimamente en un dato de prueba, así que el ámbito ancho no arriesga falsos positivos. La
//!   única excepción es `tests/golden/**`: un `stderr` histórico podría citar el oráculo, y ahí
//!   solo se comprueba la higiene de rutas temporales (más abajo).
//! - **El intérprete** —el nombre sin versión, en minúsculas y en mayúsculas— se busca en todo
//!   fichero bajo `tests/` **menos `tests/golden/**` y `tests/fixtures/**`**. La aguja es el
//!   nombre **sin versión**, de modo que toda forma —con número, con sufijo, en una ruta de
//!   entorno— queda cubierta sin enumerarlas una a una.
//!
//!   La asimetría de `fixtures/` es **decisión, no descuido**. Hoy bajo `tests/fixtures/` solo hay
//!   **datos** —transcripciones `.srt` y perfiles `.yaml`—, **sin un solo ejecutable** y **sin una
//!   sola coincidencia** de la aguja. La exclusión es **previsión declarada**: una transcripción
//!   futura puede citar el intérprete en prosa y en minúsculas y no queremos que eso rompa el CI.
//!   Su residuo —un script plantado a mano en `fixtures/` que invocase el intérprete— queda
//!   anotado como **límite conocido** de este guardián. La diferencia con la **familia A** es
//!   deliberada: las referencias inequívocas del oráculo no pueden aparecer en una transcripción,
//!   la palabra del intérprete sí, así que solo esta última se excluye de `fixtures/`.
//! - **El nombre del oráculo** —cualquier fichero o directorio cuyo nombre termine en `.py`— se
//!   comprueba en **todo lo que haya bajo `tests/`, ficheros y directorios**, sin distinguir
//!   mayúsculas. Se mira el **nombre completo**, no la extensión: un fichero llamado exactamente
//!   `.py` (sin tallo) no tiene extensión para `Path::extension()` y se colaría.
//!
//! **Los errores de lectura son infracciones, nunca silencio.** Un árbol con permisos raros haría
//! pasar el guardián en vacío —justo el patrón que este change existe para eliminar—, así que un
//! `Err` al leer un fichero o al listar un directorio se reporta como infracción y hace fallar el
//! test. Un guardián que no puede leer es un guardián que ha fallado.
//!
//! **Por qué las agujas se construyen por fragmentos con `concat!`.** Este fichero vive bajo
//! `tests/`, así que la comprobación de texto se escanea a sí misma. Una aguja escrita como un
//! literal contiguo haría que el guardián se tropezara consigo mismo y fallara siempre. Por eso
//! cada aguja se parte en trozos que solo existen unidos en tiempo de compilación: el resultado es
//! idéntico en ejecución y no queda ninguna coincidencia literal en el código fuente. **No
//! «arregles» esto a literales**: romperías el guardián. Lo mismo vale para la prosa de estos
//! comentarios, que describe las agujas sin escribirlas.

use std::path::{Path, PathBuf};

/// Agujas de la **familia A**: referencias inequívocas, buscadas en el texto de todo fichero bajo
/// `tests/` salvo `tests/golden/**`.
///
/// Son el importador dinámico del intérprete y cuatro variables de entorno del oráculo y de los
/// dos modos retirados. La variable del oráculo cubre además su variante con sufijo (`_SRT`), por
/// ser prefijo suyo.
const REFERENCIAS_INEQUIVOCAS: [&str; 5] = [
    concat!("import", "lib"),
    concat!("PODTRACK_", "ORACLE"),
    concat!("PODTRACK_", "REQUIRE_ORACLE"),
    concat!("PODTRACK_", "RECORD_", "GOLDEN"),
    concat!("PODTRACK_", "AUDIT_", "GOLDEN"),
];

/// Agujas de la **familia B**: el nombre del intérprete **sin versión**, en minúsculas y en
/// mayúsculas. Se buscan en todo fichero bajo `tests/` menos `tests/golden/**` y
/// `tests/fixtures/**` (ver la cabecera: los fixtures son datos).
const AGUJAS_INTERPRETE: [&str; 2] = [concat!("pyt", "hon"), concat!("PYT", "HON")];

/// Raíz del crate, anclada al manifest y no al directorio de trabajo.
fn raiz_del_crate() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Una entrada del árbol de `tests/`: su ruta y si es directorio (para no intentar leerla como
/// texto).
struct Entrada {
    ruta: PathBuf,
    es_dir: bool,
}

/// Recorre `root` en profundidad y devuelve, en **orden determinista**, sus entradas (ficheros y
/// directorios) junto con las **infracciones de legibilidad** encontradas al listar.
///
/// Un directorio que no se puede listar no se salta en silencio: produce una infracción. Las
/// entradas se ordenan por ruta para que el mensaje sea estable entre ejecuciones y entre
/// sistemas de ficheros.
fn recorrer(root: &Path, crate_root: &Path) -> (Vec<Entrada>, Vec<String>) {
    let mut entradas = Vec::new();
    let mut infracciones = Vec::new();
    let mut pila = vec![root.to_path_buf()];
    while let Some(dir) = pila.pop() {
        let listado = match std::fs::read_dir(&dir) {
            Ok(listado) => listado,
            Err(e) => {
                infracciones.push(format!("{}: ilegible ({e})", relativa(&dir, crate_root)));
                continue;
            }
        };
        for entrada in listado {
            let entrada = match entrada {
                Ok(entrada) => entrada,
                Err(e) => {
                    infracciones.push(format!("{}: ilegible ({e})", relativa(&dir, crate_root)));
                    continue;
                }
            };
            let ruta = entrada.path();
            let es_dir = ruta.is_dir();
            if es_dir {
                pila.push(ruta.clone());
            }
            entradas.push(Entrada { ruta, es_dir });
        }
    }
    entradas.sort_by(|a, b| a.ruta.cmp(&b.ruta));
    (entradas, infracciones)
}

/// Ruta relativa a `root`, legible en el mensaje.
///
/// El mensaje se ancla en la **raíz del crate**, de modo que cada infracción se nombra como se ve
/// desde el repositorio (`tests/...`), igual que un `grep -rn`.
fn relativa(ruta: &Path, root: &Path) -> String {
    ruta.strip_prefix(root)
        .unwrap_or(ruta)
        .to_string_lossy()
        .into_owned()
}

/// Texto de un fichero, tolerando bytes que no sean UTF-8.
///
/// `Err` devuelve ya la **infracción** formateada: un error de lectura jamás se convierte en texto
/// vacío, porque leer en falso abriría el agujero que este guardián cierra.
fn leer(ruta: &Path, crate_root: &Path) -> Result<String, String> {
    std::fs::read(ruta)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .map_err(|e| format!("{}: ilegible ({e})", relativa(ruta, crate_root)))
}

/// ¿El **nombre completo** termina en `.py`, sin distinguir mayúsculas? Se aplica a ficheros
/// **y** directorios.
///
/// Se comprueba el nombre, no `Path::extension()`: un fichero llamado exactamente `.py` (sin
/// tallo) no tiene extensión y escaparía.
fn es_del_oraculo(ruta: &Path) -> bool {
    ruta.file_name()
        .and_then(|nombre| nombre.to_str())
        .is_some_and(|nombre| {
            nombre
                .len()
                .checked_sub(3)
                .and_then(|inicio| nombre.get(inicio..))
                .is_some_and(|sufijo| sufijo.eq_ignore_ascii_case(".py"))
        })
}

/// El guardián: recoge **todas** las infracciones y hace **un solo** `assert!` al final, para que
/// un árbol con varios problemas se vea de una pasada en vez de obligar a N iteraciones.
#[test]
fn la_suite_no_reintroduce_el_arnes_retirado() {
    let crate_root = raiz_del_crate();
    let tests = crate_root.join("tests");
    let golden = tests.join("golden");
    let fixtures = tests.join("fixtures");

    let (entradas, mut infracciones) = recorrer(&tests, &crate_root);

    for entrada in &entradas {
        let rel = relativa(&entrada.ruta, &crate_root);
        let en_golden = entrada.ruta.starts_with(&golden);
        let en_fixtures = entrada.ruta.starts_with(&fixtures);

        // (a) el oráculo como nombre: se comprueba el nombre, no el contenido, y tanto en
        // ficheros como en directorios. La referencia encontrada es la propia ruta prohibida.
        if es_del_oraculo(&entrada.ruta) {
            infracciones.push(format!("{rel}: {rel}"));
        }

        // Los directorios solo se recorren; no se leen como texto.
        if entrada.es_dir {
            continue;
        }

        let contenido = match leer(&entrada.ruta, &crate_root) {
            Ok(contenido) => contenido,
            Err(infraccion) => {
                infracciones.push(infraccion);
                continue;
            }
        };

        // (b) referencias inequívocas: todo fichero salvo el golden.
        if !en_golden {
            for referencia in REFERENCIAS_INEQUIVOCAS {
                if contenido.contains(referencia) {
                    infracciones.push(format!("{rel}: {referencia}"));
                }
            }
        }

        // (c) el intérprete: todo fichero salvo el golden y los fixtures (ver la cabecera).
        if !en_golden && !en_fixtures {
            for aguja in AGUJAS_INTERPRETE {
                if contenido.contains(aguja) {
                    infracciones.push(format!("{rel}: {aguja}"));
                }
            }
        }

        // (d) higiene del golden: ninguna ruta temporal sin normalizar.
        if en_golden && contenido.contains(concat!("/tmp/", ".tm", "p")) {
            infracciones.push(format!("{rel}: ruta temporal sin normalizar"));
        }
    }

    infracciones.sort();

    assert!(
        infracciones.is_empty(),
        "el arnés retirado ha reaparecido bajo tests/ ({} infracción/es):\n{}",
        infracciones.len(),
        infracciones.join("\n")
    );
}
