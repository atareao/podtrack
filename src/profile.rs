//! Resolución del perfil del show en cascada (D5).
//!
//! `--show <slug>` → busca `shows/<slug>.yaml` en: `PODTRACK_SHOWS_DIR` (env) →
//! directorio junto al ejecutable → `~/.config/podtrack/shows/`. Sin `--show` se
//! usa `PODTRACK_SHOW` y, si no, el show por defecto compilado.

use std::path::{Component, Path, PathBuf};

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
    /// Ruta del YAML de estado declarada por el perfil, **absoluta y normalizada
    /// léxicamente** (normalización en el constructor: D1; base del `tracker:`
    /// relativo: D3 — del change `profile-absolute-paths`).
    pub tracker: PathBuf,
}

impl ResolvedShow {
    /// Raíz del árbol de guiones: el **directorio padre del tracker**
    /// (`_scripts_root` del oráculo).
    ///
    /// Es **siempre absoluta**: `tracker` se resuelve a absoluto al construir el
    /// perfil, y para un `tracker` sin padre (el caso límite de la raíz `/`) se
    /// devuelve la propia ruta, nunca el `"."` de emergencia.
    #[must_use]
    pub fn scripts_root(&self) -> PathBuf {
        self.tracker
            .parent()
            .map_or_else(|| self.tracker.clone(), Path::to_path_buf)
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

    /// Primer fichero `shows/<slug>.yaml` **existente** en la cascada.
    ///
    /// Es la vía que usan los comandos que **no** operan sobre una parrilla
    /// (`srt-stage`): localiza el perfil sin exigir la clave `tracker`.
    #[must_use]
    pub fn find_profile(&self, slug: &str) -> Option<PathBuf> {
        self.candidate_dirs()
            .into_iter()
            .map(|dir| dir.join(format!("{slug}.yaml")))
            .find(|path| path.is_file())
    }

    /// Primer candidato de la cascada (exista o no), para los mensajes de error.
    #[must_use]
    pub fn first_candidate(&self, slug: &str) -> PathBuf {
        self.candidate_dirs().into_iter().next().map_or_else(
            || PathBuf::from(format!("{slug}.yaml")),
            |dir| dir.join(format!("{slug}.yaml")),
        )
    }

    /// Perfiles disponibles (`*.yaml`) en los directorios candidatos, sin repetir.
    #[must_use]
    pub fn available_profiles(&self) -> String {
        self.available()
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
                    let base_dir = path.parent().unwrap_or_else(|| Path::new(""));
                    let tracker = absolutize(Path::new(&raw), base_dir).map_err(|e| {
                        PodtrackError::InvalidProfile {
                            slug: slug.to_string(),
                            path: path.to_path_buf(),
                            reason: e.to_string(),
                        }
                    })?;
                    Ok(ResolvedShow {
                        slug: slug.to_string(),
                        name,
                        tracker,
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

/// Colapsa léxicamente los componentes `.` y `..` de una ruta, **sin E/S**.
///
/// `Component::CurDir` se descarta; `Component::ParentDir` hace *pop* del último
/// `Component::Normal` y, si ya no queda ninguno por encima de la raíz o del
/// prefijo, se ignora (`/..` → `/`). En una ruta relativa sin raíz, los `..` que
/// suben por encima del inicio se conservan. `RootDir` y `Prefix` se preservan.
fn normalize_lexically(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => match out.components().next_back() {
                Some(Component::Normal(_)) => {
                    out.pop();
                }
                Some(Component::RootDir | Component::Prefix(_)) => {}
                _ => out.push(".."),
            },
            _ => out.push(component.as_os_str()),
        }
    }
    out
}

/// Devuelve la ruta `path` como **absoluta y normalizada léxicamente**.
///
/// Si `path` ya es absoluta se usa tal cual (normalizada). Si es relativa, se
/// resuelve contra `base_dir`; si `base_dir` fuera a su vez relativa, contra
/// `std::env::current_dir()`. La resolución es **puramente léxica**: no consulta
/// el sistema de ficheros, no exige que la ruta exista, no resuelve enlaces
/// simbólicos y no crea nada.
fn absolutize(path: &Path, base_dir: &Path) -> std::io::Result<PathBuf> {
    if path.is_absolute() {
        return Ok(normalize_lexically(path));
    }
    let base = if base_dir.is_absolute() {
        base_dir.to_path_buf()
    } else {
        std::env::current_dir()?.join(base_dir)
    };
    Ok(normalize_lexically(&base.join(path)))
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

#[cfg(test)]
mod tests {
    use super::{ResolvedShow, ShowResolver, absolutize, normalize_lexically};
    use std::path::{Path, PathBuf};

    /// `ResolvedShow` de conveniencia con un `tracker` ya resuelto.
    fn resolved(tracker: &str) -> ResolvedShow {
        ResolvedShow {
            slug: "demo".to_string(),
            name: "Demo".to_string(),
            tracker: PathBuf::from(tracker),
        }
    }

    #[test]
    fn scripts_root_de_un_tracker_en_la_raiz_no_cae_al_directorio_actual() {
        // `parent()` de `/` es `None`: el contrato es que NO se caiga a `"."`.
        assert_eq!(resolved("/").scripts_root(), PathBuf::from("/"));
    }

    #[test]
    fn scripts_root_es_el_directorio_padre_del_tracker() {
        assert_eq!(resolved("/x/y.yaml").scripts_root(), PathBuf::from("/x"));
    }

    #[test]
    fn un_tracker_absoluto_y_limpio_se_conserva_identico() {
        let tracker = Path::new("/data/notas/Podcasts/guiones/podcast-tracker.yaml");
        let abs = absolutize(tracker, Path::new("/cualquier/otra/base")).expect("absolutize");
        assert_eq!(abs, tracker);
        assert_eq!(
            resolved(abs.to_str().expect("utf8")).scripts_root(),
            PathBuf::from("/data/notas/Podcasts/guiones")
        );
    }

    #[test]
    fn los_componentes_curdir_y_parentdir_se_normalizan_lexicamente() {
        assert_eq!(
            normalize_lexically(Path::new("/a/./b/../c.yaml")),
            PathBuf::from("/a/c.yaml")
        );
        assert_eq!(normalize_lexically(Path::new("/..")), PathBuf::from("/"));
    }

    #[test]
    fn un_tracker_relativo_se_resuelve_contra_el_directorio_del_perfil() {
        let abs = absolutize(
            Path::new("../guiones/podcast-tracker.yaml"),
            Path::new("/base/shows"),
        )
        .expect("absolutize");
        assert_eq!(abs, PathBuf::from("/base/guiones/podcast-tracker.yaml"));
    }

    #[test]
    fn una_base_relativa_se_absolutiza_contra_el_directorio_de_trabajo() {
        let abs =
            absolutize(Path::new("guiones/tracker.yaml"), Path::new("shows")).expect("absolutize");
        assert!(abs.is_absolute(), "debe ser absoluta: {abs:?}");
        assert!(abs.ends_with("shows/guiones/tracker.yaml"), "ruta: {abs:?}");
    }

    #[test]
    fn un_parentdir_que_sube_por_encima_de_la_base_es_lexico() {
        let abs = absolutize(Path::new("../../x.yaml"), Path::new("/a/b")).expect("absolutize");
        assert_eq!(abs, PathBuf::from("/x.yaml"));
    }

    #[test]
    fn una_ruta_relativa_sin_raiz_conserva_los_parentdir_sobrantes() {
        assert_eq!(
            normalize_lexically(Path::new("a/../../b.yaml")),
            PathBuf::from("../b.yaml")
        );
    }

    #[test]
    fn load_resuelve_un_tracker_relativo_contra_el_directorio_del_perfil() {
        let dir = tempfile::TempDir::new().expect("tmp");
        let shows = dir.path().join("shows");
        std::fs::create_dir_all(&shows).expect("mkdir");
        let profile = shows.join("demo.yaml");
        std::fs::write(
            &profile,
            "name: Demo\ntracker: ../guiones/podcast-tracker.yaml\n",
        )
        .expect("write profile");

        let resolver = ShowResolver {
            shows_dir: Some(shows),
            exe_dir: None,
            config_dir: None,
            default_slug: "demo".to_string(),
        };
        let resolved = resolver.resolve(Some("demo")).expect("resolve");
        assert_eq!(
            resolved.tracker,
            dir.path().join("guiones/podcast-tracker.yaml")
        );
        assert_eq!(resolved.scripts_root(), dir.path().join("guiones"));
    }
}
