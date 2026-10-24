//! `where` — ruta del material de guion, sin crear nada.

use crate::commands::{Ctx, Rendered, resolve_episode};
use crate::error::PodtrackError;
use crate::json;
use crate::tracker::num_label;

/// Ejecuta `where`.
pub fn run(ctx: &Ctx<'_>, target: &str, json_mode: bool) -> Result<Rendered, PodtrackError> {
    let episode =
        resolve_episode(ctx.tracker, target).ok_or_else(|| PodtrackError::EpisodeNotFound {
            target: target.to_string(),
        })?;

    let slug = episode
        .slug
        .as_ref()
        .map(|s| s.as_str().trim().to_string())
        .unwrap_or_default();
    if slug.is_empty() {
        return Err(PodtrackError::EpisodeWithoutSlug {
            label: num_label(episode.number),
        });
    }

    let folder = ctx.show.scripts_root().join(&slug);
    if json_mode {
        return Ok(Rendered::stdout(json::where_document(
            ctx, episode, &folder,
        )));
    }

    let existence = if folder.exists() {
        "existe"
    } else {
        "no existe"
    };
    Ok(Rendered::stdout(format!(
        "{} {existence}\n",
        folder.display()
    )))
}
