//! Golden congelado de la paridad con el oráculo retirado.
//!
//! Este módulo **solo lee** el golden. Los ficheros `tests/golden/<área>/<caso>.json` se
//! grabaron **una sola vez** con el oráculo recuperado del histórico de `~/.agents` (commit
//! `c70287f^`), que ya no existe y es el **origen** del golden, no el contrato vivo. La
//! paridad **no es en vivo** y el golden **no se regenera solo**: es evidencia histórica y
//! candado contra regresiones, nunca se generó desde el binario y el crate **no puede
//! reescribirlo** (los tipos no implementan `Serialize`). Procedencia y forma:
//! `openspec/changes/close-the-port/design.md` (D1–D4).
//!
//! Los campos de texto son **arrays de líneas**: una cadena JSON escaparía `\n` y dejaría cada
//! caso en una sola línea, ilegible en un `git diff`.
//!
//! **Rutas temporales normalizadas**: el `TempDir` de la fixture es distinto en cada ejecución,
//! así que congelar su ruta haría el golden irreproducible. El golden guarda el marcador
//! `<FIXTURE>` en lugar de la raíz temporal de la fixture en `stdout`/`stderr` y también en el
//! `label` (que reproduce el `argv` y por eso puede llevar la ruta del SRT). `tracker`,
//! `material` y `yaml` **no** necesitan normalización: nunca contienen rutas.
//!
//! **Excepción acotada — el orden del plan de renombrado**: dentro de una tirada de líneas
//! `      · <origen>  →  <destino>`, los hermanos de una misma profundidad se comparan como
//! multiconjunto. Su orden lo decide `read_dir` (`src/material.rs`), no está especificado y nunca
//! fue contrato del oráculo: compararlo en orden exacto hacía la suite intermitente. Todo lo demás
//! —el contenido y el número de líneas del plan, la posición de cada tirada, el orden entre grupos
//! de profundidad y cualquier línea que no sea del plan— sigue comparándose exacto y en orden.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Fichero del golden: el agregado de escenarios de un mismo test.
#[derive(Debug, Clone, Deserialize)]
pub struct GoldenFile {
    /// Los escenarios, en el orden en que el test los compara.
    pub cases: Vec<GoldenCase>,
}

/// Un escenario congelado.
#[derive(Debug, Clone, Deserialize)]
pub struct GoldenCase {
    /// Identificación legible del escenario (el `argv` unido por espacios).
    pub label: String,
    /// Código de salida.
    pub code: i32,
    /// `stdout` como líneas.
    pub stdout: Vec<String>,
    /// `stderr` como líneas.
    pub stderr: Vec<String>,
    /// Líneas del fichero del tracker resultante, si el test lo compara.
    #[serde(default)]
    pub tracker: Option<Vec<String>>,
    /// Ruta relativa → líneas del material resultante, si el test lo compara.
    #[serde(default)]
    pub material: Option<BTreeMap<String, Vec<String>>>,
    /// Líneas del YAML canónico (writer), si el test lo compara byte a byte.
    #[serde(default)]
    pub yaml: Option<Vec<String>>,
}

/// Profundidad del origen de una línea del plan de renombrado, o `None` si la línea no es del plan.
///
/// El plan tiene la forma `      · <origen>  →  <destino>`. Cualquier otra línea que empiece por
/// `      · ` NO es del plan y NO SHALL canonizarse: los registros de `rebase-references`
/// (`      · <ruta>:<línea>`, sin flecha) van ordenados a propósito y su orden sí es contrato, y
/// los avisos de `migrate-material` (`      · <problema>`, en `stderr`) tampoco llevan flecha.
fn plan_depth(line: &str) -> Option<usize> {
    let rest = line.strip_prefix("      · ")?;
    let (from, to) = rest.split_once("  →  ")?;
    if from.is_empty() || to.is_empty() {
        return None;
    }
    Some(from.matches('/').count())
}

/// Ordena cada grupo de igual profundidad de una tirada y conserva el orden de los grupos.
fn canonical_run(tirada: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(tirada.len());
    let mut inicio = 0;
    while inicio < tirada.len() {
        let depth = plan_depth(&tirada[inicio]);
        let mut fin = inicio + 1;
        while fin < tirada.len() && plan_depth(&tirada[fin]) == depth {
            fin += 1;
        }
        let mut grupo: Vec<&String> = tirada[inicio..fin].iter().collect();
        grupo.sort_unstable();
        out.extend(grupo.into_iter().cloned());
        inicio = fin;
    }
    out
}

/// Canoniza el orden de las tiradas del plan de renombrado de una salida.
///
/// `collect_descendants` (`src/material.rs`) recorre el material con `read_dir` y ordena **estable**
/// solo por profundidad: los hermanos de una misma profundidad quedan en el orden que devuelva el
/// sistema de ficheros, que no está especificado y por tanto **no es contrato**. Aquí se agrupa cada
/// tirada máxima por profundidad y se ordena cada grupo; la posición de la tirada y el orden
/// **entre** grupos (de más profundo a menos) SHALL permanecer. El número y el contenido de las
/// líneas NO SHALL relajarse: una línea de más, de menos o alterada sigue fallando.
fn canonical_plan_runs(lineas: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(lineas.len());
    let mut tirada: Vec<String> = Vec::new();
    for linea in lineas {
        if plan_depth(&linea).is_some() {
            tirada.push(linea);
        } else {
            out.extend(canonical_run(std::mem::take(&mut tirada)));
            out.push(linea);
        }
    }
    out.extend(canonical_run(tirada));
    out
}

