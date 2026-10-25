//! Errores tipados del dominio (D4).
//!
//! Los mensajes reproducen los del oráculo (`podcast-tracker.py`) **incluido el
//! prefijo literal `ERROR: `**, de modo que `println!(stderr, "{err}")` da byte a
//! byte la misma línea que el Python. Los *exit codes* también son contrato.

use std::path::PathBuf;
use thiserror::Error;

/// Error del dominio de `podtrack`.
#[derive(Debug, Error)]
pub enum PodtrackError {
    /// El tracker declarado por el perfil no existe (exit 1).
    #[error("ERROR: No se encuentra {path}")]
    TrackerMissing { path: PathBuf },

    /// El YAML existe pero no declara episodios (exit 1).
    #[error("ERROR: El YAML no tiene episodios")]
    TrackerNoEpisodes,

    /// El YAML no es legible (exit 1).
    #[error("ERROR: Tracker ilegible: {reason}")]
    Yaml { reason: String },

    /// El objetivo de `get`/`where` no corresponde a ningún episodio (exit 1).
    #[error("ERROR: Episodio '{target}' no encontrado.")]
    EpisodeNotFound { target: String },

    /// `where` sobre un episodio sin slug (exit 1).
    #[error(
        "ERROR: El episodio #{label} no tiene slug; no se puede resolver la ruta de su material."
    )]
    EpisodeWithoutSlug { label: String },

    /// `status` fuera del conjunto cerrado (exit 1).
    #[error("ERROR: Estado inválido '{value}'. Válidos: {valid}")]
    InvalidStatus { value: String, valid: String },

    /// `slug` que no es kebab-case (exit 1).
    #[error("ERROR: Slug inválido '{value}' (se espera kebab-case).")]
    InvalidSlug { value: String },

    /// Fecha que no es `YYYY-MM-DD` (exit 1).
    #[error("ERROR: Fecha inválida '{value}': se espera YYYY-MM-DD")]
    InvalidDate { value: String },

    /// Perfil pedido por `--show` (o por defecto) que no se encuentra (exit 2).
    ///
    /// **Divergencia deliberada con el oráculo:** el Python, si falta el perfil
    /// *por defecto*, cae en silencio a una ruta legacy; aquí también es error.
    #[error(
        "ERROR: No existe el perfil de podcast '{slug}'.\n       Se buscó el perfil en:\n{searched}\n       Perfiles disponibles: {available}"
    )]
    ProfileNotFound {
        slug: String,
        searched: String,
        available: String,
    },

    /// Perfil sin la clave `tracker` (exit 2).
    #[error("ERROR: El perfil '{slug}' ({path}) no declara la clave 'tracker'.")]
    ProfileMissingTrackerKey { slug: String, path: PathBuf },

    /// Perfil **mal formado** (contenido inválido: el YAML no parsea) (exit 2).
    ///
    /// Causa de **contenido**, distinta de la de lectura
    /// (`ProfileUnreadable`) y de la ruta del tracker irresoluble
    /// (`ProfileTrackerUnresolvable`).
    #[error("ERROR: El perfil '{slug}' ({path}) no es válido: {reason}")]
    InvalidProfile {
        slug: String,
        path: PathBuf,
        reason: String,
    },

    /// Perfil ilegible: el fichero no se pudo leer (exit 2).
    ///
    /// Causa de **lectura** —permisos, la ruta es un directorio, un enlace roto—,
    /// distinta del contenido mal formado (`InvalidProfile`) y de la ruta del
    /// tracker irresoluble (`ProfileTrackerUnresolvable`).
    #[error("ERROR: No se pudo leer el perfil '{slug}' ({path}): {reason}")]
    ProfileUnreadable {
        slug: String,
        path: PathBuf,
        reason: String,
    },

    /// Perfil que se lee y parsea, pero cuya ruta de `tracker:` no se puede
    /// resolver a absoluta (exit 2).
    #[error(
        "ERROR: El perfil '{slug}' ({path}): no se pudo resolver la ruta del tracker: {reason}"
    )]
    ProfileTrackerUnresolvable {
        slug: String,
        path: PathBuf,
        reason: String,
    },

    /// Perfil que declara `tracker: none` (exit 3).
    #[error(
        "ERROR: El podcast \"{name}\" no usa tracker de episodios.\n       El perfil '{slug}' declara 'tracker: none'.\n       El número de episodio se deduce del listado de carpetas del show\n       (mayor 'episodio-NN' en disco + 1), no de este script."
    )]
    ProfileNoTracker { name: String, slug: String },

    /// Alta con un número ya ocupado (exit 1).
    #[error("ERROR: El episodio #{number} ya existe.")]
    AddDuplicate { number: u32 },

    /// Uso incorrecto de un subcomando (exit 2, como `argparse`).
    #[error("ERROR: El título no puede estar vacío.")]
    EmptyTitle,

    #[error("ERROR: Uso: {usage}")]
    Usage { usage: String },

    /// Número de episodio fuera del rango representable (exit 1).
    #[error("ERROR: Número de episodio inválido '{value}'.")]
    InvalidNumber { value: String },

    /// `insert` sobre un número que no existe (exit 1).
    #[error("ERROR: No existe el episodio #{number}.")]
    InsertMissing { number: u32 },

    /// `insert --date` que no coincide con el slot (exit 1).
    #[error("ERROR: La fecha {date} no coincide con el slot de #{before} ({slot}).")]
    InsertDateMismatch {
        date: String,
        before: u32,
        slot: String,
    },

    /// `publish` de un episodio ya publicado (exit 1).
    #[error("ERROR: El episodio #{label} ('{name}') ya está publicado.")]
    PublishAlreadyPublished { label: String, name: String },

    /// `move --date` a un día no publicable (exit 1).
    #[error("ERROR: {date} no es un día de publicación del show (lunes o jueves).")]
    MoveNotPublishDay { date: String },

    /// `move --date` a una fecha ya ocupada (exit 1).
    #[error("ERROR: Colisión de fecha: #{label} ya emite el {date}.")]
    MoveCollision { label: String, date: String },

    /// Renombrado de material con un destino ya existente (exit 1).
    #[error("  ERROR: el destino ya existe: {path}")]
    MaterialDestinationExists { path: String },

    /// `renumber`/`rename-material` con `--delta 0` (exit 1).
    #[error("ERROR: --delta no puede ser 0.")]
    DeltaZero,

    /// `renumber` sin episodios en el umbral `--from` (exit 1).
    #[error("ERROR: No hay episodios con número >= {from}.")]
    RenumberNoEpisodes { from: i64 },

    /// `renumber` con un delta que solapa con números no afectados (exit 1).
    #[error("ERROR: Colisión de numeración: el --delta solapa con episodios existentes.")]
    RenumberCollision,

    /// `rename-material` sin `--from`/`--delta` ni `--check` (exit 1).
    #[error("ERROR: rename-material requiere --from y --delta (o --check).")]
    RenameMaterialRequiresArgs,

    /// `srt-stage`: perfil inexistente (exit 2; mensaje propio de `srt-stage.py`).
    #[error(
        "ERROR: No existe el perfil de podcast '{slug}'.\n       Se esperaba el fichero {expected}\n       Perfiles disponibles: {available}"
    )]
    SrtProfileNotFound {
        slug: String,
        expected: String,
        available: String,
    },

    /// `srt-stage`: perfil ilegible (exit 2).
    #[error("ERROR: No se pudo leer el perfil {path}: {reason}")]
    SrtProfileUnreadable { path: String, reason: String },

    /// `srt-stage`: mensaje literal ya formateado (etapas, argumentos) (exit 2).
    #[error("{message}")]
    SrtStageError { message: String },

    /// `srt-stage`: fichero sin estructura de cues (exit 2).
    #[error("ERROR: El fichero {path} no parece un SRT: sin cues.")]
    SrtNoCues { path: String },

    /// `srt-stage`: SRT ilegible (exit 2).
    #[error("ERROR: No se pudo leer el SRT {path}: {reason}")]
    SrtRead { path: String, reason: String },

    /// `open`: el perfil no declara `episodes.scripts.files.presentacion` (exit 2).
    #[error("ERROR: El show '{show}' no declara una presentación en el perfil (episodes.scripts.files.presentacion).")]
    PresentationNotDeclared { show: String },

    /// `open`: el `.typ` declarado no existe en disco (exit 1).
    #[error("ERROR: No existe la presentación del episodio '{slug}': {path}")]
    PresentationNotFound { slug: String, path: PathBuf },

    /// `open`: el lanzador no está en el PATH (exit 1).
    #[error("ERROR: No se encuentra el lanzador '{launcher}' en el PATH.")]
    LauncherNotFound { launcher: String },

    /// `open`: el layout necesita `{n}` y el episodio no tiene número (exit 1).
    #[error("ERROR: El episodio '{label}' no tiene número; no se puede resolver la ruta de su presentación.")]
    EpisodeWithoutNumber { label: String },

    /// Error de E/S genérico (exit 1).
    #[error("ERROR: {0}")]
    Io(String),
}

