//! Comandos de parrilla: lectura (paridad byte a byte) y escritura.

pub mod add;
pub mod backfill_slugs;
pub mod get;
pub mod insert;
pub mod list;
pub mod list_range;
pub mod migrate_material;
pub mod move_cmd;
pub mod next;
pub mod next_date;
pub mod open;
pub mod publish;
pub mod rebase_references;
pub mod rename_material;
pub mod renumber;
pub mod srt_stage;
pub mod status;
pub mod update;
pub mod where_cmd;

use crate::model::{Episode, Status};
use crate::profile::ResolvedShow;
use crate::tracker::{Tracker, num_label, save_tracker};

/// Guarda el tracker **sin** emitir la confirmación ni los avisos (modo `--json`).
///
/// Decisión consciente (no un olvido): en modo `--json` el contrato exige **un
/// único documento JSON** en `stdout` y nada más, así que tanto la confirmación
/// `✓ Tracker actualizado: …` como los avisos que en texto irían a `stderr` (p. ej.
/// la colisión de fechas que emite `save_tracker`) se **descartan** a propósito.
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

impl Ctx<'_> {
    /// Carpeta de material de un episodio por la **regla única**
    /// ([`ResolvedShow::material_dir`]), o `None` si el episodio **no tiene slug**.
    ///
    /// Un episodio sin slug **no tiene ruta de material** en ninguna superficie de
    /// lectura: el texto muestra `—`, el JSON emite `null` y `status` omite la
    /// línea. Es la cláusula de la línea base que conservan los deltas de
    /// `tracker-read`/`json-output`; **no** se delega en `material_dir`, que solo
    /// conoce la plantilla. Con `dir: "episodio-{n:02d}"` y un episodio sin slug el
    /// resultado es, por tanto, `None` (no `EpisodeWithoutNumber`): la ausencia de
    /// slug manda.
    ///
    /// # Errors
    ///
    /// Cualquier error de `material_dir` con un episodio **con** slug
    /// (`EpisodeWithoutNumber`, `MaterialDirNotRelative`, …).
    pub fn material(
        &self,
        episode: &Episode,
    ) -> Result<Option<std::path::PathBuf>, crate::error::PodtrackError> {
        if episode.slug_str().is_none() {
            return Ok(None);
        }
        self.show
            .material_dir(episode.slug_str(), episode.number)
            .map(Some)
    }
}

/// Salida de un comando: texto de `stdout`, avisos de `stderr` y código de salida.
#[derive(Debug, Default, Clone)]
pub struct Rendered {
    /// Texto que va a `stdout`.
    pub stdout: String,
    /// Texto que va a `stderr` (avisos).
    pub stderr: String,
    /// Código de salida del comando (0 salvo que el contrato exija otro, p. ej.
    /// `migrate-material` con colisión o `rebase-references` con hook sin título).
    pub exit_code: i32,
}

impl Rendered {
    /// Construye una salida solo con `stdout` (código 0).
    #[must_use]
    pub fn stdout(text: String) -> Self {
        Self {
            stdout: text,
            stderr: String::new(),
            exit_code: 0,
        }
    }

    /// Construye una salida con `stdout` y `stderr` (código 0).
    #[must_use]
    pub fn new(stdout: String, stderr: String) -> Self {
        Self {
            stdout,
            stderr,
            exit_code: 0,
        }
    }

    /// Fija el código de salida.
    #[must_use]
    pub fn with_code(mut self, code: i32) -> Self {
        self.exit_code = code;
        self
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
