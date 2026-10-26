//! Clasificación de la **etapa de transcripción** de un SRT (`raw`/`final`).
//!
//! El módulo resultó de portar `srt-stage.py`: la etapa se decide **por
//! contenido** (nunca por el nombre del fichero). El `final` se reconoce por
//! **dos señales del intro combinadas con un AND** (texto + bloque), con el
//! marcador literal como excepción que basta por sí solo.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;
use serde_yaml::Value;
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

use crate::error::PodtrackError;
use crate::profile::ShowResolver;

/// Etapas válidas del contrato (`--expect`).
pub const VALID_STAGES: [&str; 2] = ["raw", "final"];
/// Valores válidos de `detect`.
pub const VALID_DETECT: [&str; 3] = ["any", "not_intro", "starts_with_intro"];

/// Cómo reconocer una etapa en el SRT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Detect {
    /// Todo SRT del show es de esa etapa (solo válido con una única etapa).
    Any,
    /// El primer cue con texto real **no** es el intro.
    NotIntro,
    /// El primer cue con texto real **es** el intro (dos señales, AND).
    StartsWithIntro,
}

/// Una etapa del perfil con las señales que la reconocen.
#[derive(Debug, Clone)]
pub struct Stage {
    /// Nombre de la etapa (`raw`, `final`, …).
    pub name: String,
    /// Etiqueta legible declarada por el perfil (informativa).
    pub label: String,
    /// Cómo reconocerla.
    pub detect: Detect,
    /// Entregables habilitados por la etapa (informativos).
    pub deliverables: Vec<String>,
    /// Nombre canónico del fichero (informativo).
    pub canonical_name: Option<String>,
    /// Regex del arranque del intro (forma normalizada).
    pub intro_pattern: Option<String>,
    /// Caracteres de anclaje de la señal de texto.
    pub intro_anchor_chars: i64,
    /// Marcador literal que basta por sí solo.
    pub intro_marker: Option<String>,
    /// Duración mínima (s) del primer cue para la señal de bloque.
    pub intro_block_min_seconds: Option<f64>,
    /// Inicio máximo (s) tolerado del primer cue.
    pub intro_block_max_start: f64,
    /// Regex compilada de `intro_pattern`.
    pub intro_re: Option<Regex>,
}

static BLANK_LINE_RE: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(r"\n[ \t]*\n").ok());
static TIMESTAMP_RE: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(
        r"(\d{1,3}):(\d{2}):(\d{2})[,.](\d{1,3})\s*-->\s*(\d{1,3}):(\d{2}):(\d{2})[,.](\d{1,3})",
    )
    .ok()
});
static WHITESPACE_RE: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(r"\s+").ok());

/// Error de `srt-stage` con el mensaje literal del oráculo (exit 2).
#[must_use]
pub fn stage_error(message: impl Into<String>) -> PodtrackError {
    PodtrackError::SrtStageError {
        message: message.into(),
    }
}

// ─── Lectura de SRT ─────────────────────────────────────────────────

/// Lee un SRT como texto UTF-8 con BOM tolerado y bytes inválidos sustituidos
/// (equivalente a `read_text(encoding="utf-8-sig", errors="replace")`).
pub fn read_text(path: &Path) -> Result<String, PodtrackError> {
    let bytes = fs::read(path).map_err(|e| PodtrackError::SrtRead {
        path: path.display().to_string(),
        reason: e.to_string(),
    })?;
    let text = String::from_utf8_lossy(&bytes);
    Ok(text.trim_start_matches('\u{feff}').to_string())
}

fn blocks(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    match BLANK_LINE_RE.as_ref() {
        Some(re) => re.split(trimmed).map(str::to_string).collect(),
        None => vec![trimmed.to_string()],
    }
}

/// Texto de los primeros `count` cues **con texto real** (`first_cue_texts`).
///
/// Los cues iniciales vacíos o de solo puntuación (`-`, `...`, `—`) se ignoran.
pub fn first_cue_texts(path: &Path, count: usize) -> Result<Vec<String>, PodtrackError> {
    let raw = read_text(path)?;
    let mut texts = Vec::new();
    for block in blocks(&raw) {
        let lines: Vec<&str> = block.lines().collect();
        let ts_index = lines.iter().position(|line| line.contains("-->"));
        let text_lines: &[&str] = match ts_index {
            Some(index) => lines.get(index + 1..).unwrap_or(&[]),
            None => lines.get(1..).unwrap_or(&[]),
        };
        let text = text_lines.join(" ");
        if text.chars().any(char::is_alphanumeric) {
            texts.push(text);
            if texts.len() >= count {
                break;
            }
        }
    }
    Ok(texts)
}

