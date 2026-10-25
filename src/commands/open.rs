//! `open` — resuelve la **fuente** de la presentación Typst de un episodio a
//! partir del layout de material del perfil y la abre con `typshow` sin bloquear.
//!
//! Es una extensión **aditiva** (el oráculo Python no conoce `open`): no toca la
//! salida, los códigos ni el YAML de ningún subcomando portado.

use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::commands::{Ctx, Rendered, resolve_episode};
use crate::error::PodtrackError;
use crate::json;
use crate::profile::render_material_template;
use crate::tracker::num_label;

/// Lanzador de la presentación: se busca en el `PATH` (sin override).
pub const LAUNCHER: &str = "typshow";

/// Línea literal que acompaña a la ruta en `--dry-run` (no se lanza nada).
pub const DRY_RUN_NO_LAUNCH: &str = "  ⚠ DRY-RUN: no se lanza nada";

/// Ejecuta `open`: resuelve la ruta de la fuente `.typ` de la presentación y, si
/// no es `--dry-run`, lanza `typshow` en segundo plano.
///
/// # Errors
///
/// - `EpisodeNotFound` si `target` no corresponde a ningún episodio.
/// - `PresentationNotDeclared` (exit 2) si el perfil no declara la presentación.
/// - `EpisodeWithoutSlug` / `EpisodeWithoutNumber` si el layout exige un dato que
///   el episodio no tiene.
/// - `PresentationNotFound` (exit 1) si el `.typ` resuelto no existe en disco.
/// - `LauncherNotFound` (exit 1) si `typshow` no está en el `PATH`; `Io` en
///   cualquier otro fallo del lanzamiento.
pub fn run(
    ctx: &Ctx<'_>,
    target: &str,
    dry_run: bool,
    json_mode: bool,
) -> Result<Rendered, PodtrackError> {
    let episode =
        resolve_episode(ctx.tracker, target).ok_or_else(|| PodtrackError::EpisodeNotFound {
            target: target.to_string(),
        })?;

    let slug_opt = episode.slug_str();
    let label = num_label(episode.number);

    let template = ctx
        .show
        .presentation
        .as_deref()
        .filter(|t| !t.is_empty())
        .ok_or_else(|| PodtrackError::PresentationNotDeclared {
            show: ctx.show.slug.clone(),
        })?;

    // La carpeta sale de la **regla única** de material (spec `show-profile`),
    // compartida con `where`/`get`/`status`: así `open` y la lectura no pueden
    // divergir. Hereda además el endurecimiento D5 (un `dir` no relativo es error).
    let dir = ctx.show.material_dir(slug_opt, episode.number)?;
    let file_name = render_material_template(template, slug_opt, episode.number, &label)?;
    let file_name = Path::new(&file_name).with_extension("typ");

    let path = dir.join(file_name);

    if !path.is_file() {
        return Err(PodtrackError::PresentationNotFound {
            slug: slug_opt.unwrap_or(target).to_string(),
            path,
        });
    }

    let launched = if dry_run {
        false
    } else {
        spawn_launcher(&path)?;
        true
    };

    if json_mode {
        return Ok(Rendered::stdout(json::open_document(
            ctx, episode, &path, launched,
        )));
    }

    let mut text = format!("▶ {LAUNCHER}: {}\n", path.display());
    if dry_run {
        text.push_str(DRY_RUN_NO_LAUNCH);
        text.push('\n');
    }
    Ok(Rendered::stdout(text))
}

/// Lanza `typshow <path>` **en segundo plano** (sin `wait()`), con el *stdio*
/// silenciado y en su propio grupo de procesos para que sobreviva a señales
/// dirigidas al grupo del invocador.
fn spawn_launcher(path: &Path) -> Result<(), PodtrackError> {
    let result = Command::new(LAUNCHER)
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn();

    match result {
        Ok(_child) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Err(PodtrackError::LauncherNotFound {
                launcher: LAUNCHER.to_string(),
            })
        }
        Err(e) => Err(PodtrackError::Io(e.to_string())),
    }
}
