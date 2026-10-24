//! `list` — listado filtrable, con marca `*` de no publicados.

use crate::commands::{Ctx, Rendered, show_episodes};
use crate::error::PodtrackError;
use crate::json;
use crate::model::{Episode, Status, sort_episodes};

/// Filtros de `list`.
#[derive(Debug, Default, Clone)]
pub struct Filters {
    /// Filtro por estado.
    pub status: Option<String>,
    /// Filtro por temporada.
    pub season: Option<u16>,
    /// Solo los últimos N.
    pub last: Option<usize>,
}

/// Ejecuta `list`.
pub fn run(ctx: &Ctx<'_>, filters: &Filters, json_mode: bool) -> Result<Rendered, PodtrackError> {
    let tracker = ctx.tracker;

    // `status` vacío se comporta como ausente (verdad de Python).
    let status_filter = filters
        .status
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .map(Status::parse)
        .transpose()?;
    let last = filters.last.filter(|n| *n != 0);
    let season = filters.season;

    let mut filtered: Vec<&Episode> = match status_filter {
        Some(status) => tracker
            .episodes
            .iter()
            .filter(|e| e.status == status)
            .collect(),
        None => tracker.episodes.iter().collect(),
    };
    if let Some(season) = season {
        filtered.retain(|e| e.season == season);
    }

    let mut owned: Vec<Episode> = filtered.into_iter().cloned().collect();
    match last {
        Some(n) => {
            sort_episodes(&mut owned);
            let len = owned.len();
            if n < len {
                owned.drain(0..len - n);
            }
            owned.reverse();
        }
        None => sort_episodes(&mut owned),
    }
    let view: Vec<&Episode> = owned.iter().collect();

    if json_mode {
        return Ok(Rendered::stdout(json::list_document(ctx, filters, &view)));
    }

    let mut parts = Vec::new();
    if let Some(status) = filters.status.as_deref().filter(|s| !s.trim().is_empty()) {
        parts.push(format!("[{status}]"));
    }
    if let Some(season) = season {
        parts.push(format!("Temporada {season}"));
    }
    if let Some(n) = last {
        parts.push(format!("últimos {n}"));
    }
    let label = if parts.is_empty() {
        "Todos".to_string()
    } else {
        parts.join(" ")
    };

    let mut out = String::new();
    let mut err = String::new();
    out.push_str(&format!("\n  {} ({} episodios):\n", label, view.len()));
    show_episodes(&view, true, &mut out, &mut err);
    Ok(Rendered {
        stdout: out,
        stderr: err,
    })
}
