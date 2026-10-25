//! `srt-stage` — clasifica la etapa de transcripción de los SRT de un podcast.

use std::collections::BTreeSet;
use std::path::PathBuf;

use serde_json::{Value, json};

use crate::commands::Rendered;
use crate::error::PodtrackError;
use crate::json;
use crate::profile::ShowResolver;
use crate::srt_stage;

/// Parámetros de `srt-stage`.
#[derive(Debug, Clone)]
pub struct SrtStageParams {
    /// Perfil del show (`--show`), o el por defecto.
    pub show: Option<String>,
    /// Ruta a un SRT concreto.
    pub srt: Option<String>,
    /// Número de episodio (clasifica todos sus SRT).
    pub episode: Option<i64>,
    /// Etapa exigida (`raw`/`final`).
    pub expect: Option<String>,
}

/// Ejecuta `srt-stage`.
pub fn run(
    resolver: &ShowResolver,
    params: &SrtStageParams,
    json_mode: bool,
) -> Result<Rendered, PodtrackError> {
    if params.srt.is_none() && params.episode.is_none() {
        return Err(srt_stage::stage_error(
            "ERROR: Indica --srt <ruta> o --episode NN.",
        ));
    }
    if params.srt.is_some() && params.episode.is_some() {
        return Err(srt_stage::stage_error(
            "ERROR: --srt y --episode son excluyentes.",
        ));
    }

    let (slug, profile) = srt_stage::load_profile(resolver, params.show.as_deref())?;
    let stages = srt_stage::load_stages(&profile)?;

    if let Some(expect) = &params.expect
        && !stages.iter().any(|stage| &stage.name == expect)
    {
        return Err(srt_stage::stage_error(format!(
            "ERROR: La etapa '{expect}' no existe en el perfil '{slug}'.\n       Etapas declaradas: {}",
            stages
                .iter()
                .map(|stage| stage.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }

    let single_file = params.srt.is_some();
    let episode = params.episode.unwrap_or(0);
    let targets: Vec<PathBuf> = match &params.srt {
        Some(srt) => vec![PathBuf::from(srt)],
        None => srt_stage::srt_files_for_episode(&profile, episode)?,
    };

    if targets.is_empty() {
        return Err(if single_file {
            srt_stage::stage_error(format!(
                "ERROR: No se encuentra el SRT {}",
                params.srt.as_deref().unwrap_or("")
            ))
        } else {
            srt_stage::stage_error(format!(
                "ERROR: No se encontró ningún SRT del episodio {episode} según el perfil '{slug}'."
            ))
        });
    }

    let mut results: Vec<(String, String)> = Vec::new();
    for path in &targets {
        if !path.is_file() {
            return Err(srt_stage::stage_error(format!(
                "ERROR: No se encuentra el SRT {}",
                path.display()
            )));
        }
        let stage = srt_stage::classify(path, &stages)?;
        results.push((path.display().to_string(), stage));
    }

    let expect = params.expect.as_deref();
    let found: BTreeSet<&str> = results.iter().map(|(_, stage)| stage.as_str()).collect();
    let ok = expect.is_some_and(|expected| found.contains(expected));
    let code = exit_code(expect, ok, single_file);

    if json_mode {
        let results_json: Vec<Value> = results
            .iter()
            .map(|(path, stage)| json!({ "path": path, "stage": stage }))
            .collect();
        return Ok(Rendered::new(
            json::srt_stage_document(&slug, &results_json, expect, ok),
            String::new(),
        )
        .with_code(code));
    }

    let mut out = String::new();
    for (path, stage) in &results {
        out.push_str(&format!("{path}\t{stage}\n"));
    }

    let Some(expect) = expect else {
        return Ok(Rendered::stdout(out));
    };

    if ok {
        out.push_str("OK\n");
        return Ok(Rendered::stdout(out));
    }

    out.push_str("FALLO\n");
    let err = if single_file {
        let actual = results.first().map_or("", |(_, stage)| stage.as_str());
        format!(
            "ERROR: Se esperaba la etapa '{expect}' pero el SRT es de la etapa '{actual}'.\n       No generes entregables de una etapa que el SRT no es: en un bruto las marcas de tiempo están desplazadas ~26-30 s.\n"
        )
    } else {
        format!("ERROR: Ningún SRT del episodio {episode} es de la etapa '{expect}'.\n")
    };
    Ok(Rendered::new(out, err).with_code(code))
}

fn exit_code(expect: Option<&str>, ok: bool, single_file: bool) -> i32 {
    if expect.is_some() && !ok {
        if single_file { 5 } else { 4 }
    } else {
        0
    }
}
