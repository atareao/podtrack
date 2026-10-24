//! `rename-material` — renombra el material por número (`--from/--delta`) o
//! detecta huérfanos (`--check`).

use std::collections::HashSet;
use std::path::PathBuf;

use serde_json::{Value, json};

use crate::commands::{Ctx, DRY_RUN_LINE, Rendered, bytes_to_string};
use crate::error::PodtrackError;
use crate::json;
use crate::material::{apply_renames, orphan_numeric_dirs, plan_number_renames, relative};

/// Parámetros de `rename-material`.
#[derive(Debug, Clone)]
pub struct RenameMaterialParams {
    /// Primer número afectado (`>=`).
    pub from: Option<i64>,
    /// Desplazamiento a aplicar (±d).
    pub delta: Option<i64>,
    /// Solo detectar y listar desajustes.
    pub check: bool,
    /// Previsualizar sin escribir.
    pub dry_run: bool,
}

/// Ejecuta `rename-material`.
pub fn run(
    ctx: &Ctx<'_>,
    params: &RenameMaterialParams,
    json_mode: bool,
) -> Result<Rendered, PodtrackError> {
    let root = ctx.show.scripts_root();

    if params.check {
        let numbers: HashSet<u32> = ctx
            .tracker
            .episodes
            .iter()
            .filter_map(|e| e.number)
            .collect();
        let orphans = orphan_numeric_dirs(&root, &numbers);

        if json_mode {
            let orphans_json: Vec<Value> = orphans
                .iter()
                .map(|(name, number)| json!({ "name": name, "number": number }))
                .collect();
            return Ok(Rendered::stdout(json::rename_material_document(
                ctx,
                true,
                &[],
                &orphans_json,
                None,
                None,
                false,
            )));
        }

        let mut out = String::new();
        out.push_str(&format!(
            "\n  Comprobación de material en {}:\n",
            root.display()
        ));
        if orphans.is_empty() {
            out.push_str("      Material alineado con el tracker.\n");
        } else {
            out.push_str(&format!("      {} desajuste(s):\n", orphans.len()));
            for (name, number) in &orphans {
                out.push_str(&format!(
                    "      · {name}: el número {number} no está en el tracker\n"
                ));
            }
        }
        return Ok(Rendered::stdout(out));
    }

    let (Some(from), Some(delta)) = (params.from, params.delta) else {
        return Err(PodtrackError::RenameMaterialRequiresArgs);
    };
    if delta == 0 {
        return Err(PodtrackError::DeltaZero);
    }

    // Números únicos `>= from`; descendente si `delta > 0` (no pisar destinos).
    let mut numbers: Vec<u32> = ctx
        .tracker
        .episodes
        .iter()
        .filter_map(|e| e.number)
        .filter(|n| i64::from(*n) >= from)
        .collect();
    numbers.sort_unstable();
    numbers.dedup();
    if delta > 0 {
        numbers.reverse();
    }

    let mut plans: Vec<Vec<(PathBuf, PathBuf)>> = Vec::new();
    let mut changes: Vec<Value> = Vec::new();
    let mut total = 0usize;
    for old_num in &numbers {
        let new_num = i64::from(*old_num) + delta;
        let new_num = u32::try_from(new_num).map_err(|_| PodtrackError::InvalidNumber {
            value: new_num.to_string(),
        })?;
        let pairs = plan_number_renames(&root, *old_num, new_num);
        if pairs.is_empty() {
            continue;
        }
        total += pairs.len();
        for (src, dst) in &pairs {
            changes.push(json!({ "from": relative(&root, src), "to": relative(&root, dst) }));
        }
        plans.push(pairs);
    }

    if json_mode {
        if !params.dry_run {
            // Asimetría **intencional** con el modo texto: el oráculo no tiene
            // `--json`, así que la emulación del `stdout` parcial ante un aborto
            // par-a-par (ver más abajo) solo aplica al modo texto. En `--json`
            // preferimos el documento de error tipado: si un destino ya existe,
            // `apply_renames` propaga el `Err` y `main` emite el JSON de error.
            for pairs in &plans {
                let mut sink = Vec::new();
                apply_renames(pairs, &root, &mut sink)?;
            }
        }
        return Ok(Rendered::stdout(json::rename_material_document(
            ctx,
            false,
            &changes,
            &[],
            Some(from),
            Some(delta),
            params.dry_run,
        )));
    }

    let mut out = Vec::new();
    let mut err = Vec::new();
    if params.dry_run {
        out.extend_from_slice(format!("{DRY_RUN_LINE}\n").as_bytes());
    }
    out.extend_from_slice(
        format!("  ✓ Renombrar material desde #{from} con delta {delta:+}:\n").as_bytes(),
    );
    for pairs in &plans {
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
        } else if let Err(error) = apply_renames(pairs, &root, &mut out) {
            // El oráculo aborta **par a par** (`_apply_material_renames`): emite el
            // `stdout` ya producido, el error por `stderr` y sale con 1, sin los
            // resúmenes finales.
            err.extend_from_slice(format!("{error}\n").as_bytes());
            return Ok(Rendered::new(bytes_to_string(out), bytes_to_string(err)).with_code(1));
        }
    }
    if total == 0 {
        out.extend_from_slice("      (sin material que renombrar)\n".as_bytes());
    }
    if !params.dry_run {
        out.extend_from_slice(format!("  ✓ {total} elemento(s) renombrado(s).\n").as_bytes());
    }

    Ok(Rendered::new(bytes_to_string(out), bytes_to_string(err)))
}
