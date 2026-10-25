//! `publish` — estampa el número público autoritativo.

use crate::commands::{Ctx, DRY_RUN_LINE, Rendered, bytes_to_string, resolve_index, save_quiet};
use crate::error::PodtrackError;
use crate::json;
use crate::model::Status;
use crate::tracker::{next_public_number, num_label, save_tracker};

/// Parámetros de `publish`.
#[derive(Debug, Clone)]
pub struct PublishParams {
    /// Número o slug del episodio.
    pub target: String,
    /// Previsualizar sin escribir.
    pub dry_run: bool,
}

/// `slug or title` del episodio, sin recortar: la base de las etiquetas del oráculo.
fn slug_or_title(episode: &crate::model::Episode) -> String {
    episode
        .slug
        .as_ref()
        .map_or_else(|| episode.title.clone(), |s| s.as_str().to_string())
}

/// `str(slug or title or "").strip()` (`cmd_publish`): etiqueta del alta y de la
/// anulación del provisional obsoleto de otro episodio.
fn label(episode: &crate::model::Episode) -> String {
    slug_or_title(episode).trim().to_string()
}

/// Ejecuta `publish`.
pub fn run(
    ctx: &Ctx<'_>,
    params: &PublishParams,
    json_mode: bool,
) -> Result<Rendered, PodtrackError> {
    let index = resolve_index(ctx.tracker, &params.target).ok_or_else(|| {
        PodtrackError::EpisodeNotFound {
            target: params.target.clone(),
        }
    })?;

    let current = &ctx.tracker.episodes[index];
    if current.status == Status::Published {
        return Err(PodtrackError::PublishAlreadyPublished {
            label: num_label(current.number),
            // El oráculo NO recorta en este mensaje (`ep.get('slug') or
            // ep.get('title')`), así que se usa la variante sin `strip` para no
            // divergir byte a byte.
            name: slug_or_title(current),
        });
    }

    let mut work = ctx.tracker.clone();
    let number = next_public_number(&work);
    let provisional = work.episodes[index].number;
    let etiqueta = label(&work.episodes[index]);

    // Un provisional obsoleto de **otro** episodio no publicado no puede quedar pegado.
    let mut nulled = Vec::new();
    for other_index in 0..work.episodes.len() {
        if other_index == index {
            continue;
        }
        let other = &work.episodes[other_index];
        if other.status != Status::Published && other.number == Some(number) {
            nulled.push(label(other));
            work.episodes[other_index].number = None;
        }
    }

    work.episodes[index].number = Some(number);
    work.episodes[index].status = Status::Published;
    let episode = work.episodes[index].clone();

    if json_mode {
        if !params.dry_run {
            save_quiet(&ctx.show.tracker, &ctx.show.name, &work)?;
        }
        return Ok(Rendered::stdout(json::publish_document(
            ctx,
            &episode,
            &nulled,
            params.dry_run,
        )));
    }

    let mut out = Vec::new();
    let mut err = Vec::new();
    if params.dry_run {
        out.extend_from_slice(format!("{DRY_RUN_LINE}\n").as_bytes());
    }
    out.extend_from_slice(format!("  ✓ Publicado {etiqueta} como #{number}\n").as_bytes());
    if let Some(provisional) = provisional
        && provisional != number
    {
        out.extend_from_slice(format!("    (provisional #{provisional} → #{number})\n").as_bytes());
    }
    for otro in &nulled {
        out.extend_from_slice(
            format!("    (el provisional #{number} de '{otro}' es obsoleto: se anula)\n")
                .as_bytes(),
        );
    }

    if !params.dry_run {
        save_tracker(&ctx.show.tracker, &ctx.show.name, &work, &mut out, &mut err)?;
    }

    Ok(Rendered::new(bytes_to_string(out), bytes_to_string(err)))
}
