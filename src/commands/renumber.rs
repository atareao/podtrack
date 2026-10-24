//! `renumber` — renumeración de la parrilla (solo números, sin fechas ni material).

use serde_json::{Value, json};

use crate::commands::{Ctx, DRY_RUN_LINE, Rendered, bytes_to_string, save_quiet};
use crate::error::PodtrackError;
use crate::json;
use crate::tracker::save_tracker;

/// Parámetros de `renumber`.
#[derive(Debug, Clone)]
pub struct RenumberParams {
    /// Primer número afectado (`>=`).
    pub from: i64,
    /// Desplazamiento a aplicar (±d).
    pub delta: i64,
    /// Previsualizar sin escribir.
    pub dry_run: bool,
}

/// Ejecuta `renumber`.
pub fn run(
    ctx: &Ctx<'_>,
    params: &RenumberParams,
    json_mode: bool,
) -> Result<Rendered, PodtrackError> {
    if params.delta == 0 {
        return Err(PodtrackError::DeltaZero);
    }

    let tracker = ctx.tracker;
    let mut affected: Vec<(usize, u32)> = tracker
        .episodes
        .iter()
        .enumerate()
        .filter_map(|(index, e)| {
            e.number
                .filter(|n| i64::from(*n) >= params.from)
                .map(|n| (index, n))
        })
        .collect();
    if affected.is_empty() {
        return Err(PodtrackError::RenumberNoEpisodes { from: params.from });
    }

    let untouched: Vec<i64> = tracker
        .episodes
        .iter()
        .filter_map(|e| e.number)
        .filter(|n| i64::from(*n) < params.from)
        .map(i64::from)
        .collect();
    let new_numbers: Vec<i64> = affected
        .iter()
        .map(|(_, n)| i64::from(*n) + params.delta)
        .collect();
    let mut all = untouched;
    all.extend(new_numbers.iter().copied());
    let mut unique = all.clone();
    unique.sort_unstable();
    unique.dedup();
    if unique.len() != all.len() {
        return Err(PodtrackError::RenumberCollision);
    }

    // Orden ascendente de número antiguo para el informe.
    affected.sort_by_key(|(_, n)| *n);

    let mut changes: Vec<(u32, u32)> = Vec::new();
    for (_, old) in &affected {
        let new = i64::from(*old) + params.delta;
        let new = u32::try_from(new).map_err(|_| PodtrackError::InvalidNumber {
            value: new.to_string(),
        })?;
        changes.push((*old, new));
    }

    if json_mode {
        if !params.dry_run {
            let mut work = tracker.clone();
            for ((index, _), (_, new)) in affected.iter().zip(changes.iter()) {
                work.episodes[*index].number = Some(*new);
            }
            save_quiet(&ctx.show.tracker, &ctx.show.name, &work)?;
        }
        let changes_json: Vec<Value> = changes
            .iter()
            .map(|(old, new)| json!({ "from": old, "to": new }))
            .collect();
        return Ok(Rendered::stdout(json::renumber_document(
            ctx,
            params.from,
            params.delta,
            &changes_json,
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
            "  ✓ Renumerar desde #{} con delta {:+}:\n",
            params.from, params.delta
        )
        .as_bytes(),
    );
    let mut work = tracker.clone();
    for ((index, old), (_, new)) in affected.iter().zip(changes.iter()) {
        out.extend_from_slice(format!("      #{old} → #{new}\n").as_bytes());
        if !params.dry_run {
            work.episodes[*index].number = Some(*new);
        }
    }

    if !params.dry_run {
        save_tracker(&ctx.show.tracker, &ctx.show.name, &work, &mut out, &mut err)?;
    }

    Ok(Rendered::new(bytes_to_string(out), bytes_to_string(err)))
}
