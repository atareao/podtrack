//! Lector (por `serde`) y **writer canónico a mano** del tracker (D1).
//!
//! La lectura usa `serde_yaml`; la escritura reproduce el formato fijo del oráculo
//! (`save_tracker`): cabecera de comentarios literal, orden de campos top-level y
//! por episodio, `title` entrecomillado, `slug` solo si existe, backup `.yaml.bak`
//! y escritura atómica (temp + rename).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use serde::Deserialize;

use crate::error::PodtrackError;
use crate::model::{Date, Episode, Slug, Status, next_publish_date, sort_episodes};

/// Duración por defecto si el tracker no la declara (el oráculo usa 25).
pub const DEFAULT_EPISODE_DURATION_MINUTES: u16 = 25;

/// El estado del tracker completo (lo que el YAML describe).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tracker {
    /// Temporada actual.
    pub season: u16,
    /// Inicio de temporada (`season_start`), si está declarado.
    pub season_start: Option<String>,
    /// Duración de episodio en minutos, si está declarada.
    pub episode_duration_minutes: Option<u16>,
    /// Episodios, en el orden del fichero (se ordenan al vuelo donde toca).
    pub episodes: Vec<Episode>,
}

#[derive(Deserialize)]
struct RawTracker {
    season: u16,
    #[serde(default)]
    season_start: Option<serde_yaml::Value>,
    #[serde(default)]
    episode_duration_minutes: Option<u16>,
    #[serde(default)]
    episodes: Vec<RawEpisode>,
}

#[derive(Deserialize)]
struct RawEpisode {
    #[serde(default)]
    number: Option<u32>,
    #[serde(default)]
    slug: Option<String>,
    #[serde(default)]
    season: Option<u16>,
    #[serde(default)]
    date: Option<String>,
    title: String,
    status: String,
}

/// Carga el tracker y normaliza fechas (`load_tracker`).
pub fn load_tracker(path: &Path) -> Result<Tracker, PodtrackError> {
    if !path.exists() {
        return Err(PodtrackError::TrackerMissing {
            path: path.to_path_buf(),
        });
    }

    let text = fs::read_to_string(path).map_err(|e| PodtrackError::Io(e.to_string()))?;
    if text.trim().is_empty() {
        return Err(PodtrackError::TrackerNoEpisodes);
    }

    let value: serde_yaml::Value =
        serde_yaml::from_str(&text).map_err(|e| PodtrackError::Yaml {
            reason: e.to_string(),
        })?;

    let mapping = match &value {
        serde_yaml::Value::Mapping(m) if m.contains_key("episodes") => m,
        _ => return Err(PodtrackError::TrackerNoEpisodes),
    };
    let _ = mapping;

    let raw: RawTracker = serde_yaml::from_value(value).map_err(|e| PodtrackError::Yaml {
        reason: e.to_string(),
    })?;

    let season = raw.season;
    let episodes = raw
        .episodes
        .into_iter()
        .map(|ep| {
            let status = Status::parse(&ep.status)?;
            let slug = match ep.slug {
                Some(s) if s.trim().is_empty() => None,
                Some(s) => Some(Slug::parse(s.trim())?),
                None => None,
            };
            Ok(Episode {
                number: ep.number,
                slug,
                season: ep.season.unwrap_or(season),
                date: Date::from_option(ep.date),
                title: ep.title,
                status,
            })
        })
        .collect::<Result<Vec<_>, PodtrackError>>()?;

    Ok(Tracker {
        season,
        season_start: raw.season_start.as_ref().and_then(value_to_string),
        episode_duration_minutes: raw.episode_duration_minutes,
        episodes,
    })
}

