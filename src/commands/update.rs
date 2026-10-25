//! `update` — cambio de estado y/o de título de un episodio (por número o slug).

use crate::commands::{Ctx, Rendered, bytes_to_string, resolve_index, save_quiet};
use crate::error::PodtrackError;
use crate::json;
use crate::model::Status;
use crate::tracker::{num_label, save_tracker};

/// Parámetros de `update`.
#[derive(Debug, Clone)]
pub struct UpdateParams {
    /// Número o slug del episodio.
    pub target: String,
    /// Nuevo estado (opcional).
    pub status: Option<String>,
    /// Nuevo título (opcional).
    pub title: Option<String>,
}

/// Ejecuta `update`.
pub fn run(
    ctx: &Ctx<'_>,
    params: &UpdateParams,
    json_mode: bool,
) -> Result<Rendered, PodtrackError> {
    // Validación previa: nada se escribe si algo falla.
    let new_status = params.status.as_deref().map(Status::parse).transpose()?;
    let new_title = match params.title.as_deref() {
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Err(PodtrackError::EmptyTitle);
            }
            Some(trimmed.to_string())
        }
        None => None,
    };

    let index = resolve_index(ctx.tracker, &params.target).ok_or_else(|| {
        PodtrackError::EpisodeNotFound {
            target: params.target.clone(),
        }
    })?;

    let mut work = ctx.tracker.clone();
    let old_status = work.episodes[index].status;
    let old_title = work.episodes[index].title.clone();

    let mut changes: Vec<(&'static str, String, String)> = Vec::new();
    if let Some(status) = new_status {
        work.episodes[index].status = status;
        changes.push(("status", old_status.to_string(), status.to_string()));
    }
    if let Some(title) = &new_title {
        work.episodes[index].title = title.clone();
        changes.push(("title", old_title.clone(), title.clone()));
    }

    if json_mode {
        let episode = work.episodes[index].clone();
        save_quiet(&ctx.show.tracker, &ctx.show.name, &work)?;
        return Ok(Rendered::stdout(json::update_document(
            ctx, &episode, &changes,
        )));
    }

    let mut out = Vec::new();
    let mut err = Vec::new();
    save_tracker(&ctx.show.tracker, &ctx.show.name, &work, &mut out, &mut err)?;

    let episode = &work.episodes[index];
    out.extend_from_slice(
        format!(
            "  ✓ Episodio #{}: \"{}\"\n",
            num_label(episode.number),
            episode.title
        )
        .as_bytes(),
    );
    if let Some(status) = new_status {
        out.extend_from_slice(
            format!(
                "    {} {} → {} {}\n",
                old_status.icon(),
                old_status,
                status.icon(),
                status
            )
            .as_bytes(),
        );
    }
    if let Some(title) = &new_title {
        out.extend_from_slice(
            format!("    Título: \"{}\" → \"{}\"\n", old_title, title).as_bytes(),
        );
    }

    Ok(Rendered::new(bytes_to_string(out), bytes_to_string(err)))
}
