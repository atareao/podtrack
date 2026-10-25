//! Renombrado del material de guion al cambiar el número de un episodio.
//!
//! Porta las utilidades del parche puente del oráculo: `_renamed_component`,
//! `_collect_material_renames` y `_apply_material_renames`. La fase 3
//! (`migration-surgery`) reutilizará esta maquinaria para `rename-material`.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::PodtrackError;

/// Reescribe los tokens de número de un nombre de fichero/carpeta
/// (`_renamed_component`).
///
/// Reconoce la forma rellena a cuatro dígitos (`0NNN`) y la forma con prefijo `e`
/// (`eNNN`), acotadas por límites alfanuméricos: no toca secuencias de dígitos más
/// largas ni identificadores que las contengan.
#[must_use]
pub fn renamed_component(name: &str, old: u32, new: u32) -> String {
    let mut result = name.to_string();
    for (old_token, new_token) in [
        (format!("{old:04}"), format!("{new:04}")),
        (format!("e{old}"), format!("e{new}")),
    ] {
        result = replace_token(&result, &old_token, &new_token);
    }
    result
}

/// Sustituye todas las apariciones de `token` acotadas por límites alfanuméricos:
/// precedidas de algo que **no** sea `[0-9A-Za-z]` y seguidas de algo que **no**
/// sea un dígito (equivalente a `(?<![0-9A-Za-z])token(?![0-9])`).
fn replace_token(haystack: &str, token: &str, replacement: &str) -> String {
    if token.is_empty() {
        return haystack.to_string();
    }
    let bytes = haystack.as_bytes();
    let token_bytes = token.as_bytes();
    let mut out = String::with_capacity(haystack.len());
    let mut index = 0;
    while index < bytes.len() {
        let matches = index + token_bytes.len() <= bytes.len()
            && &bytes[index..index + token_bytes.len()] == token_bytes;
        if matches {
            let previous_ok = index == 0 || !bytes[index - 1].is_ascii_alphanumeric();
            let next_index = index + token_bytes.len();
            let next_ok = next_index == bytes.len() || !bytes[next_index].is_ascii_digit();
            if previous_ok && next_ok {
                out.push_str(replacement);
                index = next_index;
                continue;
            }
        }
        // Avanza un carácter completo (los tokens son ASCII, nunca cortan un
        // carácter multibyte).
        match haystack[index..].chars().next() {
            Some(ch) => {
                out.push(ch);
                index += ch.len_utf8();
            }
            None => break,
        }
    }
    out
}

/// Planifica el renombrado del material de un episodio (`old` → `new`).
///
/// Devuelve pares (origen, destino) ordenados **de más profundo a menos**: primero
/// los ficheros y subcarpetas, y la carpeta superior `0NNN` al final, de modo que
/// se puedan aplicar en secuencia sin invalidar rutas intermedias.
#[must_use]
pub fn plan_number_renames(root: &Path, old: u32, new: u32) -> Vec<(PathBuf, PathBuf)> {
    let top = root.join(format!("{old:04}"));
    if !top.is_dir() {
        return Vec::new();
    }

    let mut descendants = Vec::new();
    collect_descendants(&top, &mut descendants);
    // `sorted(..., key=len(parts), reverse=True)`: más profundo primero. El orden
    // dentro de la misma profundidad es el del recorrido (estable).
    descendants.sort_by_key(|path| std::cmp::Reverse(path.components().count()));

    let mut pairs = Vec::new();
    for path in descendants {
        let name = file_name_of(&path);
        let new_name = renamed_component(&name, old, new);
        if new_name != name {
            let parent = path.parent().map_or_else(PathBuf::new, Path::to_path_buf);
            pairs.push((path, parent.join(new_name)));
        }
    }

    let top_name = file_name_of(&top);
    let new_top_name = renamed_component(&top_name, old, new);
    if new_top_name != top_name {
        pairs.push((top, root.join(new_top_name)));
    }
    pairs
}

/// Recorre el árbol replicando el orden de `Path.rglob("*")` de Python: primero
/// los hijos directos de `dir` (en orden de `readdir`) y después, en ese mismo
/// orden, se desciende en cada subcarpeta.
fn collect_descendants(dir: &Path, out: &mut Vec<PathBuf>) {
    let children = read_dir_order(dir);
    for child in &children {
        out.push(child.clone());
    }
    for child in &children {
        if child.is_dir() {
            collect_descendants(child, out);
        }
    }
}