fn value_to_string(value: &serde_yaml::Value) -> Option<String> {
    match value {
        serde_yaml::Value::Null => None,
        serde_yaml::Value::String(s) => Some(s.clone()),
        serde_yaml::Value::Number(n) => Some(n.to_string()),
        serde_yaml::Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Guarda el tracker con el formato canónico (`save_tracker`).
///
/// Hace copia `.yaml.bak` del fichero previo (si existe) y escribe de forma
/// atómica. Los avisos de fechas duplicadas van a `err`; la confirmación, a `out`.
pub fn save_tracker(
    path: &Path,
    show_name: &str,
    tracker: &Tracker,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> Result<(), PodtrackError> {
    // Backup: `<tracker>.yaml.bak` (el oráculo usa `with_suffix(".yaml.bak")`,
    // que para un `*.yaml` da exactamente el nombre original + `.bak`).
    if path.exists() {
        let backup = backup_path(path);
        fs::copy(path, &backup).map_err(|e| PodtrackError::Io(e.to_string()))?;
    }

    let mut episodes = tracker.episodes.clone();
    sort_episodes(&mut episodes);
    warn_date_collisions(&episodes, err)?;

    let mut body = String::new();
    body.push_str("# podcast-tracker.yaml\n");
    body.push_str(&format!("# Estado del podcast \"{show_name}\"\n"));
    body.push_str("# Temporadas: empiezan el 1 de septiembre\n");
    body.push_str("# Formato de episodio: XXXX (4 dígitos)\n");
    body.push('\n');
    // Orden de campos top-level fijo (quirk del oráculo: `publish_days` se
    // reescribe siempre como `[Monday, Thursday]`, ignorando el valor leído).
    body.push_str(&format!("season: {}\n", tracker.season));
    body.push_str(&format!(
        "season_start: {}\n",
        tracker.season_start.as_deref().unwrap_or("?")
    ));
    body.push_str(&format!(
        "episode_duration_minutes: {}\n",
        tracker
            .episode_duration_minutes
            .unwrap_or(DEFAULT_EPISODE_DURATION_MINUTES)
    ));
    body.push_str("publish_days: [Monday, Thursday]\n");
    body.push('\n');
    body.push_str("episodes:\n");

    for ep in &episodes {
        body.push_str(&format!(
            "  - number: {}\n",
            ep.number
                .map_or_else(|| "null".to_string(), |n| n.to_string())
        ));
        // `slug` es opcional y aditivo: solo se escribe si el episodio lo trae.
        if let Some(slug) = &ep.slug {
            body.push_str(&format!("    slug: {slug}\n"));
        }
        body.push_str(&format!("    season: {}\n", ep.season));
        body.push_str(&format!("    date: {}\n", ep.date.to_iso()));
        body.push_str(&format!("    title: \"{}\"\n", ep.title));
        body.push_str(&format!("    status: {}\n", ep.status.as_str()));
    }

    write_atomic(path, &body)?;
    writeln!(out, "✓ Tracker actualizado: {}", path.display())
        .map_err(|e| PodtrackError::Io(e.to_string()))?;
    Ok(())
}

fn backup_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".bak");
    PathBuf::from(name)
}

/// Escritura atómica: escribe a un temporal y lo renombra sobre el destino.
fn write_atomic(path: &Path, body: &str) -> Result<(), PodtrackError> {
    let mut tmp = path.as_os_str().to_os_string();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    fs::write(&tmp, body).map_err(|e| PodtrackError::Io(e.to_string()))?;
    fs::rename(&tmp, path).map_err(|e| PodtrackError::Io(e.to_string()))?;
    Ok(())
}

/// Avisa por `err` de las fechas de emisión compartidas (`_warn_date_collisions`).
fn warn_date_collisions(episodes: &[Episode], err: &mut dyn Write) -> Result<(), PodtrackError> {
    // Fecha -> episodios que la comparten, en orden de fecha y de aparición.
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
            writeln!(
                err,
                "  ⚠ Aviso: la fecha {fecha} la comparten {} episodios (#{}).",
                nums.len(),
                nums.join(", ")
            )
            .map_err(|e| PodtrackError::Io(e.to_string()))?;
        }
    }
    Ok(())
}

/// Etiqueta corta de número para textos: `—` cuando es nulo (`_num_label`).
#[must_use]
pub fn num_label(number: Option<u32>) -> String {
    number.map_or_else(|| "—".to_string(), |n| n.to_string())
}

/// Siguiente número público autoritativo: `max(number publicado) + 1` (o 1).
#[must_use]
pub fn next_public_number(tracker: &Tracker) -> u32 {
    let max = tracker
        .episodes
        .iter()
        .filter(|e| e.status == Status::Published)
        .filter_map(|e| e.number)
        .max();
    max.map_or(1, |n| n + 1)
}

