//! Comandos de parrilla: lectura (paridad byte a byte) y escritura.

pub mod add;
pub mod get;
pub mod insert;
pub mod list;
pub mod list_range;
pub mod move_cmd;
pub mod next;
pub mod next_date;
pub mod publish;
pub mod status;
pub mod update;
pub mod where_cmd;

use crate::model::{Episode, Status};
use crate::profile::ResolvedShow;
use crate::tracker::{Tracker, num_label, save_tracker};

/// Guarda el tracker **sin** emitir la confirmación (modo `--json`, donde solo se
/// emite un documento).
pub fn save_quiet(
    tracker_path: &std::path::Path,
    name: &str,
    tracker: &Tracker,
) -> Result<(), crate::error::PodtrackError> {
    let mut out = Vec::new();
    let mut err = Vec::new();
    save_tracker(tracker_path, name, tracker, &mut out, &mut err)
}

/// Línea literal de `--dry-run` (`_print_dry_run`).
pub const DRY_RUN_LINE: &str = "  ⚠ DRY-RUN: no se escribe nada (ni tracker ni disco)";

/// Índice del episodio que corresponde a `target` (número o slug), o `None`
/// (`_resolve_episode`).
#[must_use]
pub fn resolve_index(tracker: &Tracker, target: &str) -> Option<usize> {
    let trimmed = target.trim();
    if !trimmed.is_empty()
        && trimmed.chars().all(|c| c.is_ascii_digit())
        && let Ok(number) = trimmed.parse::<u64>()
    {
        return tracker
            .episodes
            .iter()
            .position(|e| e.number.is_some_and(|n| u64::from(n) == number));
    }
    tracker
        .episodes
        .iter()
        .position(|e| e.slug.as_ref().is_some_and(|s| s.as_str() == trimmed))
}

/// Resuelve un episodio por número (dígitos) o por slug (`_resolve_episode`).
#[must_use]
pub fn resolve_episode<'a>(tracker: &'a Tracker, target: &str) -> Option<&'a Episode> {
    resolve_index(tracker, target).map(|index| &tracker.episodes[index])
}

/// Contexto compartido por los comandos de lectura.
pub struct Ctx<'a> {
    /// Tracker cargado.
    pub tracker: &'a Tracker,
    /// Perfil resuelto (nombre y raíz de guiones).
    pub show: &'a ResolvedShow,
}

/// Salida de un comando: texto de `stdout` y avisos de `stderr`.
#[derive(Debug, Default, Clone)]
pub struct Rendered {
    /// Texto que va a `stdout`.
    pub stdout: String,
    /// Texto que va a `stderr` (avisos).
    pub stderr: String,
}

impl Rendered {
    /// Construye una salida solo con `stdout`.
    #[must_use]
    pub fn stdout(text: String) -> Self {
        Self {
            stdout: text,
            stderr: String::new(),
        }
    }
}

/// Interpreta un buffer de salida como texto UTF-8 (nunca falla: es texto propio).
#[must_use]
pub fn bytes_to_string(buffer: Vec<u8>) -> String {
    String::from_utf8_lossy(&buffer).into_owned()
}

/// Celda de la columna «Nº» (ancho 4), con `—` para números nulos y `*` si es
/// provisional (`_fmt_number_cell`).
#[must_use]
pub fn fmt_number_cell(number: Option<u32>, provisional: bool) -> String {
    match number {
        None => format!("{:>4}", "—"),
        Some(n) if provisional => format!("{n:>3}*"),
        Some(n) => format!("{n:>4}"),
    }
}

/// Avisa por `err` de fechas compartidas (`_warn_date_collisions`).
pub fn warn_date_collisions(episodes: &[&Episode], err: &mut String) {
    let mut order: Vec<String> = Vec::new();
    let mut by_date: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for ep in episodes {
        if ep.date.is_no_date() {
            continue;
        }
        let fecha = ep.date.to_iso();
        if !by_date.contains_key(&fecha) {
            order.push(fecha.clone());
        }
        by_date.entry(fecha).or_default().push(num_label(ep.number));
    }
    order.sort();
    for fecha in order {
        if let Some(nums) = by_date.get(&fecha)
            && nums.len() > 1
        {
            err.push_str(&format!(
                "  ⚠ Aviso: la fecha {fecha} la comparten {} episodios (#{}).\n",
                nums.len(),
                nums.join(", ")
            ));
        }
    }
}

/// Muestra una lista de episodios formateada (`show_episodes`).
pub fn show_episodes(
    episodes: &[&Episode],
    mark_provisional: bool,
    out: &mut String,
    err: &mut String,
) {
    if episodes.is_empty() {
        out.push_str("  No hay episodios.\n");
        return;
    }

    out.push_str(&format!(
        "  {:>5}  {:<13}  {:<12}  {:<5}  Título\n",
        "Nº", "Fecha", "Estado", "Temporada"
    ));
    out.push_str(&format!(
        "  {}  {}  {}  {}  {}\n",
        "─".repeat(5),
        "─".repeat(13),
        "─".repeat(12),
        "─".repeat(9),
        "─".repeat(29)
    ));

    warn_date_collisions(episodes, err);

    let mut has_provisional = false;
    for ep in episodes {
        let provisional = mark_provisional && ep.status != Status::Published && ep.number.is_some();
        has_provisional = has_provisional || provisional;
        let title: String = ep.title.chars().take(50).collect();
        out.push_str(&format!(
            "  {}   {:<13} {} {:<9}  S{:<4}  {}\n",
            fmt_number_cell(ep.number, provisional),
            ep.date.to_iso(),
            ep.status.icon(),
            ep.status.as_str(),
            ep.season,
            title
        ));
    }
    if mark_provisional && has_provisional {
        out.push_str("  * número provisional (se fija al publicar)\n");
    }
    out.push('\n');
}
