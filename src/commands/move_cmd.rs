//! `move` — cambia la fecha de emisión de un episodio a un día publicable.

use chrono::Datelike;

use crate::commands::{Ctx, DRY_RUN_LINE, Rendered, bytes_to_string, resolve_index, save_quiet};
use crate::error::PodtrackError;
use crate::json;
use crate::model::{PUBLISH_DAYS, parse_iso};
use crate::tracker::{num_label, save_tracker};

/// Parámetros de `move`.
#[derive(Debug, Clone)]
pub struct MoveParams {
    /// Número o slug del episodio.
    pub target: String,
    /// Nueva fecha `YYYY-MM-DD` (debe ser lunes o jueves).
    pub date: String,
    /// Previsualizar sin escribir.
    pub dry_run: bool,
}

/// Ejecuta `move`.
pub fn run(ctx: &Ctx<'_>, params: &MoveParams, json_mode: bool) -> Result<Rendered, PodtrackError> {
    let parsed = parse_iso(&params.date)?;
    let new_date = parsed.format("%Y-%m-%d").to_string();
    if !PUBLISH_DAYS.contains(&parsed.weekday().num_days_from_monday()) {
        return Err(PodtrackError::MoveNotPublishDay { date: new_date });
    }

    let index = resolve_index(ctx.tracker, &params.target).ok_or_else(|| {
        PodtrackError::EpisodeNotFound {
            target: params.target.clone(),
        }
    })?;

    for (other_index, other) in ctx.tracker.episodes.iter().enumerate() {
        if other_index != index && other.date.to_iso() == new_date {
            return Err(PodtrackError::MoveCollision {
                label: num_label(other.number),
                date: new_date,
            });
        }
    }

    let old_date = ctx.tracker.episodes[index].date.to_iso();

    if json_mode {
        let mut work = ctx.tracker.clone();
        if !params.dry_run {
            work.episodes[index].date = crate::model::Date::parse(&new_date);
            save_quiet(&ctx.show.tracker, &ctx.show.name, &work)?;
        }
        let episode = work.episodes[index].clone();
        return Ok(Rendered::stdout(json::move_document(
            ctx,
            &episode,
            &old_date,
            &new_date,
            params.dry_run,
        )));
    }

    let mut out = Vec::new();
    let mut err = Vec::new();
    if params.dry_run {
        out.extend_from_slice(format!("{DRY_RUN_LINE}\n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "  ✓ Mover #{}: {old_date} → {new_date}\n",
            num_label(ctx.tracker.episodes[index].number)
        )
        .as_bytes(),
    );

    if !params.dry_run {
        let mut work = ctx.tracker.clone();
        work.episodes[index].date = crate::model::Date::parse(&new_date);
        save_tracker(&ctx.show.tracker, &ctx.show.name, &work, &mut out, &mut err)?;
    }

    Ok(Rendered {
        stdout: bytes_to_string(out),
        stderr: bytes_to_string(err),
    })
}