/// Siguiente hueco de parrilla: `max(number de todos) + 1` (o 1).
#[must_use]
pub fn next_free_number(tracker: &Tracker) -> u32 {
    tracker
        .episodes
        .iter()
        .filter_map(|e| e.number)
        .max()
        .map_or(1, |n| n + 1)
}

/// Episodio **no publicado** más antiguo por fecha (`_next_to_publish`).
#[must_use]
pub fn next_to_publish(tracker: &Tracker) -> Option<&Episode> {
    let mut pending: Vec<&Episode> = tracker
        .episodes
        .iter()
        .filter(|e| e.status != Status::Published)
        .collect();
    pending.sort_by_key(|e| e.sort_key());
    pending.first().copied()
}

/// Último publicado por fecha de emisión, o `None` (`cmd_status`).
///
/// En empate de fecha conserva el **primero** encontrado, como `max` de Python.
#[must_use]
pub fn last_published(tracker: &Tracker) -> Option<&Episode> {
    let mut best: Option<&Episode> = None;
    for ep in &tracker.episodes {
        if ep.status != Status::Published || ep.date.is_no_date() {
            continue;
        }
        match best {
            None => best = Some(ep),
            Some(current) => {
                if ep.date.to_iso() > current.date.to_iso() {
                    best = Some(ep);
                }
            }
        }
    }
    best
}

/// Fecha de la próxima publicación según el último episodio (`calc_next_date`).
#[must_use]
pub fn calc_next_date(tracker: &Tracker) -> NaiveDate {
    let mut episodes = tracker.episodes.clone();
    sort_episodes(&mut episodes);
    if let Some(last) = episodes.last()
        && let Date::Known(d) = last.date
        && let Some(next_day) = d.succ_opt()
    {
        return next_publish_date(next_day);
    }
    if let Some(season_start) = tracker.season_start.as_deref()
        && let Ok(base) = NaiveDate::parse_from_str(season_start.trim(), "%Y-%m-%d")
    {
        return next_publish_date(base);
    }
    next_publish_date(crate::model::today())
}

#[cfg(test)]
mod tests {
    use super::{Tracker, load_tracker, save_tracker};
    use crate::model::{Date, Episode, Slug, sort_episodes};
    use std::io::Cursor;

    const MINI: &str = "\
# podcast-tracker.yaml
# Estado del podcast \"Show de prueba\"
# Temporadas: empiezan el 1 de septiembre
# Formato de episodio: XXXX (4 dígitos)

season: 8
season_start: 2026-09-01
episode_duration_minutes: 25
publish_days: [Monday, Thursday]

episodes:
  - number: 100
    season: 8
    date: 2026-09-03
    title: \"Primero\"
    status: published
  - number: 101
    season: 8
    date: 2026-09-07
    title: \"Segundo\"
    status: scripted
  - number: 102
    season: 8
    date: 2026-09-10
    title: \"Tercero\"
    status: scripted
";

