//! `add` — alta de un episodio `scripted` (número opcional, slug derivado).

use crate::commands::{Ctx, Rendered, bytes_to_string, save_quiet};
use crate::error::PodtrackError;
use crate::json;
use crate::model::{Date, Episode, Slug, Status, slugify, weekday_es};
use crate::tracker::{Tracker, calc_next_date, next_free_number, save_tracker};

use chrono::Datelike;

/// Parámetros de `add`.
#[derive(Debug, Clone)]
pub struct AddParams {
    /// Número explícito (opcional; si falta se usa el hueco provisional).
    pub number: Option<u32>,
    /// Título del episodio.
    pub title: String,
    /// Fecha de publicación (`YYYY-MM-DD`); por defecto, la siguiente publicable.
    pub date: Option<String>,
    /// Temporada; por defecto, la actual.
    pub season: Option<u16>,
    /// Slug kebab-case explícito (si falta, se deriva del título).
    pub slug: Option<String>,
}

/// Interpreta los posicionales de `add` (`_parse_add_positionals`).
///
/// Acepta `add <num> "<título>"` (número opcional) o `add "<título>"`.
pub fn parse_add_positionals(items: &[String]) -> Result<(Option<u32>, String), PodtrackError> {
    match items {
        [only] => Ok((None, only.clone())),
        [first, second] if looks_like_number(first) => {
            Ok((Some(parse_number(first)?), second.clone()))
        }
        _ => Err(PodtrackError::Usage {
            usage: "add [<num>] \"<título>\" (el número es opcional).".to_string(),
        }),
    }
}

fn looks_like_number(value: &str) -> bool {
    let stripped = value.trim().trim_start_matches(['+', '-']);
    !stripped.is_empty() && stripped.chars().all(|c| c.is_ascii_digit())
}

fn parse_number(value: &str) -> Result<u32, PodtrackError> {
    let parsed = value
        .trim()
        .parse::<i64>()
        .map_err(|_| PodtrackError::InvalidNumber {
            value: value.to_string(),
        })?;
    u32::try_from(parsed).map_err(|_| PodtrackError::InvalidNumber {
        value: value.to_string(),
    })
}

/// Slug libre a partir de `base`, sufijando `-2`, `-3`, … si colisiona (`_unique_slug`).
fn unique_slug(tracker: &Tracker, base: &str) -> String {
    let used: Vec<&str> = tracker
        .episodes
        .iter()
        .filter_map(|e| e.slug.as_ref().map(Slug::as_str))
        .collect();
    if !used.contains(&base) {
        return base.to_string();
    }
    let mut suffix = 2u32;
    loop {
        let candidate = format!("{base}-{suffix}");
        if !used.contains(&candidate.as_str()) {
            return candidate;
        }
        suffix += 1;
    }
}

/// Ejecuta `add`.
pub fn run(ctx: &Ctx<'_>, params: &AddParams, json_mode: bool) -> Result<Rendered, PodtrackError> {
    let tracker = ctx.tracker;

    if let Some(number) = params.number
        && tracker.episodes.iter().any(|e| e.number == Some(number))
    {
        return Err(PodtrackError::AddDuplicate { number });
    }

    let number = params.number.unwrap_or_else(|| next_free_number(tracker));
    let ep_date = match params.date.as_deref().filter(|d| !d.is_empty()) {
        Some(date) => date.to_string(),
        None => calc_next_date(tracker).format("%Y-%m-%d").to_string(),
    };
    let season = params.season.filter(|s| *s != 0).unwrap_or(tracker.season);

    let slug = match params.slug.as_deref().filter(|s| !s.is_empty()) {
        Some(explicit) => Some(Slug::parse(explicit)?),
        None => {
            let derived = slugify(&params.title);
            if derived.is_empty() {
                None
            } else {
                Some(Slug::parse(&unique_slug(tracker, &derived))?)
            }
        }
    };

    let new_episode = Episode {
        number: Some(number),
        slug,
        season,
        date: Date::parse(&ep_date),
        title: params.title.trim().to_string(),
        status: Status::Scripted,
    };

    let mut work = tracker.clone();
    work.episodes.push(new_episode.clone());

    if json_mode {
        save_quiet(&ctx.show.tracker, &ctx.show.name, &work)?;
        return Ok(Rendered::stdout(json::add_document(ctx, &new_episode)));
    }

    let mut out = Vec::new();
    let mut err = Vec::new();
    save_tracker(&ctx.show.tracker, &ctx.show.name, &work, &mut out, &mut err)?;

    let fecha_str = match Date::parse(&ep_date) {
        Date::Known(day) => format!("{ep_date} ({})", weekday_es(day.weekday())),
        _ => ep_date.clone(),
    };
    out.extend_from_slice(
        format!(
            "  ✓ Episodio #{number} añadido:\n    Título:  \"{}\"\n    Fecha:   {fecha_str}\n    Estado:  📝 scripted\n",
            params.title
        )
        .as_bytes(),
    );

    Ok(Rendered::new(bytes_to_string(out), bytes_to_string(err)))
}