/// Texto del primer cue con texto real (o `""`).
pub fn first_cue_text(path: &Path) -> Result<String, PodtrackError> {
    Ok(first_cue_texts(path, 1)?
        .first()
        .cloned()
        .unwrap_or_default())
}

fn ts_seconds(hours: &str, minutes: &str, seconds: &str, millis: &str) -> f64 {
    let hours = hours.parse::<i64>().unwrap_or(0);
    let minutes = minutes.parse::<i64>().unwrap_or(0);
    let seconds = seconds.parse::<i64>().unwrap_or(0);
    let mut padded = millis.to_string();
    while padded.len() < 3 {
        padded.push('0');
    }
    let millis = padded.parse::<i64>().unwrap_or(0);
    (hours * 3600 + minutes * 60 + seconds) as f64 + millis as f64 / 1000.0
}

/// Inicio y fin (segundos) del **primer cue** del fichero (`first_cue_span`).
pub fn first_cue_span(path: &Path) -> Result<Option<(f64, f64)>, PodtrackError> {
    let raw = read_text(path)?;
    for block in blocks(&raw) {
        for line in block.lines() {
            let Some(caps) = TIMESTAMP_RE.as_ref().and_then(|re| re.captures(line)) else {
                continue;
            };
            let get = |index: usize| caps.get(index).map_or("", |m| m.as_str());
            return Ok(Some((
                ts_seconds(get(1), get(2), get(3), get(4)),
                ts_seconds(get(5), get(6), get(7), get(8)),
            )));
        }
    }
    Ok(None)
}

/// Minúsculas, sin acentos, sin BOM y con los espacios colapsados (`normalize`).
#[must_use]
pub fn normalize(text: &str) -> String {
    let without_bom: String = text.chars().filter(|c| *c != '\u{feff}').collect();
    let decomposed: String = without_bom
        .nfkd()
        .filter(|c| !is_combining_mark(*c))
        .collect();
    let lowered = decomposed.to_lowercase();
    let collapsed = match WHITESPACE_RE.as_ref() {
        Some(re) => re.replace_all(&lowered, " ").into_owned(),
        None => lowered,
    };
    collapsed.trim().to_string()
}

/// ¿El fichero tiene al menos un cue con línea de tiempos (`srt_has_cues`)?
pub fn srt_has_cues(path: &Path) -> Result<bool, PodtrackError> {
    let raw = read_text(path)?;
    Ok(blocks(&raw)
        .iter()
        .any(|block| block.lines().any(|line| line.contains("-->"))))
}

// ─── Regla del intro ────────────────────────────────────────────────

/// Etapa que describe el intro (`detect: starts_with_intro`), si la hay.
#[must_use]
pub fn intro_config(stages: &[Stage]) -> Option<&Stage> {
    stages
        .iter()
        .find(|stage| stage.detect == Detect::StartsWithIntro)
}

/// ¿El intro ocupa el arranque del fichero? (segunda señal, `intro_block_ok`).
#[must_use]
pub fn intro_block_ok(span: Option<(f64, f64)>, intro: Option<&Stage>) -> bool {
    let Some(intro) = intro else {
        return false;
    };
    let Some(block_min) = intro.intro_block_min_seconds else {
        return true;
    };
    let Some((start, end)) = span else {
        return false;
    };
    start <= intro.intro_block_max_start && end >= block_min
}

/// ¿El arranque del episodio es el intro locutado (o el marcador literal)?
#[must_use]
pub fn looks_like_intro(
    cue: &str,
    head: &str,
    span: Option<(f64, f64)>,
    intro: Option<&Stage>,
) -> bool {
    let Some(intro) = intro else {
        return false;
    };
    if let Some(marker) = &intro.intro_marker
        && !marker.is_empty()
        && cue == normalize(marker)
    {
        return true;
    }
    let Some(re) = &intro.intro_re else {
        return false;
    };
    let Some(found) = re.find(head) else {
        return false;
    };
    // El oráculo compara el índice en **caracteres** (`match.start()` sobre un
    // `str`), no en bytes: `¿`, `—`, `«`, `»`, `…` antes de la coincidencia no
    // deben consumir el anclaje de más.
    let char_index = head[..found.start()].chars().count();
    if char_index as i64 >= intro.intro_anchor_chars {
        return false;
    }
    intro_block_ok(span, Some(intro))
}

