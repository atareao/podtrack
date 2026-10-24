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

    /// Perfil ilegible o mal formado (exit 2).
    #[error("ERROR: El perfil '{slug}' ({path}) no es válido: {reason}")]
    InvalidProfile {
        slug: String,
        path: PathBuf,
        reason: String,
    },

    /// Perfil que declara `tracker: none` (exit 3).
    #[error(
        "ERROR: El podcast \"{name}\" no usa tracker de episodios.\n       El perfil '{slug}' declara 'tracker: none'.\n       El número de episodio se deduce del listado de carpetas del show\n       (mayor 'episodio-NN' en disco + 1), no de este script."
    )]
    ProfileNoTracker { name: String, slug: String },

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
            | Self::InvalidProfile { .. } => 2,
            // Perfil que no usa tracker.
            Self::ProfileNoTracker { .. } => 3,
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
}
