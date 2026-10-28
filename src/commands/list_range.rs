//! `list-range` — listado por rango numérico, con marca `*` de no publicados.

use crate::commands::{Ctx, Rendered, show_episodes};
use crate::error::PodtrackError;
use crate::json;
use crate::model::Episode;

/// Ejecuta `list-range`.
pub fn run(
    ctx: &Ctx<'_>,
    start: u64,
    end: u64,
    json_mode: bool,
) -> Result<Rendered, PodtrackError> {
    let mut episodes: Vec<&Episode> = ctx
        .tracker
        .episodes
        .iter()
        .filter(|e| {
            e.number
                .is_some_and(|n| start <= u64::from(n) && u64::from(n) <= end)
        })
        .collect();
    episodes.sort_by_key(|e| e.number);

    if json_mode {
        return Ok(Rendered::stdout(json::list_range_document(
            ctx, start, end, &episodes,
        )));
    }

    if episodes.is_empty() {
        return Ok(Rendered::stdout(format!(
            "  No hay episodios entre #{start} y #{end}.\n"
        )));
    }

    let mut out = String::new();
    let mut err = String::new();
    out.push_str(&format!(
        "\n  Episodios #{start}–#{end} ({}):\n",
        episodes.len()
    ));
    show_episodes(&episodes, true, None, &mut out, &mut err);
    Ok(Rendered::new(out, err))
}
