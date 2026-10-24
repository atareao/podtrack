//! `next-date` — próxima fecha publicable.

use chrono::Datelike;

use crate::commands::{Ctx, Rendered};
use crate::error::PodtrackError;
use crate::json;
use crate::model::{today, weekday_es};
use crate::tracker::calc_next_date;

/// Ejecuta `next-date`.
pub fn run(ctx: &Ctx<'_>, json_mode: bool) -> Result<Rendered, PodtrackError> {
    if json_mode {
        return Ok(Rendered::stdout(json::next_date_document(ctx)));
    }

    let next_date = calc_next_date(ctx.tracker);
    let days_until = (next_date - today()).num_days();

    let mut out = String::new();
    out.push_str(&format!(
        "\n  📅 Próxima publicación: {} ({})\n",
        next_date,
        weekday_es(next_date.weekday())
    ));
    if days_until == 0 {
        out.push_str("  ⏰ ¡Hoy! (antes de las 12:00)\n");
    } else if days_until == 1 {
        out.push_str("  ⏰ Mañana\n");
    } else {
        out.push_str(&format!("  ⏰ En {days_until} días\n"));
    }
    out.push('\n');
    Ok(Rendered::stdout(out))
}