/// Clasifica un SRT en una de las etapas del perfil (`classify`).
pub fn classify(path: &Path, stages: &[Stage]) -> Result<String, PodtrackError> {
    if !srt_has_cues(path)? {
        return Err(PodtrackError::SrtNoCues {
            path: path.display().to_string(),
        });
    }

    if stages.len() == 1 && stages[0].detect == Detect::Any {
        return Ok(stages[0].name.clone());
    }

    let intro = intro_config(stages);
    let cues = first_cue_texts(path, 2)?;
    let cue = cues.first().map_or_else(String::new, |c| normalize(c));
    let head = normalize(&cues.join(" "));
    let span = first_cue_span(path)?;
    let is_intro = looks_like_intro(&cue, &head, span, intro);

    for stage in stages {
        match stage.detect {
            Detect::Any => return Ok(stage.name.clone()),
            Detect::StartsWithIntro if is_intro => return Ok(stage.name.clone()),
            Detect::NotIntro if !is_intro => return Ok(stage.name.clone()),
            _ => {}
        }
    }

    // Un SRT que no encaja en ninguna etapa se trata como "no intro" (bruto).
    Ok(stages
        .iter()
        .find(|stage| stage.name == "raw")
        .or_else(|| stages.last())
        .map(|stage| stage.name.clone())
        .unwrap_or_default())
}

// ─── Navegación de YAML ─────────────────────────────────────────────

fn map_get<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    match value {
        Value::Mapping(map) => map
            .iter()
            .find(|(k, _)| k.as_str() == Some(key))
            .map(|(_, v)| v),
        _ => None,
    }
}

fn mget<'a>(map: &'a serde_yaml::Mapping, key: &str) -> Option<&'a Value> {
    map.iter()
        .find(|(k, _)| k.as_str() == Some(key))
        .map(|(_, v)| v)
}

fn is_falsy(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::Bool(b) => !*b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f == 0.0),
        Value::String(s) => s.is_empty(),
        Value::Sequence(s) => s.is_empty(),
        Value::Mapping(m) => m.is_empty(),
        _ => false,
    }
}

fn value_as_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Verdad booleana de Python para un `Option<String>`: `None` y `Some("")` son
/// falsos (una cadena vacía cuenta como ausente).
fn string_is_falsy(value: &Option<String>) -> bool {
    value.as_ref().is_none_or(String::is_empty)
}

/// Representación `repr()` de Python para los mensajes de error.
fn py_repr(value: &Value) -> String {
    match value {
        Value::Null => "None".to_string(),
        Value::Bool(b) => if *b { "True" } else { "False" }.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => format!("'{s}'"),
        other => {
            serde_yaml::to_string(other).map_or_else(|_| String::new(), |s| s.trim().to_string())
        }
    }
}

fn string_list(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Sequence(seq)) => seq.iter().filter_map(value_as_string).collect(),
        _ => Vec::new(),
    }
}

// ─── Carga del perfil ───────────────────────────────────────────────

/// Localiza y parsea el perfil `shows/<slug>.yaml` **sin exigir `tracker`**.
///
/// Devuelve `(slug, perfil)`.
pub fn load_profile(
    resolver: &ShowResolver,
    show: Option<&str>,
) -> Result<(String, Value), PodtrackError> {
    let slug = show
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| resolver.default_slug.clone());

    let Some(path) = resolver.find_profile(&slug) else {
        let expected = resolver.first_candidate(&slug).display().to_string();
        let available = resolver.available_profiles();
        return Err(PodtrackError::SrtProfileNotFound {
            slug,
            expected,
            available,
        });
    };

    let text = fs::read_to_string(&path).map_err(|e| PodtrackError::SrtProfileUnreadable {
        path: path.display().to_string(),
        reason: e.to_string(),
    })?;
    let value: Value =
        serde_yaml::from_str(&text).map_err(|e| PodtrackError::SrtProfileUnreadable {
            path: path.display().to_string(),
            reason: e.to_string(),
        })?;
    // `yaml.safe_load(f) or {}`.
    let profile = match value {
        Value::Null => Value::Mapping(serde_yaml::Mapping::new()),
        other => other,
    };
    Ok((slug, profile))
}

