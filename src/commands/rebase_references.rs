//! `rebase-references` — reescribe los hooks `Próximo episodio: NNN — <título>`
//! del material de los episodios no publicados.

use std::collections::HashSet;
use std::fs;

use serde_json::{Map, Value, json};

use crate::commands::{Ctx, DRY_RUN_LINE, Rendered};
use crate::error::PodtrackError;
use crate::json;
use crate::material::relative;
use crate::model::{Status, sort_episodes};
use crate::rebase::{
    PROSE_SAMPLE, RebaseKind, collect_md_files, numeric_references, rebase_line, write_atomic_text,
};

/// Parámetros de `rebase-references`.
#[derive(Debug, Clone)]
pub struct RebaseParams {
    /// Previsualizar sin escribir.
    pub dry_run: bool,
}

/// Una reescritura de hook: fichero relativo, línea, antes y después.
struct Rewritten {
    file: String,
    line: usize,
    before: String,
    after: String,
}

/// Un hook sin título: fichero relativo, línea y texto.
struct NoTitle {
    file: String,
    line: usize,
    text: String,
}

/// Ejecuta `rebase-references`.
pub fn run(
    ctx: &Ctx<'_>,
    params: &RebaseParams,
    json_mode: bool,
) -> Result<Rendered, PodtrackError> {
    let root = ctx.show.scripts_root();

    let unpublished: HashSet<u32> = ctx
        .tracker
        .episodes
        .iter()
        .filter(|e| e.status != Status::Published)
        .filter_map(|e| e.number)
        .collect();

    let mut rewritten: Vec<Rewritten> = Vec::new();
    let mut no_title: Vec<NoTitle> = Vec::new();
    // Prosa: fichero → (línea, contexto), en orden de inserción.
    let mut prose: Vec<(String, Vec<(usize, String)>)> = Vec::new();

    let mut episodes = ctx.tracker.episodes.clone();
    sort_episodes(&mut episodes);

    for episode in &episodes {
        if episode.status == Status::Published {
            continue;
        }
        let slug = episode
            .slug
            .as_ref()
            .map(|s| s.as_str().trim().to_string())
            .unwrap_or_default();
        if slug.is_empty() {
            continue;
        }
        let folder = root.join(&slug);
        if !folder.is_dir() {
            continue;
        }

        for md in collect_md_files(&folder) {
            let rel = relative(&root, &md);
            let original = fs::read_to_string(&md).map_err(|e| PodtrackError::Io(e.to_string()))?;
            let mut out_lines: Vec<String> = Vec::new();
            let mut changed = false;

            for (index, line) in original.split('\n').enumerate() {
                let lineno = index + 1;
                let (new_line, kind) = rebase_line(line);
                match kind {
                    Some(RebaseKind::Rewritten) => {
                        rewritten.push(Rewritten {
                            file: rel.clone(),
                            line: lineno,
                            before: line.to_string(),
                            after: new_line.clone(),
                        });
                        changed = true;
                        out_lines.push(new_line);
                        continue;
                    }
                    Some(RebaseKind::NoTitle) => {
                        no_title.push(NoTitle {
                            file: rel.clone(),
                            line: lineno,
                            text: line.to_string(),
                        });
                    }
                    None => {
                        if numeric_references(line)
                            .iter()
                            .any(|n| unpublished.contains(n))
                        {
                            push_prose(&mut prose, &rel, lineno, line.trim());
                        }
                    }
                }
                out_lines.push(line.to_string());
            }

            if changed && !params.dry_run {
                write_atomic_text(&md, &out_lines.join("\n"))?;
            }
        }
    }

    let code = if no_title.is_empty() { 0 } else { 1 };

    if json_mode {
        let rewritten_json: Vec<Value> = rewritten
            .iter()
            .map(|r| {
                json!({
                    "file": r.file,
                    "line": r.line,
                    "before": r.before.trim(),
                    "after": r.after.trim(),
                })
            })
            .collect();
        let no_title_json: Vec<Value> = no_title
            .iter()
            .map(|r| json!({ "file": r.file, "line": r.line, "text": r.text.trim() }))
            .collect();
        let mut prose_map = Map::new();
        let mut sorted = prose.clone();
        sorted.sort_by(|a, b| a.0.cmp(&b.0));
        for (rel, hits) in sorted {
            let entries: Vec<Value> = hits
                .iter()
                .map(|(line, context)| json!({ "line": line, "context": context }))
                .collect();
            prose_map.insert(rel, Value::Array(entries));
        }
        return Ok(Rendered::stdout(json::rebase_references_document(
            ctx,
            &rewritten_json,
            &no_title_json,
            Value::Object(prose_map),
            params.dry_run,
        ))
        .with_code(code));
    }

    let mut out = String::new();
    if params.dry_run {
        out.push_str(&format!("{DRY_RUN_LINE}\n"));
    }
    out.push_str("  ✓ Rebase de hooks estructurales (episodios no publicados):\n");
    if rewritten.is_empty() {
        out.push_str("      (sin hooks que reescribir)\n");
    } else {
        for r in &rewritten {
            out.push_str(&format!("      · {}:{}\n", r.file, r.line));
            out.push_str(&format!("          antes:  {}\n", r.before.trim()));
            out.push_str(&format!("          ahora:  {}\n", r.after.trim()));
        }
    }

    if !no_title.is_empty() {
        out.push_str(&format!(
            "  ⚠ Hooks sin título (revisar a mano): {}\n",
            no_title.len()
        ));
        for r in &no_title {
            out.push_str(&format!("      · {}:{}\n", r.file, r.line));
            out.push_str(&format!("          {}\n", r.text.trim()));
        }
    }

    let total_prose: usize = prose.iter().map(|(_, hits)| hits.len()).sum();
    if total_prose > 0 {
        let plural = if prose.len() != 1 { "s" } else { "" };
        out.push_str(&format!(
            "  ℹ Referencias en prosa (no se tocan): {total_prose} en {} fichero{plural}\n",
            prose.len()
        ));
        let mut sorted = prose.clone();
        sorted.sort_by(|a, b| a.0.cmp(&b.0));
        for (rel, hits) in sorted {
            out.push_str(&format!("      · {rel} — {} ref(s)\n", hits.len()));
            for (line, context) in hits.iter().take(PROSE_SAMPLE) {
                let sample: String = context.chars().take(80).collect();
                out.push_str(&format!("          l.{line}: {sample}\n"));
            }
        }
    } else {
        out.push_str("  ℹ Referencias en prosa (no se tocan): ninguna\n");
    }

    Ok(Rendered::stdout(out).with_code(code))
}

fn push_prose(prose: &mut Vec<(String, Vec<(usize, String)>)>, rel: &str, line: usize, text: &str) {
    if let Some(entry) = prose.iter_mut().find(|(name, _)| name == rel) {
        entry.1.push((line, text.to_string()));
    } else {
        prose.push((rel.to_string(), vec![(line, text.to_string())]));
    }
}
