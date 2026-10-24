//! Salida estructurada `--json`: aditiva, opt-in y versionada (spec `json-output`).
//!
//! Todo documento incluye `schema_version` (entero) y `show` (slug resuelto).
//! Los errores con `--json` se emiten como un documento con la clave `error`.

use std::path::Path;

use serde_json::{Value, json};

use crate::commands::Ctx;
use crate::commands::list::Filters;
use crate::commands::status::counts;
use crate::model::Episode;
use crate::tracker::{
    DEFAULT_EPISODE_DURATION_MINUTES, calc_next_date, last_published, next_free_number,
    next_public_number, next_to_publish,
};

/// Versión del esquema de la salida JSON.
pub const SCHEMA_VERSION: u32 = 1;

/// Documento de error (va a `stderr`, con `error` y la versión del esquema).
#[must_use]
pub fn error_document(message: &str) -> String {
    let doc = json!({
        "schema_version": SCHEMA_VERSION,
        "error": message,
    });
    format!("{doc}\n")
}

/// Documento JSON del comando `status`.
#[must_use]
pub fn status_document(ctx: &Ctx<'_>) -> String {
    let tracker = ctx.tracker;
    let stats = counts(tracker);
    let next_date = calc_next_date(tracker);
    let next_ep = next_to_publish(tracker);
    let next_to_publish = next_ep.map(|ep| {
        json!({
            "number": ep.number,
            "slug": ep.slug.as_ref().map(|s| s.as_str()),
            "date": ep.date.to_iso(),
            "material": ep.slug.as_ref().map(|s| {
                ctx.show.scripts_root().join(s.as_str()).display().to_string()
            }),
        })
    });
    let last = last_published(tracker);

    document(
        &ctx.show.slug,
        json!({
            "season": tracker.season,
            "season_start": tracker.season_start,
            "episode_duration_minutes": tracker
                .episode_duration_minutes
                .unwrap_or(DEFAULT_EPISODE_DURATION_MINUTES),
            "total": tracker.episodes.len(),
            "counts": {
                "published": stats.published,
                "recorded": stats.recorded,
                "pending": stats.pending,
                "scripted": stats.scripted,
            },
            "next_date": next_date.format("%Y-%m-%d").to_string(),
            "next_number": next_free_number(tracker),
            "last_published": last.map(|ep| json!({
                "number": ep.number,
                "slug": ep.slug.as_ref().map(|s| s.as_str()),
                "date": ep.date.to_iso(),
            })),
            "next_to_publish": next_to_publish,
        }),
    )
}

/// Documento JSON del comando `next`.
#[must_use]
pub fn next_document(ctx: &Ctx<'_>) -> String {
    let tracker = ctx.tracker;
    let next_date = calc_next_date(tracker);
    document(
        &ctx.show.slug,
        json!({
            "next_number": next_free_number(tracker),
            "next_public_number": next_public_number(tracker),
            "next_date": next_date.format("%Y-%m-%d").to_string(),
            "season": tracker.season,
        }),
    )
}

/// Documento JSON del comando `next-date`.
#[must_use]
pub fn next_date_document(ctx: &Ctx<'_>) -> String {
    let next_date = calc_next_date(ctx.tracker);
    document(
        &ctx.show.slug,
        json!({ "next_date": next_date.format("%Y-%m-%d").to_string() }),
    )
}

/// Documento JSON del comando `list`.
#[must_use]
pub fn list_document(ctx: &Ctx<'_>, filters: &Filters, episodes: &[&Episode]) -> String {
    document(
        &ctx.show.slug,
        json!({
            "filter": {
                "status": filters.status,
                "season": filters.season,
                "last": filters.last,
            },
            "episodes": episodes.iter().map(|e| episode_json(e)).collect::<Vec<_>>(),
        }),
    )
}

/// Documento JSON del comando `list-range`.
#[must_use]
pub fn list_range_document(ctx: &Ctx<'_>, start: u64, end: u64, episodes: &[&Episode]) -> String {
    document(
        &ctx.show.slug,
        json!({
            "start": start,
            "end": end,
            "episodes": episodes.iter().map(|e| episode_json(e)).collect::<Vec<_>>(),
        }),
    )
}

/// Documento JSON del comando `get`.
#[must_use]
pub fn get_document(ctx: &Ctx<'_>, episode: &Episode) -> String {
    document(&ctx.show.slug, json!({ "episode": episode_json(episode) }))
}

/// Documento JSON del comando `where`.
#[must_use]
pub fn where_document(ctx: &Ctx<'_>, episode: &Episode, folder: &Path) -> String {
    document(
        &ctx.show.slug,
        json!({
            "episode": {
                "number": episode.number,
                "slug": episode.slug.as_ref().map(|s| s.as_str()),
            },
            "material": folder.display().to_string(),
            "exists": folder.exists(),
        }),
    )
}

fn episode_json(episode: &Episode) -> Value {
    json!({
        "number": episode.number,
        "slug": episode.slug.as_ref().map(|s| s.as_str()),
        "season": episode.season,
        "date": episode.date.to_iso(),
        "title": episode.title,
        "status": episode.status.as_str(),
    })
}

fn document(show: &str, payload: Value) -> String {
    let mut map = serde_json::Map::new();
    map.insert("schema_version".to_string(), json!(SCHEMA_VERSION));
    map.insert("show".to_string(), json!(show));
    if let Value::Object(fields) = payload {
        for (key, value) in fields {
            map.insert(key, value);
        }
    }
    format!("{}\n", Value::Object(map))
}
