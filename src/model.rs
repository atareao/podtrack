//! Modelo tipado del dominio: episodio, estado, slug y fecha.
//!
//! El invariante estructural clave —**el número no ordena la parrilla**— se
//! materializa en [`Episode::sort_key`]: la fecha ordena, el número solo desempata
//! y los episodios sin fecha van al final.

use std::fmt;

use chrono::{Datelike, Local, NaiveDate, Timelike, Weekday};

use crate::error::PodtrackError;

/// Estados válidos, en orden alfabético (el que usa el oráculo en sus mensajes).
pub const VALID_STATUSES: [&str; 4] = ["pending", "published", "recorded", "scripted"];

/// Días de publicación del show: lunes (0) y jueves (3), como el oráculo.
///
/// **Nota de divergencia (fase futura):** el oráculo lleva estos días
/// *hardcodeados* (`PUBLISH_DAYS = (0, 3)`); el esquema del perfil declara
/// `publish_days`, pero el Python no lo consulta para calcular fechas. Aquí se
/// reproduce ese comportamiento para mantener la paridad byte a byte.
pub const PUBLISH_DAYS: [u32; 2] = [0, 3];

/// Estado de un episodio: enum cerrado de 4 variantes (spec `tracker-store`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// Pendiente (⬜).
    Pending,
    /// Guionizado (📝).
    Scripted,
    /// Grabado (🎙).
    Recorded,
    /// Publicado (✅).
    Published,
}

impl Status {
    /// Convierte desde la forma textual del tracker. Un valor fuera del conjunto
    /// cerrado es un **error de lectura** (spec `tracker-store`), no un valor
    /// tolerado.
    pub fn parse(value: &str) -> Result<Self, PodtrackError> {
        match value {
            "pending" => Ok(Self::Pending),
            "scripted" => Ok(Self::Scripted),
            "recorded" => Ok(Self::Recorded),
            "published" => Ok(Self::Published),
            other => Err(PodtrackError::InvalidStatus {
                value: other.to_string(),
                valid: VALID_STATUSES.join(", "),
            }),
        }
    }

    /// Forma textual canónica.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Scripted => "scripted",
            Self::Recorded => "recorded",
            Self::Published => "published",
        }
    }

    /// Icono del oráculo (`icon_for`).
    #[must_use]
    pub fn icon(self) -> &'static str {
        match self {
            Self::Published => "✅",
            Self::Recorded => "🎙",
            Self::Scripted => "📝",
            Self::Pending => "⬜",
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// `slug` kebab-case validado (newtype). Es la identidad estable del episodio.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Slug(String);

impl Slug {
    /// Valida y construye un slug. Un slug mal formado es un error de lectura.
    pub fn parse(value: &str) -> Result<Self, PodtrackError> {
        if is_kebab_case(value) {
            Ok(Self(value.to_string()))
        } else {
            Err(PodtrackError::InvalidSlug {
                value: value.to_string(),
            })
        }
    }

    /// Vista prestada de la cadena.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Slug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn is_kebab_case(value: &str) -> bool {
    !value.is_empty()
        && value.split('-').all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
}

/// Fecha tipada con estado «sin fecha» (`TBD`/vacía) que ordena al final.
///
/// Se conserva una variante [`Date::Raw`] para reproducir la **tolerancia** del
/// oráculo (que nunca valida la fecha al leer): una cadena que no parsea se
/// preserva literalmente, igual que `_normalize_date` en el Python.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Date {
    /// Fecha ISO `YYYY-MM-DD` válida.
    Known(NaiveDate),
    /// Sin fecha: `TBD`, vacía o ausente. Ordena al final.
    Tbd,
    /// Cadena no parseable, preservada tal cual.
    Raw(String),
}

impl Date {
    /// Interpreta una cadena del tracker.
    #[must_use]
    pub fn parse(value: &str) -> Self {
        let trimmed = value.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("TBD") {
            Self::Tbd
        } else {
            NaiveDate::parse_from_str(trimmed, "%Y-%m-%d")
                .map_or_else(|_| Self::Raw(trimmed.to_string()), Self::Known)
        }
    }

    /// Interpreta un campo opcional (`None` → sin fecha).
    #[must_use]
    pub fn from_option(value: Option<String>) -> Self {
        value.map_or(Self::Tbd, |s| Self::parse(&s))
    }