/// Carga las etapas del perfil en el orden de `srt_stages.order` (`load_stages`).
pub fn load_stages(profile: &Value) -> Result<Vec<Stage>, PodtrackError> {
    let episodes = map_get(profile, "episodes").cloned().unwrap_or(Value::Null);
    let stages_value = map_get(&episodes, "srt_stages")
        .cloned()
        .unwrap_or(Value::Null);

    let Value::Mapping(stages) = &stages_value else {
        return Err(no_stages_error());
    };
    if stages.is_empty() {
        return Err(no_stages_error());
    }

    let order: Vec<String> = match mget(stages, "order") {
        Some(Value::Sequence(seq)) if !seq.is_empty() => {
            seq.iter().filter_map(value_as_string).collect()
        }
        _ => stages
            .iter()
            .filter_map(|(k, _)| k.as_str().map(str::to_string))
            .filter(|k| k != "order")
            .collect(),
    };

    let mut result = Vec::new();
    for name in &order {
        let Some(Value::Mapping(spec)) = mget(stages, name) else {
            return Err(stage_error(format!(
                "ERROR: La etapa '{name}' no está definida en 'srt_stages'."
            )));
        };

        let detect_value = mget(spec, "detect")
            .cloned()
            .unwrap_or(Value::String("any".into()));
        let detect = match detect_value.as_str() {
            Some("any") => Detect::Any,
            Some("not_intro") => Detect::NotIntro,
            Some("starts_with_intro") => Detect::StartsWithIntro,
            _ => {
                return Err(stage_error(format!(
                    "ERROR: La etapa '{name}' declara un 'detect' desconocido ({}). Valores válidos: {}.",
                    py_repr(&detect_value),
                    VALID_DETECT.join(", ")
                )));
            }
        };

        let mut stage = Stage {
            name: name.clone(),
            label: value_as_string(mget(spec, "label").unwrap_or(&Value::Null))
                .unwrap_or_else(|| name.clone()),
            detect,
            deliverables: string_list(mget(spec, "deliverables")),
            canonical_name: mget(spec, "canonical_name").and_then(value_as_string),
            intro_pattern: mget(spec, "intro_pattern").and_then(value_as_string),
            intro_anchor_chars: 0,
            intro_marker: mget(spec, "intro_marker").and_then(value_as_string),
            intro_block_min_seconds: None,
            intro_block_max_start: 1.0,
            intro_re: None,
        };

        if detect == Detect::StartsWithIntro {
            validate_intro_stage(name, spec, &mut stage)?;
        }

        result.push(stage);
    }

    if result.len() > 1 && result.iter().any(|s| s.detect == Detect::Any) {
        return Err(stage_error(
            "ERROR: 'detect: any' solo es válido cuando 'order' tiene una única etapa. Con varias etapas, usa 'not_intro' o 'starts_with_intro'.",
        ));
    }

    Ok(result)
}

/// Valida y completa las señales del intro de una etapa `starts_with_intro`.
fn validate_intro_stage(
    name: &str,
    spec: &serde_yaml::Mapping,
    stage: &mut Stage,
) -> Result<(), PodtrackError> {
    // El oráculo usa verdad booleana de Python: una cadena **vacía** cuenta como
    // ausente, igual que faltar la clave.
    if string_is_falsy(&stage.intro_pattern) && string_is_falsy(&stage.intro_marker) {
        return Err(stage_error(format!(
            "ERROR: La etapa '{name}' usa detect: starts_with_intro pero no declara 'intro_pattern' ni 'intro_marker'."
        )));
    }

    // `intro_anchor_chars` (default 0 → inválido: debe ser > 0).
    let anchor_value = mget(spec, "intro_anchor_chars").cloned();
    let anchor = match &anchor_value {
        None => Some(0),
        Some(Value::Number(n)) if n.is_i64() || n.is_u64() => n.as_i64(),
        _ => None,
    };
    match anchor.filter(|a| *a > 0) {
        Some(anchor) => stage.intro_anchor_chars = anchor,
        None => {
            let repr = anchor_value
                .as_ref()
                .map_or_else(|| "0".to_string(), py_repr);
            return Err(stage_error(format!(
                "ERROR: La etapa '{name}' declara 'intro_anchor_chars' inválido ({repr}); debe ser un entero > 0."
            )));
        }
    }

    // `intro_block_min_seconds`: ausente o null → señal de bloque no aplicable.
    match mget(spec, "intro_block_min_seconds") {
        None | Some(Value::Null) => {}
        Some(value) => match number_value(value).filter(|f| *f > 0.0) {
            Some(min) => stage.intro_block_min_seconds = Some(min),
            None => {
                return Err(stage_error(format!(
                    "ERROR: La etapa '{name}' declara 'intro_block_min_seconds' inválido ({}); debe ser un número > 0.",
                    py_repr(value)
                )));
            }
        },
    }

    // `intro_block_max_start` (default 1.0; un null presente es inválido).
    match mget(spec, "intro_block_max_start") {
        None => stage.intro_block_max_start = 1.0,
        Some(value) => match number_value(value).filter(|f| *f >= 0.0) {
            Some(max) => stage.intro_block_max_start = max,
            None => {
                return Err(stage_error(format!(
                    "ERROR: La etapa '{name}' declara 'intro_block_max_start' inválido ({}); debe ser un número >= 0.",
                    py_repr(value)
                )));
            }
        },
    }

    if let Some(pattern) = &stage.intro_pattern
        && !pattern.is_empty()
    {
        match Regex::new(pattern) {
            Ok(re) => stage.intro_re = Some(re),
            Err(e) => {
                return Err(stage_error(format!(
                    "ERROR: 'intro_pattern' de la etapa '{name}' no es una regex válida: {e}"
                )));
            }
        }
    }
    Ok(())
}

