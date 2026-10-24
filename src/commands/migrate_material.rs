//! `migrate-material` — migra el material de la convención numérica a la de slug.

use serde_json::{Value, json};

use crate::commands::{Ctx, DRY_RUN_LINE, Rendered, bytes_to_string};
use crate::error::PodtrackError;
use crate::json;
use crate::material::{apply_slug_renames, collect_slug_renames, relative, slug_collisions};
use crate::model::{Status, sort_episodes};

/// Parámetros de `migrate-material`.
#[derive(Debug, Clone)]
pub struct MigrateParams {
    /// Incluir también los episodios publicados.
    pub include_published: bool,
    /// Previsualizar sin escribir.
    pub dry_run: bool,
}

fn folder_name(path: &std::path::Path) -> String {
    path.file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
}

/// Ejecuta `migrate-material`.
pub fn run(
    ctx: &Ctx<'_>,
    params: &MigrateParams,
    json_mode: bool,
) -> Result<Rendered, PodtrackError> {
    let root = ctx.show.scripts_root();

    let mut out = Vec::new();
    let mut err = Vec::new();
    if params.dry_run {
        out.extend_from_slice(format!("{DRY_RUN_LINE}\n").as_bytes());
    }

    let mut episodes = ctx.tracker.episodes.clone();
    sort_episodes(&mut episodes);

    let mut had_collision = false;
    let mut migrated = 0usize;
    let mut migrations: Vec<Value> = Vec::new();
    let mut collisions: Vec<Value> = Vec::new();
    let mut skipped: Vec<Value> = Vec::new();

    for episode in &episodes {
        if episode.status == Status::Published && !params.include_published {
            continue;
        }
        let etiqueta = format!("#{}", crate::tracker::num_label(episode.number));
        let slug = episode
            .slug
            .as_ref()
            .map(|s| s.as_str().trim().to_string())
            .unwrap_or_default();
        if slug.is_empty() {
            err.extend_from_slice(
                format!("  ⚠ Aviso: el episodio {etiqueta} no tiene slug; se salta.\n").as_bytes(),
            );
            skipped.push(json!({ "number": episode.number, "reason": "sin-slug" }));
            continue;
        }

        let Some(number) = episode.number else {
            continue;
        };
        let src = root.join(format!("{number:04}"));
        if !src.is_dir() {
            continue;
        }

        let pairs = collect_slug_renames(&root, &src, number, &slug);
        let problems = slug_collisions(&pairs);
        if !problems.is_empty() {
            had_collision = true;
            err.extend_from_slice(
                format!(
                    "  ✗ Colisión al migrar {} → {slug}: no se renombra nada.\n",
                    folder_name(&src)
                )
                .as_bytes(),
            );
            for problem in &problems {
                err.extend_from_slice(format!("      · {problem}\n").as_bytes());
            }
            collisions.push(json!({
                "folder": folder_name(&src),
                "slug": slug,
                "problems": problems,
            }));
            continue;
        }

        out.extend_from_slice(format!("  ✓ Migrar {} → {slug}:\n", folder_name(&src)).as_bytes());
        if params.dry_run {
            for (old, new) in &pairs {
                out.extend_from_slice(
                    format!(
                        "      · {}  →  {}\n",
                        relative(&root, old),
                        relative(&root, new)
                    )
                    .as_bytes(),
                );
            }
        } else {
            apply_slug_renames(&pairs, &root, &mut out)?;
        }
        migrated += 1;
        migrations.push(json!({ "from": folder_name(&src), "to": slug }));
    }

    if migrated == 0 && !had_collision {
        out.extend_from_slice("      (sin material que migrar)\n".as_bytes());
    }

    let code = if had_collision { 1 } else { 0 };

    if json_mode {
        return Ok(Rendered::stdout(json::migrate_material_document(
            ctx,
            &migrations,
            &collisions,
            &skipped,
            params.include_published,
            params.dry_run,
        ))
        .with_code(code));
    }

    Ok(Rendered::new(bytes_to_string(out), bytes_to_string(err)).with_code(code))
}
