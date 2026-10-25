//! `get` — detalle de un episodio (por número o slug).

use crate::commands::{Ctx, Rendered, resolve_episode};
use crate::error::PodtrackError;
use crate::json;
use crate::model::fmt_date;
use crate::tracker::num_label;

/// Ejecuta `get`.
pub fn run(ctx: &Ctx<'_>, target: &str, json_mode: bool) -> Result<Rendered, PodtrackError> {
    let episode =
        resolve_episode(ctx.tracker, target).ok_or_else(|| PodtrackError::EpisodeNotFound {
            target: target.to_string(),
        })?;

    if json_mode {
        return Ok(Rendered::stdout(json::get_document(ctx, episode)));
    }

    let slug = episode.slug_str().unwrap_or_default();

    let mut out = String::new();
    out.push_str(&format!("\n  {}\n", "═".repeat(55)));
    out.push_str(&format!("  Episodio #{}\n", num_label(episode.number)));
    out.push_str(&format!("  {}\n", "═".repeat(55)));
    out.push_str(&format!("  Título:    {}\n", episode.title));
    out.push_str(&format!(
        "  Estado:    {} {}\n",
        episode.status.icon(),
        episode.status.as_str()
    ));
    out.push_str(&format!("  Fecha:     {}\n", fmt_date(&episode.date)));
    out.push_str(&format!("  Temporada: {}\n", episode.season));
    if slug.is_empty() {
        out.push_str(&format!("  {:<10} {}\n", "Slug:", "—"));
        out.push_str(&format!("  {:<10} {}\n", "Ruta:", "—"));
    } else {
        out.push_str(&format!("  {:<10} {}\n", "Slug:", slug));
        out.push_str(&format!(
            "  {:<10} {}\n",
            "Ruta:",
            ctx.show.scripts_root().join(slug).display()
        ));
    }
    out.push_str(&format!("  {}\n\n", "═".repeat(55)));
    Ok(Rendered::stdout(out))
}