    fn write(text: &str) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().expect("tmp");
        let path = dir.path().join("podcast-tracker.yaml");
        std::fs::write(&path, text).expect("write");
        (dir, path)
    }

    #[test]
    fn load_normaliza_fechas() {
        let (_d, path) = write(MINI);
        let tracker = load_tracker(&path).expect("load");
        let dates: Vec<String> = tracker.episodes.iter().map(|e| e.date.to_iso()).collect();
        assert_eq!(dates, ["2026-09-03", "2026-09-07", "2026-09-10"]);
    }

    #[test]
    fn load_sin_fichero_falla() {
        let dir = tempfile::tempdir().expect("tmp");
        let err = load_tracker(&dir.path().join("nope.yaml")).expect_err("err");
        assert_eq!(err.exit_code(), 1);
        assert!(err.to_string().contains("No se encuentra"));
    }

    #[test]
    fn load_sin_episodios_falla() {
        let (_d, path) = write("season: 8\n");
        let err = load_tracker(&path).expect_err("err");
        assert!(err.to_string().contains("no tiene episodios"));
    }

    #[test]
    fn save_roundtrip_preserva_episodios() {
        let (_d, path) = write(MINI);
        let tracker = load_tracker(&path).expect("load");
        let mut out = Cursor::new(Vec::new());
        let mut err = Cursor::new(Vec::new());
        save_tracker(&path, "Show de prueba", &tracker, &mut out, &mut err).expect("save");
        let again = load_tracker(&path).expect("reload");
        assert_eq!(again.episodes, tracker.episodes);
    }

    #[test]
    fn save_escribe_cabecera_y_ordena() {
        let (_d, path) = write(MINI);
        let mut tracker = load_tracker(&path).expect("load");
        tracker.episodes.reverse();
        let mut out = Cursor::new(Vec::new());
        let mut err = Cursor::new(Vec::new());
        save_tracker(&path, "Show de prueba", &tracker, &mut out, &mut err).expect("save");
        let text = std::fs::read_to_string(&path).expect("read");
        assert!(text.starts_with("# podcast-tracker.yaml\n"));
        assert!(text.contains("Estado del podcast \"Show de prueba\""));
        let i100 = text.find("number: 100").expect("100");
        let i101 = text.find("number: 101").expect("101");
        let i102 = text.find("number: 102").expect("102");
        assert!(i100 < i101 && i101 < i102);
        // El backup existe.
        assert!(path.with_file_name("podcast-tracker.yaml.bak").exists());
    }

    #[test]
    fn save_ordena_por_fecha_con_null_y_slug() {
        let yaml = "season: 9\nseason_start: 2026-09-03\nepisode_duration_minutes: 25\npublish_days: [Monday, Thursday]\n\nepisodes:\n  - number: 200\n    slug: primer-episodio\n    season: 8\n    date: 2025-09-04\n    title: \"Primero\"\n    status: published\n  - number: null\n    slug: sin-numero\n    season: 9\n    date: 2026-09-07\n    title: \"Sin número\"\n    status: scripted\n  - number: 202\n    season: 9\n    date: 2026-09-10\n    title: \"Tercero\"\n    status: recorded\n";
        let (_d, path) = write(yaml);
        let mut tracker = load_tracker(&path).expect("load");
        tracker.episodes.reverse();
        let mut out = Cursor::new(Vec::new());
        let mut err = Cursor::new(Vec::new());
        save_tracker(&path, "Show de prueba", &tracker, &mut out, &mut err).expect("save");
        let text = std::fs::read_to_string(&path).expect("read");
        assert!(text.contains("number: null"));
        assert!(text.contains("slug: sin-numero"));
        let i200 = text.find("number: 200").expect("200");
        let inull = text.find("number: null").expect("null");
        let i202 = text.find("number: 202").expect("202");
        assert!(i200 < inull && inull < i202);
    }

    #[test]
    fn save_avisa_fechas_duplicadas() {
        let (_d, path) = write(MINI);
        let mut tracker = load_tracker(&path).expect("load");
        let first_date = tracker.episodes[0].date.to_iso();
        tracker.episodes[1].date = Date::parse(&first_date);
        let mut out = Cursor::new(Vec::new());
        let mut err = Cursor::new(Vec::new());
        save_tracker(&path, "Show de prueba", &tracker, &mut out, &mut err).expect("save");
        let err_text = String::from_utf8(err.into_inner()).expect("utf8");
        assert!(err_text.contains("Aviso"));
    }

    #[test]
    fn sort_por_fecha_con_number_desempate() {
        let mut eps = vec![
            Episode {
                number: Some(2),
                slug: Some(Slug::parse("b").expect("slug")),
                season: 1,
                date: Date::parse("2026-09-07"),
                title: "b".into(),
                status: crate::model::Status::Scripted,
            },
            Episode {
                number: Some(1),
                slug: None,
                season: 1,
                date: Date::parse("2026-09-07"),
                title: "a".into(),
                status: crate::model::Status::Scripted,
            },
        ];
        sort_episodes(&mut eps);
        assert_eq!(eps[0].number, Some(1));
        let _ = Tracker {
            season: 1,
            season_start: None,
            episode_duration_minutes: None,
            episodes: eps,
        };
    }
}
