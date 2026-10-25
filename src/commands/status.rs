//! `status` — resumen de la parrilla y bloque «Siguiente a publicar».

use chrono::Datelike;

use crate::commands::{Ctx, Rendered, show_episodes};
use crate::error::PodtrackError;
use crate::json;
use crate::model::{Status, weekday_es};
use crate::tracker::{
    DEFAULT_EPISODE_DURATION_MINUTES, Tracker, calc_next_date, last_published, next_to_publish,
    num_label,
};

/// Próximos a publicar que se muestran en la ventana final de `status`.
const WINDOW_UPCOMING: usize = 7;

/// Totales por estado (más «unknown», siempre 0 con el enum cerrado).
#[derive(Debug, Default, Clone, Copy)]
pub struct Counts {
    /// Publicados.
    pub published: usize,
    /// Grabados.
    pub recorded: usize,
    /// Pendientes.
    pub pending: usize,
    /// Guionizados.
    pub scripted: usize,
    /// Estados desconocidos (0: el enum es cerrado).
    pub unknown: usize,
}

/// Cuenta episodios por estado.
#[must_use]
pub fn counts(tracker: &Tracker) -> Counts {
    let mut counts = Counts::default();
    for ep in &tracker.episodes {
        match ep.status {
            Status::Published => counts.published += 1,
            Status::Recorded => counts.recorded += 1,
            Status::Pending => counts.pending += 1,
            Status::Scripted => counts.scripted += 1,
        }
    }
    counts.unknown = tracker.episodes.len()
        - (counts.published + counts.recorded + counts.pending + counts.scripted);
    counts
}

/// Ejecuta `status`.
pub fn run(ctx: &Ctx<'_>, json_mode: bool) -> Result<Rendered, PodtrackError> {
    if json_mode {
        return Ok(Rendered::stdout(json::status_document(ctx)?));
    }

    let tracker = ctx.tracker;
    let stats = counts(tracker);
    let total = tracker.episodes.len();
    let season_start = tracker
        .season_start
        .clone()
        .unwrap_or_else(|| "?".to_string());
    let duration = tracker
        .episode_duration_minutes
        .unwrap_or(DEFAULT_EPISODE_DURATION_MINUTES);
    let next_date = calc_next_date(tracker);
    let last = last_published(tracker);

    let mut out = String::new();
    let mut err = String::new();

    out.push_str(&format!("\n{}\n", "=".repeat(65)));
    out.push_str(&format!("  🎙  {} — Tracker de episodios\n", ctx.show.name));
    out.push_str(&format!("{}\n", "=".repeat(65)));
    out.push_str(&format!(
        "  Temporada:      {}  (desde {})\n",
        tracker.season, season_start
    ));
    out.push_str("  Publicación:    Lunes y Jueves\n");
    out.push_str(&format!("  Duración:       ~{duration} min/episodio\n"));
    out.push_str(&format!("  Total epis:     {total}\n"));
    out.push_str("  ───────────────────────────────────────\n");
    out.push_str(&format!("  ✅ Publicados:  {}", stats.published));
    if let Some(ep) = last {
        out.push_str(&format!(
            "  (último: #{} — {})\n",
            num_label(ep.number),
            ep.date.to_iso()
        ));
    } else {
        out.push('\n');
    }
    out.push_str(&format!("  🎙 Grabados:    {}\n", stats.recorded));
    out.push_str(&format!("  ⬜ Pendientes:  {}\n", stats.pending));
    out.push_str(&format!("  📝 Guionizados: {}\n", stats.scripted));
    if stats.unknown > 0 {
        out.push_str(&format!("  ⬜ Sin estado:  {}\n", stats.unknown));
    }
    out.push_str("  ───────────────────────────────────────\n");
    // quirk del oráculo: la línea «▶ Siguiente» usa el literal `total + 726`
    // (herencia histórica del script); NO es el hueco de parrilla ni el número a
    // publicar. Se replica tal cual por paridad de contrato.
    out.push_str(&format!(
        "  ▶ Siguiente:    #{} — {} ({})\n",
        total + 726,
        next_date,
        weekday_es(next_date.weekday())
    ));
    if let Some(ep) = next_to_publish(tracker) {
        for line in next_to_publish_block(ctx, ep)? {
            out.push_str(&line);
            out.push('\n');
        }
    }
    out.push_str(&format!("{}\n\n", "=".repeat(65)));

    // Ventana final: el último publicado seguido de los próximos a publicar.
    // Deliberadamente distinta del oráculo retirado (change `status-upcoming-window`):
    // responde a «¿qué está publicado y qué toca publicar?», no a «qué es lo último
    // de la parrilla». El publicado va primero y sin marca; los próximos, en orden
    // ascendente de fecha y con la marca provisional.
    out.push_str("  Último publicado y próximos a publicar:\n");
    let mut window: Vec<&crate::model::Episode> = Vec::new();
    if let Some(ep) = last {
        window.push(ep);
    }
    let mut upcoming: Vec<&crate::model::Episode> = tracker
        .episodes
        .iter()
        .filter(|e| e.status != Status::Published)
        .collect();
    upcoming.sort_by_key(|e| e.sort_key());
    window.extend(upcoming.into_iter().take(WINDOW_UPCOMING));
    show_episodes(&window, true, &mut out, &mut err);

    Ok(Rendered::new(out, err))
}

/// Líneas del bloque «Siguiente a publicar» (`_next_to_publish_block`).
///
/// La ruta sale de la **regla única** ([`Ctx::material`]): un episodio sin slug
/// muestra `—` y **no** inventa ruta (se omite la línea `ruta:`), igual que su
/// documento `--json` deja `material` a `null`. Cualquier otro error del layout se
/// propaga.
///
/// # Errors
///
/// Cualquier error de `material_dir` con un episodio **con** slug.
pub fn next_to_publish_block(
    ctx: &Ctx<'_>,
    episode: &crate::model::Episode,
) -> Result<Vec<String>, PodtrackError> {
    let slug = episode.slug_str().unwrap_or_default();
    let numero = num_label(episode.number);
    let mut lines = vec![
        format!("  ▶ Siguiente a publicar: #{} «{}»", numero, episode.title),
        format!("      slug:  {}", if slug.is_empty() { "—" } else { slug }),
    ];
    if let Some(path) = ctx.material(episode)? {
        lines.push(format!("      ruta:  {}", path.display()));
    }
    Ok(lines)
}
