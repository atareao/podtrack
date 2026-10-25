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

    // `where` no informa de una ruta que no se pueda construir sin `slug`, **sea
    // cual sea la plantilla**: sin slug es el mismo caso tratado que la línea base
    // (`EpisodeWithoutSlug`, exit 1), y así `where` y `get` no pueden discrepar.
    let folder = ctx
        .material(episode)?
        .ok_or_else(|| PodtrackError::EpisodeWithoutSlug {
            label: num_label(episode.number),
        })?;
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