fn read_dir_order(dir: &Path) -> Vec<PathBuf> {
    match fs::read_dir(dir) {
        Ok(entries) => entries.flatten().map(|entry| entry.path()).collect(),
        Err(_) => Vec::new(),
    }
}

fn file_name_of(path: &Path) -> String {
    path.file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().into_owned())
}

/// Aplica un plan de renombrado reportando cada par a `out`.
///
/// Aborta con error si un destino ya existe, informando de la ruta en conflicto.
pub fn apply_renames(
    pairs: &[(PathBuf, PathBuf)],
    root: &Path,
    out: &mut Vec<u8>,
) -> Result<(), PodtrackError> {
    for (src, dst) in pairs {
        if dst.exists() {
            return Err(PodtrackError::MaterialDestinationExists {
                path: relative_to(dst, root),
            });
        }
        fs::rename(src, dst).map_err(|e| PodtrackError::Io(e.to_string()))?;
        out.extend_from_slice(
            format!(
                "      · {}  →  {}\n",
                relative_to(src, root),
                relative_to(dst, root)
            )
            .as_bytes(),
        );
    }
    Ok(())
}

fn relative_to(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

/// Ruta de `path` relativa a `root`, como texto (para reportar renombrados).
#[must_use]
pub fn relative(root: &Path, path: &Path) -> String {
    relative_to(path, root)
}

// ── Renombrado por slug (fase 3: `migrate-material`) ────────────────

/// Reescribe los tokens de número de un nombre **al slug** del episodio
/// (`_slug_component`).
///
/// Reconoce `episodio-NNN`, `eNNN`, `0NNN` y `NNN` (sin ceros) y los normaliza a
/// `slug`; `episodio-NNN` conserva la palabra (`episodio-<slug>`). Las demás
/// variantes del número colapsan al mismo destino (la migración lo reporta como
/// colisión). Los tokens van acotados por límites alfanuméricos.
#[must_use]
pub fn slug_component(name: &str, number: u32, slug: &str) -> String {
    let mut result = name.to_string();
    for (old_token, new_token) in [
        (format!("episodio-{number}"), format!("episodio-{slug}")),
        (format!("e{number}"), slug.to_string()),
        (format!("{number:04}"), slug.to_string()),
        (number.to_string(), slug.to_string()),
    ] {
        result = replace_token(&result, &old_token, &new_token);
    }
    result
}

/// Planifica la migración del material `0NNN/` de un episodio a `<slug>/`
/// (`_collect_slug_renames`).
///
/// Pares (origen, destino) de más profundo a menos, con la carpeta del episodio al
/// final. El contenido sin número no aparece en el plan: viaja con la carpeta.
///
/// El destino de la **carpeta** es el que fija la **regla única** de resolución de
/// material (`<raíz>/<dir>`), que el llamante pasa ya resuelto (`dest`); los tokens
/// de **número** de los nombres internos se reescriben al `slug`.
#[must_use]
pub fn collect_slug_renames_into(
    src: &Path,
    number: u32,
    slug: &str,
    dest: &Path,
) -> Vec<(PathBuf, PathBuf)> {
    let mut descendants = Vec::new();
    collect_descendants(src, &mut descendants);
    descendants.sort_by_key(|path| std::cmp::Reverse(path.components().count()));

    let mut pairs = Vec::new();
    for path in descendants {
        let name = file_name_of(&path);
        let new_name = slug_component(&name, number, slug);
        if new_name != name {
            let parent = path.parent().map_or_else(PathBuf::new, Path::to_path_buf);
            pairs.push((path, parent.join(new_name)));
        }
    }
    pairs.push((src.to_path_buf(), dest.to_path_buf()));
    pairs
}

/// Planifica la migración del material `0NNN/` de un episodio a `<raíz>/<slug>`
/// (compatibilidad con la caracterización de `migration-surgery`).
///
/// Es la forma histórica de [`collect_slug_renames_into`] con el destino cableado
/// a `<root>/<slug>`: la usan los tests de `material_renumber`. La vía de
/// producción (`migrate-material`) pasa el destino **ya resuelto** por
/// `ResolvedShow::material_dir`, para que lea y cirugía coincidan.
#[must_use]
pub fn collect_slug_renames(
    root: &Path,
    src: &Path,
    number: u32,
    slug: &str,
) -> Vec<(PathBuf, PathBuf)> {
    collect_slug_renames_into(src, number, slug, &root.join(slug))
}

/// Problemas de destino del plan: destinos existentes o convergentes
/// (`_slug_collisions`).
#[must_use]
pub fn slug_collisions(pairs: &[(PathBuf, PathBuf)]) -> Vec<String> {
    let mut problems = Vec::new();
    // Mapa ordenado por inserción: destino → orígenes.
    let mut by_dest: Vec<(PathBuf, Vec<PathBuf>)> = Vec::new();
    for (src, dst) in pairs {
        if let Some(entry) = by_dest.iter_mut().find(|(dest, _)| dest == dst) {
            entry.1.push(src.clone());
        } else {
            by_dest.push((dst.clone(), vec![src.clone()]));
        }
        if dst.exists() && dst != src {
            problems.push(format!("el destino ya existe: {}", file_name_of(dst)));
        }
    }
    for (dst, sources) in &by_dest {
        if sources.len() > 1 {
            let mut names: Vec<String> = sources.iter().map(|s| file_name_of(s)).collect();
            names.sort();
            problems.push(format!(
                "varios orígenes colisionan en {}: {}",
                file_name_of(dst),
                names.join(", ")
            ));
        }
    }
    problems
}

/// Aplica un plan de migración por slug reportando cada renombrado
/// (`_apply_slug_renames`). Las colisiones se detectan **antes** con
/// [`slug_collisions`], así que aquí no se comprueba el destino.
pub fn apply_slug_renames(
    pairs: &[(PathBuf, PathBuf)],
    root: &Path,
    out: &mut Vec<u8>,
) -> Result<(), PodtrackError> {
    for (src, dst) in pairs {
        fs::rename(src, dst).map_err(|e| PodtrackError::Io(e.to_string()))?;
        out.extend_from_slice(
            format!(
                "      · {}  →  {}\n",
                relative_to(src, root),
                relative_to(dst, root)
            )
            .as_bytes(),
        );
    }
    Ok(())
}

/// Carpetas numéricas `\d{4}` del árbol de guiones cuyo número no está en el
/// tracker (`_rename_material_check`), ordenadas por nombre.
#[must_use]
pub fn orphan_numeric_dirs(
    root: &Path,
    tracker_numbers: &std::collections::HashSet<u32>,
) -> Vec<(String, u32)> {
    if !root.is_dir() {
        return Vec::new();
    }
    let mut names: Vec<String> = read_dir_order(root)
        .into_iter()
        .filter_map(|path| path.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    names.sort();
    names
        .into_iter()
        .filter_map(|name| {
            let number = exact_four_digits(&name)?;
            (!tracker_numbers.contains(&number)).then_some((name, number))
        })
        .collect()
}

fn exact_four_digits(name: &str) -> Option<u32> {
    if name.len() == 4 && name.bytes().all(|b| b.is_ascii_digit()) {
        name.parse::<u32>().ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{plan_number_renames, renamed_component};
    use std::fs;

    #[test]
    fn reescribe_tokens_y_respeta_secuencias_largas() {
        assert_eq!(renamed_component("0843", 843, 844), "0844");
        assert_eq!(
            renamed_component("e843-articulo.md", 843, 844),
            "e844-articulo.md"
        );
        assert_eq!(renamed_component("08430", 843, 844), "08430");
        assert_eq!(renamed_component("x0843", 843, 844), "x0843");
    }

    #[test]
    fn plan_ordenado_de_mas_profundo_a_menos() {
        let dir = tempfile::tempdir().expect("tmp");
        let root = dir.path();
        let top = root.join("0843");
        fs::create_dir_all(top.join("sub")).expect("mkdir");
        fs::write(top.join("guion-0843.md"), "x").expect("write");
        fs::write(top.join("sub/deep-0843.md"), "x").expect("write");

        let pairs = plan_number_renames(root, 843, 844);
        let last = pairs.last().expect("last");
        assert_eq!(last.0, top);
        assert_eq!(last.1, root.join("0844"));
        let deep = pairs
            .iter()
            .position(|(src, _)| src.ends_with("sub/deep-0843.md"))
            .expect("deep");
        assert!(deep < pairs.len() - 1);
    }
}