    /// Representación textual canónica (`YYYY-MM-DD`, `TBD` o la cadena cruda).
    #[must_use]
    pub fn to_iso(&self) -> String {
        match self {
            Self::Known(d) => d.format("%Y-%m-%d").to_string(),
            Self::Tbd => "TBD".to_string(),
            Self::Raw(s) => s.clone(),
        }
    }

    /// ¿Es un estado «sin fecha» (ordena al final)?
    #[must_use]
    pub fn is_no_date(&self) -> bool {
        match self {
            Self::Known(_) => false,
            Self::Tbd => true,
            Self::Raw(s) => s.is_empty(),
        }
    }

    /// Fecha real si la hay.
    #[must_use]
    pub fn known(&self) -> Option<NaiveDate> {
        match self {
            Self::Known(d) => Some(*d),
            _ => None,
        }
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_iso())
    }
}

/// Un episodio de la parrilla (spec `tracker-store`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Episode {
    /// Número público. `None` mientras no está publicado; provisional si no lo está.
    pub number: Option<u32>,
    /// Identidad estable (opcional durante la migración).
    pub slug: Option<Slug>,
    /// Temporada (en numeración estacional).
    pub season: u16,
    /// Fecha de emisión (o estado sin fecha).
    pub date: Date,
    /// Título libre.
    pub title: String,
    /// Estado del ciclo de vida.
    pub status: Status,
}

impl Episode {
    /// Clave de orden del oráculo (`_episode_sort_key`): `(sin_fecha, fecha, número)`,
    /// donde `sin_fecha` manda (los sin fecha al final) y `number: null` desempata
    /// por detrás de cualquier número real.
    #[must_use]
    pub fn sort_key(&self) -> (u8, String, (u8, u32)) {
        let fecha = self.date.to_iso();
        let sin_fecha = u8::from(self.date.is_no_date());
        let number_key = match self.number {
            None => (1u8, 0u32),
            Some(n) => (0u8, n),
        };
        (sin_fecha, fecha, number_key)
    }
}

/// Ordena in-place por la clave del dominio (fecha ascendente, número desempate).
pub fn sort_episodes(episodes: &mut [Episode]) {
    episodes.sort_by_key(|e| e.sort_key());
}

/// Nombres de día en español, indexados por [`Weekday`].
#[must_use]
pub fn weekday_es(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Mon => "lunes",
        Weekday::Tue => "martes",
        Weekday::Wed => "miércoles",
        Weekday::Thu => "jueves",
        Weekday::Fri => "viernes",
        Weekday::Sat => "sábado",
        Weekday::Sun => "domingo",
    }
}

/// Próximo día de publicación (lunes o jueves) desde `from_date` (`next_publish_date`).
///
/// Reproduce el matiz horario del oráculo: si `from_date` ya es publicable, solo se
/// devuelve tal cual cuando aún no ha pasado el mediodía.
#[must_use]
pub fn next_publish_date(from_date: NaiveDate) -> NaiveDate {
    let mut day = from_date;
    for _ in 0..14 {
        if PUBLISH_DAYS.contains(&day.weekday().num_days_from_monday()) {
            let before_noon = Local::now().hour() < 12;
            if day > from_date || (day == from_date && before_noon) {
                return day;
            }
        }
        day = day.succ_opt().unwrap_or(day);
    }
    day
}

/// Hoy (fecha local).
#[must_use]
pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

/// Parsea una fecha `YYYY-MM-DD` con error claro (`_parse_date` del oráculo).
pub fn parse_iso(value: &str) -> Result<NaiveDate, PodtrackError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| PodtrackError::InvalidDate {
        value: value.to_string(),
    })
}

/// Siguiente día publicable **estrictamente** posterior a `day` (`_next_slot`).
#[must_use]
pub fn next_slot(day: NaiveDate) -> NaiveDate {
    let mut candidate = day.succ_opt().unwrap_or(day);
    while !PUBLISH_DAYS.contains(&candidate.weekday().num_days_from_monday()) {
        match candidate.succ_opt() {
            Some(next) => candidate = next,
            None => break,
        }
    }
    candidate
}