impl GoldenCase {
    /// Compara código de salida, `stdout` y `stderr`, normalizando las rutas temporales.
    ///
    /// `stdout` y `stderr` se comparan exactos y en orden, con la única excepción de las tiradas del
    /// plan de renombrado: dentro de una tirada, los hermanos de una misma profundidad se comparan
    /// como multiconjunto, porque su orden lo decide el sistema de ficheros y no es contrato.
    pub fn assert_outcome(&self, dir: &Path, code: i32, stdout: &str, stderr: &str) {
        assert_eq!(self.code, code, "exit code difiere [{}]", self.label);
        assert_eq!(
            canonical_plan_runs(self.stdout.clone()),
            canonical_plan_runs(lines(&normalize(stdout, dir))),
            "stdout difiere [{}]",
            self.label
        );
        assert_eq!(
            canonical_plan_runs(self.stderr.clone()),
            canonical_plan_runs(lines(&normalize(stderr, dir))),
            "stderr difiere [{}]",
            self.label
        );
    }

    /// Compara el fichero del tracker resultante.
    pub fn assert_tracker(&self, bytes: &[u8]) {
        let expected = self
            .tracker
            .as_ref()
            .unwrap_or_else(|| panic!("el golden de [{}] no guarda `tracker`", self.label));
        assert_eq!(
            expected,
            &lines(&utf8(bytes)),
            "tracker resultante difiere [{}]",
            self.label
        );
    }

    /// Compara el material resultante (rutas relativas → líneas).
    pub fn assert_material(&self, snapshot: &BTreeMap<String, Vec<u8>>) {
        let expected = self
            .material
            .as_ref()
            .unwrap_or_else(|| panic!("el golden de [{}] no guarda `material`", self.label));
        assert_eq!(
            expected,
            &lines_map(snapshot),
            "material resultante difiere [{}]",
            self.label
        );
    }

    /// Compara el árbol completo (el `material` del golden más el `tracker` en su ruta relativa).
    pub fn assert_tree(&self, tree: &BTreeMap<String, Vec<u8>>, root: &Path, tracker: &Path) {
        // A diferencia de `assert_tracker`/`assert_material`/`assert_yaml`, aquí un `material`
        // ausente NO es un error: hay escenarios sin material que comparar, así que el árbol
        // esperado se reduce al `tracker`. De ahí `unwrap_or_default()` en vez de `panic!`.
        let mut expected = self.material.clone().unwrap_or_default();
        if let Some(tracker_lines) = &self.tracker {
            expected.insert(rel_path(root, tracker), tracker_lines.clone());
        }
        assert_eq!(
            expected,
            lines_map(tree),
            "árbol resultante difiere [{}]",
            self.label
        );
    }

    /// Compara los bytes del YAML canónico.
    pub fn assert_yaml(&self, bytes: &[u8]) {
        let expected = self
            .yaml
            .as_ref()
            .unwrap_or_else(|| panic!("el golden de [{}] no guarda `yaml`", self.label));
        assert_eq!(
            expected,
            &lines(&utf8(bytes)),
            "YAML canónico difiere byte a byte [{}]",
            self.label
        );
    }
}

/// Texto → líneas, conservando la línea vacía final que produce el salto de cola.
pub fn lines(text: &str) -> Vec<String> {
    text.split('\n').map(str::to_owned).collect()
}

/// Sustituye la raíz temporal del fixture por el marcador `<FIXTURE>`.
///
/// El `TempDir` de la fixture cambia en cada ejecución, así que congelar su ruta haría el
/// golden irreproducible. Se normaliza **solo el prefijo correcto** (la raíz de la fixture):
/// cualquier otra ruta que aparezca en la salida se compara tal cual, de modo que un binario
/// que imprimiera una ruta equivocada sigue fallando.
pub fn normalize(text: &str, dir: &Path) -> String {
    text.replace(dir.to_string_lossy().as_ref(), "<FIXTURE>")
}

/// Bytes → texto, exigiendo UTF-8 válido (el golden guarda texto).
pub fn utf8(bytes: &[u8]) -> String {
    std::str::from_utf8(bytes)
        .expect("el golden solo guarda UTF-8")
        .to_owned()
}

/// Instantánea de ficheros (bytes) → mapa de líneas.
pub fn lines_map(snapshot: &BTreeMap<String, Vec<u8>>) -> BTreeMap<String, Vec<String>> {
    snapshot
        .iter()
        .map(|(path, bytes)| (path.clone(), lines(&utf8(bytes))))
        .collect()
}

/// Directorio del golden (`tests/golden`).
pub fn dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

/// Ruta del fichero de un caso (`<área>/<caso>`).
fn path_of(golden: &str) -> PathBuf {
    dir().join(format!("{golden}.json"))
}

/// Ruta relativa del tracker dentro de la raíz de la fixture.
fn rel_path(root: &Path, tracker: &Path) -> String {
    tracker
        .strip_prefix(root)
        .unwrap_or_else(|_| {
            panic!(
                "el tracker {} no está bajo {}",
                tracker.display(),
                root.display()
            )
        })
        .to_string_lossy()
        .to_string()
}

/// Los escenarios congelados del caso. Un golden ausente **falla** nombrando el fichero: así un
/// caso nuevo no pasa en vacío.
pub fn read(golden: &str) -> GoldenFile {
    let path = path_of(golden);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("golden ausente o ilegible ({}): {e}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("golden inválido ({}): {e}", path.display()))
}

/// El escenario `idx` del caso.
pub fn case(golden: &str, idx: usize) -> GoldenCase {
    let file = read(golden);
    file.cases.get(idx).cloned().unwrap_or_else(|| {
        panic!(
            "golden {golden} sin escenario {idx}: {} grabados",
            file.cases.len()
        )
    })
}
