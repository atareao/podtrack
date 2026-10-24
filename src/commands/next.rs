//! `next` — hueco provisional y número autoritativo a publicar.

use chrono::Datelike;

use crate::commands::{Ctx, Rendered};
use crate::error::PodtrackError;
use crate::json;
use crate::model::weekday_es;
use crate::tracker::{calc_next_date, next_free_number, next_public_number};

/// Ejecuta `next`.
pub fn run(ctx: &Ctx<'_>, json_mode: bool) -> Result<Rendered, PodtrackError> {
    if json_mode {
        return Ok(Rendered::stdout(json::next_document(ctx)));
    }

    let tracker = ctx.tracker;
    let next_num = next_free_number(tracker);
    let public_num = next_public_number(tracker);
    let next_date = calc_next_date(tracker);

    let mut out = String::new();
    out.push_str(&format!(
        "\n  Siguiente episodio: #{}  (Temporada {})\n",
        next_num, tracker.season
    ));
    out.push_str(&format!("  Próximo a publicar: #{public_num}\n"));
    out.push_str(&format!(
        "  Fecha propuesta:    {} ({})\n\n",
        next_date,
        weekday_es(next_date.weekday())
    ));
    Ok(Rendered::stdout(out))
}