/// Formatea una fecha con su día de la semana (`fmt_date`). `TBD` → `TBD`.
#[must_use]
pub fn fmt_date(date: &Date) -> String {
    match date {
        Date::Tbd => "TBD".to_string(),
        Date::Known(d) => format!("{} ({})", d.format("%Y-%m-%d"), weekday_es(d.weekday())),
        Date::Raw(s) => s.clone(),
    }
}

/// Normaliza un texto a kebab-case ASCII (`slugify`).
///
/// NFKD → descarta lo no ASCII → minúsculas → colapsa lo no alfanumérico a un
/// único guion → recorta guiones extremos. Un texto sin caracteres utilizables
/// devuelve cadena vacía.
#[must_use]
pub fn slugify(texto: &str) -> String {
    use unicode_normalization::UnicodeNormalization;

    let ascii: String = texto.nfkd().filter(char::is_ascii).collect();
    let lowered = ascii.to_ascii_lowercase();

    let mut out = String::with_capacity(lowered.len());
    let mut previous_dash = false;
    for c in lowered.chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            out.push(c);
            previous_dash = false;
        } else if !previous_dash {
            out.push('-');
            previous_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

/// Slug libre a partir de `base`, sufijando `-2`, `-3`, … si ya está en `used`
/// (`_unique_slug` del oráculo). El orden de `used` no importa.
#[must_use]
pub fn unique_slug<'a>(used: impl IntoIterator<Item = &'a str>, base: &str) -> String {
    let used: Vec<&str> = used.into_iter().collect();
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

#[cfg(test)]
mod tests {
    use super::{Date, Episode, Slug, Status, slugify, sort_episodes, unique_slug};

    fn ep(number: Option<u32>, date: &str) -> Episode {
        Episode {
            number,
            slug: None,
            season: 9,
            date: Date::parse(date),
            title: format!("ep {number:?}"),
            status: Status::Scripted,
        }
    }

    #[test]
    fn el_numero_no_ordena() {
        let mut eps = vec![ep(Some(900), "2026-10-26"), ep(Some(843), "2026-10-29")];
        sort_episodes(&mut eps);
        assert_eq!(eps[0].number, Some(900));
        assert_eq!(eps[1].number, Some(843));
    }

    #[test]
    fn sin_fecha_va_al_final() {
        let mut eps = vec![ep(None, "TBD"), ep(Some(1), "2026-10-26")];
        sort_episodes(&mut eps);
        assert_eq!(eps[0].number, Some(1));
        assert!(eps[1].date.is_no_date());
    }

    #[test]
    fn slug_valida_kebab_case() {
        assert!(Slug::parse("la-memoria-de-valet").is_ok());
        assert!(Slug::parse("MalFormado").is_err());
        assert!(Slug::parse("-x").is_err());
    }

    #[test]
    fn slugify_normaliza_acentos_espacios_y_simbolos() {
        assert_eq!(slugify("Cómo hacer un Podcast"), "como-hacer-un-podcast");
        assert_eq!(slugify("  Espacios   de más  "), "espacios-de-mas");
        assert_eq!(slugify("Ñandú & Cía: ¡vamos!"), "nandu-cia-vamos");
        assert_eq!(slugify("A/B__C--D"), "a-b-c-d");
        assert_eq!(slugify("Versión 2.0 (final)"), "version-2-0-final");
    }

    #[test]
    fn unique_slug_sufija_solo_si_colisiona() {
        let used = ["a", "b-2", "c"];
        assert_eq!(unique_slug(used, "b"), "b"); // 'b' no está en used
        assert_eq!(unique_slug(used, "a"), "a-2"); // 'a' ocupado → -2
        assert_eq!(unique_slug(used, "b-2"), "b-2-2"); // se mira el candidato, no el prefijo
        assert_eq!(unique_slug(["x"], "x"), "x-2");
        assert_eq!(unique_slug(["x", "x-2"], "x"), "x-3");
        assert_eq!(unique_slug(std::iter::empty::<&str>(), "libre"), "libre");
    }

    #[test]
    fn slugify_casos_vacios_y_ruido() {
        assert_eq!(slugify(""), "");
        assert_eq!(slugify("   "), "");
        assert_eq!(slugify("!!!"), "");
        assert_eq!(slugify("MiXeD CaSe"), "mixed-case");
    }
}