fn number_value(value: &Value) -> Option<f64> {
    match value {
        Value::Number(n) => n.as_f64(),
        _ => None,
    }
}

fn no_stages_error() -> PodtrackError {
    stage_error(
        "ERROR: El perfil no declara 'episodes.srt_stages'.\n       Sin etapas no se puede clasificar ningún SRT.",
    )
}

// ─── Localización de SRT ────────────────────────────────────────────

/// Localiza los SRT de un episodio con `episodes.srt_glob` (`srt_files_for_episode`).
pub fn srt_files_for_episode(profile: &Value, episode: i64) -> Result<Vec<PathBuf>, PodtrackError> {
    let episodes = map_get(profile, "episodes").cloned().unwrap_or(Value::Null);
    let root = map_get(&episodes, "root").cloned().unwrap_or(Value::Null);
    let srt_glob = map_get(&episodes, "srt_glob")
        .cloned()
        .unwrap_or(Value::Null);

    if is_falsy(&root) || is_falsy(&srt_glob) {
        return Err(stage_error(
            "ERROR: El perfil no declara episodes.root y episodes.srt_glob.",
        ));
    }

    let pattern = render_glob(&value_as_string(&srt_glob).unwrap_or_default(), episode)?;
    let root_str = value_as_string(&root).unwrap_or_default();
    let full = Path::new(&root_str).join(&pattern);

    let mut files: Vec<PathBuf> = Vec::new();
    if let Ok(paths) = glob::glob(&full.to_string_lossy()) {
        for entry in paths.flatten() {
            files.push(entry);
        }
    }
    files.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
    Ok(files)
}

/// Compone `srt_glob` con el número del episodio (`str(srt_glob).format(n=NN)`).
fn render_glob(pattern: &str, episode: i64) -> Result<String, PodtrackError> {
    let mut out = String::new();
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '{' => {
                if chars.peek() == Some(&'{') {
                    chars.next();
                    out.push('{');
                    continue;
                }
                let mut spec = String::new();
                for c2 in chars.by_ref() {
                    if c2 == '}' {
                        break;
                    }
                    spec.push(c2);
                }
                out.push_str(&render_field(&spec, episode)?);
            }
            '}' => {
                if chars.peek() == Some(&'}') {
                    chars.next();
                }
                out.push('}');
            }
            _ => out.push(c),
        }
    }
    Ok(out)
}

fn render_field(spec: &str, episode: i64) -> Result<String, PodtrackError> {
    let (head, format_spec) = spec.split_once(':').map_or((spec, ""), |(h, f)| (h, f));
    let (field, conversion) = head.split_once('!').map_or((head, ""), |(f, c)| (f, c));

    if field != "n" {
        return Err(glob_marker_error(format!("'{field}'")));
    }
    match conversion {
        "" => {}
        "r" => return Ok(episode.to_string()),
        other => return Err(glob_marker_error(format!("conversion '{other}'"))),
    }
    if format_spec.is_empty() {
        return Ok(episode.to_string());
    }

    let (zero, rest) = match format_spec.strip_prefix('0') {
        Some(rest) => (true, rest),
        None => (false, format_spec),
    };
    let Some(width_text) = rest.strip_suffix('d') else {
        return Err(glob_marker_error(format!(
            "format specifier '{format_spec}'"
        )));
    };
    if !width_text.chars().all(|c| c.is_ascii_digit()) {
        return Err(glob_marker_error(format!(
            "format specifier '{format_spec}'"
        )));
    }
    let width: usize = width_text.parse().unwrap_or(0);
    if zero {
        Ok(format!("{episode:0>width$}"))
    } else if width == 0 {
        Ok(episode.to_string())
    } else {
        Ok(format!("{episode:>width$}"))
    }
}

fn glob_marker_error(detail: String) -> PodtrackError {
    stage_error(format!(
        "ERROR: episodes.srt_glob no admite el marcador del número: {detail}"
    ))
}
