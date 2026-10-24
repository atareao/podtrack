//! Reconocimiento y reescritura de los hooks «Próximo episodio»
//! (`rebase-references`).
//!
//! Porta los patrones del oráculo (`_HOOK_TITLE_RE`, `_HOOK_BARE_RE`,
//! `_NUM_REF_RE`) y la clasificación de `_rebase_line`, más la escritura atómica
//! (`_write_atomic_text`). No se usan *lookbehinds* (ni el oráculo los usa).

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;

use crate::error::PodtrackError;

/// Etiqueta del hook estructural: «Próximo episodio» (admite la variante sin tilde).
pub const HOOK_LABEL: &str = r"Pr[oó]ximo\s+episodio";

/// Cuántas líneas de muestra por fichero en el informe de prosa.
pub const PROSE_SAMPLE: usize = 3;

/// Clasificación de una línea del material.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RebaseKind {
    /// Hook con título: se elimina el número y el separador.
    Rewritten,
    /// Hook con número pero sin título (no se toca).
    NoTitle,
}

/// Compila un patrón constante. Si fallara (imposible para estos literales),
/// devuelve `None` en vez de entrar en pánico.
fn compile(pattern: &str) -> Option<Regex> {
    Regex::new(pattern).ok()
}

/// Hook con título: `...Próximo episodio<...>: NNN <sep> <título>`.
static HOOK_TITLE_RE: LazyLock<Option<Regex>> = LazyLock::new(|| {
    compile(&format!(
        r"^(?P<head>.*?{HOOK_LABEL}\b[^\d\n]*)(?P<number>\d{{3,4}})\s*[—–:\-]\s*(?P<title>\S.*?)\s*$"
    ))
});

/// Hook sin título: `...Próximo episodio<...>: NNN` y nada más en la línea.
static HOOK_BARE_RE: LazyLock<Option<Regex>> = LazyLock::new(|| {
    compile(&format!(
        r"^(?P<head>.*?{HOOK_LABEL}\b[^\d\n]*)(?P<number>\d{{3,4}})\s*(?:[—–:\-]\s*)?$"
    ))
});

/// Números candidatos a referencia en prosa (3-4 dígitos).
static NUM_REF_RE: LazyLock<Option<Regex>> = LazyLock::new(|| compile(r"\b\d{3,4}\b"));

/// Clasifica y rebasa una línea (`_rebase_line`).
///
/// Devuelve la línea resultante y su clase: `Rewritten` (hook con título, se
/// eliminó número/separador), `NoTitle` (hook con número pero sin título), o
/// `None` (no es un hook estructural). Nunca reescribe la prosa.
#[must_use]
pub fn rebase_line(line: &str) -> (String, Option<RebaseKind>) {
    if let Some(re) = HOOK_TITLE_RE.as_ref()
        && let Some(caps) = re.captures(line)
    {
        let head = caps.name("head").map_or("", |m| m.as_str()).trim_end();
        let title = caps.name("title").map_or("", |m| m.as_str());
        return (format!("{head} {title}"), Some(RebaseKind::Rewritten));
    }
    if HOOK_BARE_RE.as_ref().is_some_and(|re| re.is_match(line)) {
        return (line.to_string(), Some(RebaseKind::NoTitle));
    }
    (line.to_string(), None)
}

/// Números de 3-4 dígitos presentes en una línea (`_NUM_REF_RE`).
#[must_use]
pub fn numeric_references(line: &str) -> Vec<u32> {
    let Some(re) = NUM_REF_RE.as_ref() else {
        return Vec::new();
    };
    re.find_iter(line)
        .filter_map(|m| m.as_str().parse::<u32>().ok())
        .collect()
}

/// Escribe `text` en `path` de forma atómica (fichero temporal + replace),
/// `_write_atomic_text`.
pub fn write_atomic_text(path: &Path, text: &str) -> Result<(), PodtrackError> {
    let mut name = path
        .file_name()
        .map_or_else(OsString::new, |n| n.to_os_string());
    name.push(".tmp");
    let tmp = path.with_file_name(name);
    fs::write(&tmp, text).map_err(|e| PodtrackError::Io(e.to_string()))?;
    fs::rename(&tmp, path).map_err(|e| PodtrackError::Io(e.to_string()))?;
    Ok(())
}

/// Ficheros `*.md` de `folder` (recursivo), ordenados por componentes de ruta
/// (equivalente a `sorted(folder.rglob("*.md"))` del oráculo).
#[must_use]
pub fn collect_md_files(folder: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_md(folder, &mut files);
    files.sort_by_key(|a| path_parts(a));
    files
}

fn collect_md(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_md(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            out.push(path);
        }
    }
}

fn path_parts(path: &Path) -> Vec<String> {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{RebaseKind, numeric_references, rebase_line};

    #[test]
    fn clasifica_hooks_y_prosa() {
        assert_eq!(
            rebase_line("## Próximo episodio: 844 — Encuadernar apuntes"),
            (
                "## Próximo episodio: Encuadernar apuntes".to_string(),
                Some(RebaseKind::Rewritten)
            )
        );
        assert_eq!(
            rebase_line("## Próximo episodio: 844"),
            (
                "## Próximo episodio: 844".to_string(),
                Some(RebaseKind::NoTitle)
            )
        );
        assert_eq!(
            rebase_line("En el episodio 844 hablamos."),
            ("En el episodio 844 hablamos.".to_string(), None)
        );
    }

    #[test]
    fn prosa_de_tres_a_cuatro_digitos() {
        assert_eq!(numeric_references("El 502 y 866"), vec![502, 866]);
        assert!(numeric_references("12 y 12345").is_empty());
    }
}
