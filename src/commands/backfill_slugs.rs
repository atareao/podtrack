//! `backfill-slugs` — deriva y persiste el `slug` de los episodios sin él.

use serde_json::{Value, json};

use crate::commands::{Ctx, DRY_RUN_LINE, Rendered, bytes_to_string, save_quiet};
use crate::error::PodtrackError;
use crate::json;
use crate::model::{Slug, slugify};
use crate::tracker::{num_label, save_tracker};

/// Parámetros de `backfill-slugs`.
#[derive(Debug, Clone)]
pub struct BackfillParams {
    /// Previsualizar sin escribir.
    pub dry_run: bool,
}

/// Slug libre a partir de `base`, sufijando `-2`, `-3`, … si colisiona
/// (`_unique_slug`).
fn unique_slug(used: &[String], base: &str) -> String {
    if !used.iter().any(|u| u == base) {
        return base.to_string();
    }
    let mut suffix = 2u32;
    loop {
        let candidate = format!("{base}-{suffix}");
        if !used.iter().any(|u| u == &candidate) {
            return candidate;
        }
        suffix += 1;
    }
}

/// Ejecuta `backfill-slugs`.
pub fn run(
    ctx: &Ctx<'_>,
    params: &BackfillParams,
    json_mode: bool,
) -> Result<Rendered, PodtrackError> {
    let tracker = ctx.tracker;
    let total = tracker.episodes.len();
    let mut work = tracker.clone();

    let mut used: Vec<String> = work
        .episodes
        .iter()
        .filter_map(|e| e.slug.as_ref().map(|s| s.as_str().to_string()))
        .collect();

    let mut out = Vec::new();
    let mut err = Vec::new();
    let mut changes: Vec<Value> = Vec::new();
    let mut skipped: Vec<Value> = Vec::new();
    let mut assigned = 0usize;

    if params.dry_run {
        out.extend_from_slice(format!("{DRY_RUN_LINE}\n").as_bytes());
    }

    for index in 0..work.episodes.len() {
        if work.episodes[index].slug.is_some() {
            continue;
        }
        let base = slugify(&work.episodes[index].title);
        let label = num_label(work.episodes[index].number);
        if base.is_empty() {
            err.extend_from_slice(
                format!(
                    "  ⚠ Aviso: el episodio #{label} no produce un slug utilizable; se omite.\n"
                )
                .as_bytes(),
            );
            skipped.push(json!({ "number": work.episodes[index].number }));
            continue;
        }
        let slug = unique_slug(&used, &base);
        used.push(slug.clone());
        work.episodes[index].slug = Some(Slug::parse(&slug)?);
        assigned += 1;
        out.extend_from_slice(format!("  {label:>5} -> {slug}\n").as_bytes());
        changes.push(json!({ "number": work.episodes[index].number, "slug": slug }));
    }

    out.extend_from_slice(
        format!("  {assigned} slugs asignados de {total} episodios\n").as_bytes(),
    );

    if json_mode {
        if !params.dry_run && assigned > 0 {
            save_quiet(&ctx.show.tracker, &ctx.show.name, &work)?;
        }
        return Ok(Rendered::stdout(json::backfill_slugs_document(
            ctx,
            &changes,
            &skipped,
            assigned,
            total,
            params.dry_run,
        )));
    }

    if !params.dry_run && assigned > 0 {
        save_tracker(&ctx.show.tracker, &ctx.show.name, &work, &mut out, &mut err)?;
    }

    Ok(Rendered::new(bytes_to_string(out), bytes_to_string(err)))
}
