//! Resolución del perfil del show en cascada (D5).
//!
//! `--show <slug>` → busca `shows/<slug>.yaml` en: `PODTRACK_SHOWS_DIR` (env) →
//! directorio junto al ejecutable → `~/.config/podtrack/shows/`. Sin `--show` se
//! usa `PODTRACK_SHOW` y, si no, el show por defecto compilado.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::PodtrackError;

/// Show por defecto cuando no hay `--show` ni `PODTRACK_SHOW` (D1 del SRS).
pub const DEFAULT_SHOW: &str = "atareao-con-linux";

/// Perfil resuelto: el slug pedido, el nombre visible y la ruta del tracker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedShow {
    /// Slug del perfil.
    pub slug: String,
    /// Nombre visible del show (cabecera del tracker).
    pub name: String,
    /// Ruta del YAML de estado declarada por el perfil.
    pub tracker: PathBuf,
}

impl ResolvedShow {
    /// Raíz del árbol de guiones: el **directorio padre del tracker**
    /// (`_scripts_root` del oráculo).
    #[must_use]
    pub fn scripts_root(&self) -> PathBuf {
        self.tracker
            .parent()
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
    }
}

/// Resolutor de perfiles con la cascada de búsqueda, inyectable para tests.
#[derive(Debug, Clone)]
pub struct ShowResolver {
    /// Directorio de `PODTRACK_SHOWS_DIR` (tiene prioridad).
    pub shows_dir: Option<PathBuf>,
    /// Directorio del ejecutable (se busca en `<exe_dir>/shows`).
    pub exe_dir: Option<PathBuf>,
    /// Directorio de configuración (`~/.config/podtrack/shows`).
    pub config_dir: Option<PathBuf>,
    /// Slug del show por defecto (`PODTRACK_SHOW` o el compilado).
    pub default_slug: String,
}

impl ShowResolver {
    /// Construye el resolutor leyendo el entorno y el directorio del ejecutable.
    #[must_use]
    pub fn from_env(exe_dir: Option<PathBuf>) -> Self {
        let shows_dir = std::env::var_os("PODTRACK_SHOWS_DIR")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from);
        let default_slug = std::env::var("PODTRACK_SHOW")
            .ok()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_SHOW.to_string());
        Self {
            shows_dir,
            exe_dir,
            config_dir: config_dir(),
            default_slug,
        }
    }

    /// Directorios candidatos, en orden de prioridad.
    #[must_use]
    pub fn candidate_dirs(&self) -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        if let Some(dir) = &self.shows_dir {
            dirs.push(dir.clone());
        }
        if let Some(dir) = &self.exe_dir {
            dirs.push(dir.join("shows"));
        }
        if let Some(dir) = &self.config_dir {
            dirs.push(dir.clone());
        }
        dirs
    }

    /// Resuelve el perfil pedido (o el por defecto).
    ///
    /// La cascada se aplica cuando `--show` **no** es una ruta absoluta; una ruta
    /// absoluta se carga tal cual (spec `show-profile`).
    pub fn resolve(&self, requested: Option<&str>) -> Result<ResolvedShow, PodtrackError> {
        if let Some(raw) = requested.map(str::trim).filter(|s| !s.is_empty()) {
            let candidate = Path::new(raw);
            if candidate.is_absolute() {
                let slug = candidate
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(raw)
                    .to_string();
                if candidate.is_file() {
                    return self.load(&slug, candidate);
                }
                return Err(PodtrackError::ProfileNotFound {
                    slug,
                    searched: format!("         - {}", candidate.display()),
                    available: self.available(),
                });
            }
        }

        let slug = match requested {
            Some(s) if !s.trim().is_empty() => s.trim().to_string(),
            _ => self.default_slug.clone(),
        };

        let mut searched = Vec::new();
        for dir in self.candidate_dirs() {
            let path = dir.join(format!("{slug}.yaml"));
            searched.push(path.clone());
            if path.is_file() {
                return self.load(&slug, &path);
            }
        }

        Err(PodtrackError::ProfileNotFound {
            slug,
            searched: searched
                .iter()
                .map(|p| format!("         - {}", p.display()))
                .collect::<Vec<_>>()
                .join("\n"),
            available: self.available(),
        })
    }

    fn load(&self, slug: &str, path: &Path) -> Result<ResolvedShow, PodtrackError> {
        let text = std::fs::read_to_string(path).map_err(|e| PodtrackError::InvalidProfile {
            slug: slug.to_string(),
            path: path.to_path_buf(),
            reason: e.to_string(),
        })?;
        let profile: RawProfile =
            serde_yaml::from_str(&text).map_err(|e| PodtrackError::InvalidProfile {
                slug: slug.to_string(),
                path: path.to_path_buf(),
                reason: e.to_string(),
            })?;

        let name = profile
            .name
            .filter(|n| !n.trim().is_empty())
            .unwrap_or_else(|| slug.to_string());

        match profile.tracker {
            None | Some(serde_yaml::Value::Null) => Err(PodtrackError::ProfileMissingTrackerKey {
                slug: slug.to_string(),
                path: path.to_path_buf(),
            }),
            Some(value) => {
                let raw = scalar_to_string(&value);
                if raw.trim().is_empty() || raw.trim().eq_ignore_ascii_case("none") {
                    Err(PodtrackError::ProfileNoTracker {
                        name,
                        slug: slug.to_string(),
                    })
                } else {
                    Ok(ResolvedShow {
                        slug: slug.to_string(),
                        name,
                        tracker: PathBuf::from(raw),
                    })
                }
            }
        }
    }

    /// Perfiles disponibles (`*.yaml`) en los directorios candidatos, sin repetir.
    fn available(&self) -> String {
        let mut slugs: Vec<String> = Vec::new();
        for dir in self.candidate_dirs() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("yaml")
                    && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                {
                    let stem = stem.to_string();
                    if !slugs.contains(&stem) {
                        slugs.push(stem);
                    }
                }
            }
        }
        slugs.sort();
        if slugs.is_empty() {
            "ninguno".to_string()
        } else {
            slugs.join(", ")
        }
    }
}

#[derive(Deserialize)]
struct RawProfile {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    tracker: Option<serde_yaml::Value>,
}

fn scalar_to_string(value: &serde_yaml::Value) -> String {
    match value {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        other => serde_yaml::to_string(other)
            .map(|s| s.trim().to_string())
            .unwrap_or_default(),
    }
}

fn config_dir() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .filter(|v| !v.is_empty())
                .map(|home| PathBuf::from(home).join(".config"))
        })?;
    Some(base.join("podtrack").join("shows"))
}
