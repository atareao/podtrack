//! `update` — cambio de estado de un episodio (por número o slug).

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
    /// Nuevo estado.
    pub status: String,
}

/// Ejecuta `update`.
pub fn run(
    ctx: &Ctx<'_>,
    params: &UpdateParams,
    json_mode: bool,
) -> Result<Rendered, PodtrackError> {
    let new_status = Status::parse(&params.status)?;

    let index = resolve_index(ctx.tracker, &params.target).ok_or_else(|| {
        PodtrackError::EpisodeNotFound {
            target: params.target.clone(),
        }
    })?;

    let mut work = ctx.tracker.clone();
    let old_status = work.episodes[index].status;
    work.episodes[index].status = new_status;

    if json_mode {
        let episode = work.episodes[index].clone();
        save_quiet(&ctx.show.tracker, &ctx.show.name, &work)?;
        return Ok(Rendered::stdout(json::update_document(
            ctx, &episode, old_status,
        )));
    }

    let mut out = Vec::new();
    let mut err = Vec::new();
    save_tracker(&ctx.show.tracker, &ctx.show.name, &work, &mut out, &mut err)?;

    let episode = &work.episodes[index];
    out.extend_from_slice(
        format!(
            "  ✓ Episodio #{}: \"{}\"\n    {} {} → {} {}\n",
            num_label(episode.number),
            episode.title,
            old_status.icon(),
            old_status,
            new_status.icon(),
            new_status
        )
        .as_bytes(),
    );

    Ok(Rendered {
        stdout: bytes_to_string(out),
        stderr: bytes_to_string(err),
    })
}
