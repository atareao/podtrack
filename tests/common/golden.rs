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

impl GoldenCase {
    /// Compara código de salida, `stdout` y `stderr`, normalizando las rutas temporales.
    pub fn assert_outcome(&self, dir: &Path, code: i32, stdout: &str, stderr: &str) {
        assert_eq!(self.code, code, "exit code difiere [{}]", self.label);
        assert_eq!(
            self.stdout,
            lines(&normalize(stdout, dir)),
            "stdout difiere [{}]",
            self.label
        );
        assert_eq!(
            self.stderr,
            lines(&normalize(stderr, dir)),
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