impl PodtrackError {
    /// Código de salida asociado, reproduciendo el del oráculo.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            // Perfil explícito inexistente / inválido.
            Self::ProfileNotFound { .. }
            | Self::ProfileMissingTrackerKey { .. }
            | Self::InvalidProfile { .. }
            | Self::ProfileUnreadable { .. }
            | Self::ProfileTrackerUnresolvable { .. } => 2,
            // `srt-stage`: perfil/SRT/argumentos inválidos (contrato de la fase 4).
            Self::SrtProfileNotFound { .. }
            | Self::SrtProfileUnreadable { .. }
            | Self::SrtStageError { .. }
            | Self::SrtNoCues { .. }
            | Self::SrtRead { .. } => 2,
            // Perfil que no usa tracker.
            Self::ProfileNoTracker { .. } => 3,
            // Uso incorrecto de un subcomando: `argparse` sale con 2.
            Self::Usage { .. } => 2,
            // Todo lo demás (tracker ausente, episodio inexistente, estado/slug/fecha
            // inválidos, E/S) es exit 1, como el `sys.exit(1)` del Python.
            _ => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PodtrackError;
    use std::path::PathBuf;

    #[test]
    fn exit_codes_reproducen_los_del_oraculo() {
        assert_eq!(
            PodtrackError::TrackerMissing {
                path: PathBuf::from("/x")
            }
            .exit_code(),
            1
        );
        assert_eq!(
            PodtrackError::ProfileNotFound {
                slug: "x".into(),
                searched: String::new(),
                available: "ninguno".into()
            }
            .exit_code(),
            2
        );
        assert_eq!(
            PodtrackError::ProfileNoTracker {
                name: "x".into(),
                slug: "x".into()
            }
            .exit_code(),
            3
        );
    }

