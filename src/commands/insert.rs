//! `insert` — inserta un episodio en el hueco de otro, desplazando los posteriores
//! (números, fechas y material).

use serde_json::{Value, json};

use crate::commands::{Ctx, DRY_RUN_LINE, Rendered, save_quiet};
use crate::error::PodtrackError;
use crate::json;
use crate::material::{apply_renames, plan_number_renames, relative};
use crate::model::{Date, Episode, Slug, Status, next_slot, parse_iso};
use crate::tracker::save_tracker;

/// Parámetros de `insert`.
#[derive(Debug, Clone)]
pub struct InsertParams {
    /// Número del episodio ante el que insertar.
    pub before: u32,
    /// Título del episodio nuevo.
    pub title: String,
    /// Slug kebab-case (opcional).
    pub slug: Option<String>,
    /// Fecha del slot (debe coincidir con la de `before`).
    pub date: Option<String>,
    /// Previsualizar sin escribir.
    pub dry_run: bool,
}

/// Desplazamiento de un episodio (`#from → #to` con su fecha).
struct Shift {
    from: u32,
    to: u32,
    date_from: String,
    date_to: String,
}

/// Ejecuta `insert`.
pub fn run(
    ctx: &Ctx<'_>,
    params: &InsertParams,
    json_mode: bool,
) -> Result<Rendered, PodtrackError> {
    let tracker = ctx.tracker;

    let target_index = tracker
        .episodes
        .iter()
        .position(|e| e.number == Some(params.before))
        .ok_or(PodtrackError::InsertMissing {
            number: params.before,
        })?;

    let slot_date = tracker.episodes[target_index].date.to_iso();
    let provided = params.date.as_deref().filter(|d| !d.is_empty());
    if let Some(date) = provided
        && *date != slot_date
    {
        return Err(PodtrackError::InsertDateMismatch {
            date: date.to_string(),
            before: params.before,
            slot: slot_date.clone(),
        });
    }
    let new_date = provided.map_or_else(|| slot_date.clone(), ToString::to_string);

    let root = ctx.show.scripts_root();

    // Números afectados (>= before), de mayor a menor: primero el plan de
    // renombrado (con los números ORIGINALES) y luego los desplazamientos.
    let mut affected: Vec<u32> = tracker
        .episodes
        .iter()
        .filter_map(|e| e.number.filter(|n| *n >= params.before))
        .collect();
    affected.sort_unstable_by(|a, b| b.cmp(a));

    let mut rename_plan: Vec<Vec<(std::path::PathBuf, std::path::PathBuf)>> = Vec::new();
    for old_num in &affected {
        let pairs = plan_number_renames(&root, *old_num, old_num + 1);
        if !pairs.is_empty() {
            rename_plan.push(pairs);
        }
    }

    // Índices afectados, en el mismo orden (número descendente).
    let mut order: Vec<usize> = tracker
        .episodes
        .iter()
        .enumerate()
        .filter(|(_, e)| e.number.is_some_and(|n| n >= params.before))
        .map(|(index, _)| index)
        .collect();
    order.sort_by_key(|index| std::cmp::Reverse(tracker.episodes[*index].number));

    let mut shifts = Vec::new();
    for index in &order {
        if let Some(old_num) = tracker.episodes[*index].number {
            let date_from = tracker.episodes[*index].date.to_iso();
            let date_to = if date_from == "TBD" {
                date_from.clone()
            } else {
                next_slot(parse_iso(&date_from)?)
                    .format("%Y-%m-%d")
                    .to_string()
            };
            shifts.push(Shift {
                from: old_num,
                to: old_num + 1,
                date_from,
                date_to,
            });
        }
    }

    let new_episode = Episode {
        number: Some(params.before),
        slug: match params.slug.as_deref().filter(|s| !s.is_empty()) {
            Some(slug) => Some(Slug::parse(slug)?),
            None => None,
        },
        season: tracker.episodes[target_index].season,
        date: Date::parse(&new_date),
        title: params.title.trim().to_string(),
        status: Status::Scripted,
    };

    let mut work = tracker.clone();
    for (index, shift) in order.iter().zip(shifts.iter()) {
        work.episodes[*index].number = Some(shift.to);
        work.episodes[*index].date = Date::parse(&shift.date_to);
    }
    work.episodes.push(new_episode.clone());

    if json_mode {
        let shifts_json: Vec<Value> = shifts
            .iter()
            .map(|s| {
                json!({
                    "from": s.from,
                    "to": s.to,
                    "date_from": s.date_from,
                    "date_to": s.date_to,
                })
            })
            .collect();
        let material_json: Vec<Value> = rename_plan
            .iter()
            .flat_map(|pairs| {
                pairs.iter().map(|(src, dst)| {
                    json!({ "from": relative(&root, src), "to": relative(&root, dst) })
                })
            })
            .collect();

        if !params.dry_run {
            for pairs in &rename_plan {
                let mut sink = Vec::new();
                apply_renames(pairs, &root, &mut sink)?;
            }
            save_quiet(&ctx.show.tracker, &ctx.show.name, &work)?;
        }
        return Ok(Rendered::stdout(json::insert_document(
            ctx,
            &new_episode,
            shifts_json,
            material_json,
            params.dry_run,
        )));
    }

    let mut out = Vec::new();
    let mut err = Vec::new();
    if params.dry_run {
        out.extend_from_slice(format!("{DRY_RUN_LINE}\n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "  ✓ Insertar antes de #{}: \"{}\"\n",
            params.before,
            params.title.trim()
        )
        .as_bytes(),
    );
    out.extend_from_slice(format!("    Nuevo: #{}  ({new_date})\n", params.before).as_bytes());
    for shift in &shifts {
        out.extend_from_slice(
            format!(
                "      #{} → #{}  ({} → {})\n",
                shift.from, shift.to, shift.date_from, shift.date_to
            )
            .as_bytes(),
        );
    }

    for pairs in &rename_plan {
        if params.dry_run {
            for (src, dst) in pairs {
                out.extend_from_slice(
                    format!(
                        "      · {}  →  {}\n",
                        relative(&root, src),
                        relative(&root, dst)
                    )
                    .as_bytes(),
                );
            }
        } else {
            apply_renames(pairs, &root, &mut out)?;
        }
    }

    if !params.dry_run {
        save_tracker(&ctx.show.tracker, &ctx.show.name, &work, &mut out, &mut err)?;
    }

    Ok(Rendered::new(
        crate::commands::bytes_to_string(out),
        crate::commands::bytes_to_string(err),
    ))
}