    #[test]
    fn los_errores_de_perfil_nuevos_salen_con_dos() {
        let ilegible = PodtrackError::ProfileUnreadable {
            slug: "x".into(),
            path: PathBuf::from("/p"),
            reason: "Is a directory".into(),
        };
        assert_eq!(ilegible.exit_code(), 2);
        let msg = ilegible.to_string();
        assert!(msg.contains("No se pudo leer el perfil"), "mensaje: {msg}");
        assert!(!msg.contains("no es válido"), "mensaje: {msg}");

        let irresoluble = PodtrackError::ProfileTrackerUnresolvable {
            slug: "x".into(),
            path: PathBuf::from("/p"),
            reason: "cwd".into(),
        };
        assert_eq!(irresoluble.exit_code(), 2);
        let msg = irresoluble.to_string();
        assert!(
            msg.contains("no se pudo resolver la ruta del tracker"),
            "mensaje: {msg}"
        );
    }

    #[test]
    fn los_errores_de_open_salen_con_su_codigo_y_mensaje() {
        let no_declarada = PodtrackError::PresentationNotDeclared {
            show: "la-era-de-las-distros".into(),
        };
        assert_eq!(no_declarada.exit_code(), 2, "defecto de perfil: exit 2");
        let msg = no_declarada.to_string();
        assert!(msg.contains("no declara una presentación"), "mensaje: {msg}");

        let no_existe = PodtrackError::PresentationNotFound {
            slug: "la-memoria-de-valet".into(),
            path: PathBuf::from("/x/la-memoria-de-valet/presentacion-la-memoria-de-valet.typ"),
        };
        assert_eq!(no_existe.exit_code(), 1);
        let msg = no_existe.to_string();
        assert!(
            msg.contains("No existe la presentación del episodio"),
            "mensaje: {msg}"
        );
        assert!(msg.contains("la-memoria-de-valet"), "mensaje: {msg}");

        let lanzador = PodtrackError::LauncherNotFound {
            launcher: "typshow".into(),
        };
        assert_eq!(lanzador.exit_code(), 1);
        let msg = lanzador.to_string();
        assert!(msg.contains("No se encuentra el lanzador"), "mensaje: {msg}");

        let sin_numero = PodtrackError::EpisodeWithoutNumber {
            label: "—".into(),
        };
        assert_eq!(sin_numero.exit_code(), 1);
        let msg = sin_numero.to_string();
        assert!(msg.contains("no tiene número"), "mensaje: {msg}");
    }
}
