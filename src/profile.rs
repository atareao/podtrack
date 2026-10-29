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
    /// Plantilla del directorio de material (`episodes.scripts.dir`), sin renderizar.
    pub scripts_dir: Option<String>,
    /// Plantilla del fichero de presentación (`episodes.scripts.files.presentacion`),
    /// sin renderizar.
    pub presentation: Option<String>,
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

    /// Carpeta de material de un episodio: el **punto único** de derivación.
    ///
    /// Todas las superficies del binario que deriven la carpeta de material de un
    /// episodio —las que **informan** de ella (`where`, `get`, `status`, sus
    /// documentos `--json`) y las que **operan** sobre el material
    /// (`rebase-references`, `collect_slug_renames` de `migrate-material`) y
    /// `open`— SHALL pasar por aquí, para que la regla de resolución sea **una
    /// sola** (spec `show-profile`). Ningún subcomando SHALL volver a cablear el
    /// slug por su cuenta.
    ///
    /// La regla es `<scripts_root>/<dir>`, con `dir` = `episodes.scripts.dir`
    /// **renderizado** (`{slug}`, `{n}`, `{n:0Nd}`); si el perfil no declara `dir`
    /// o lo declara vacío, `dir` por defecto es `{slug}` (mismo fallback que usaba
    /// `open`).
    ///
    /// El `dir` renderizado SHALL ser **relativo**: si es absoluto o contiene un
    /// componente `..`, la raíz de material no es una convención sino una frontera
    /// y se devuelve [`PodtrackError::MaterialDirNotRelative`] en vez de resolver
    /// una ruta fuera de ella.
    ///
    /// # Errors
    ///
    /// - `EpisodeWithoutSlug` — el layout usa `{slug}` y el episodio no tiene slug.
    /// - `EpisodeWithoutNumber` — el layout usa `{n}`/`{n:0Nd}` y el episodio no
    ///   tiene número.
    /// - `MaterialDirNotRelative` — el `dir` renderizado es absoluto o contiene un
    ///   componente `..` (defecto de configuración del perfil).
    pub fn material_dir(
        &self,
        slug: Option<&str>,
        number: Option<u32>,
    ) -> Result<PathBuf, PodtrackError> {
        let template = self
            .scripts_dir
            .as_deref()
            .filter(|d| !d.is_empty())
            .unwrap_or(DEFAULT_SCRIPTS_DIR);
        let label = render_episode_label(slug, number);
        let dir = render_material_template(template, slug, number, &label)?;

        // D5: el `dir` renderizado es una frontera, no una convención. Se comprueba
        // **antes** de unirlo a la raíz, para no resolver nunca fuera de ella.
        let dir_path = Path::new(&dir);
        if dir_path.is_absolute() || dir_path.components().any(|c| c == Component::ParentDir) {
            return Err(PodtrackError::MaterialDirNotRelative {
                show: self.slug.clone(),
                dir,
            });
        }
        Ok(self.scripts_root().join(dir_path))
    }
}

/// Carpeta de material asumida si el perfil no declara `episodes.scripts.dir`
/// (mismo fallback que usaba `open`).
const DEFAULT_SCRIPTS_DIR: &str = "{slug}";

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
        let text = std::fs::read_to_string(path).map_err(|e| PodtrackError::ProfileUnreadable {
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

        let scripts_dir = profile
            .episodes
            .as_ref()
            .and_then(|e| e.get("scripts"))
            .and_then(|s| s.get("dir"))
            .and_then(yaml_scalar_str)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let presentation = profile
            .episodes
            .as_ref()
            .and_then(|e| e.get("scripts"))
            .and_then(|s| s.get("files"))
            .and_then(|f| f.get("presentacion"))
            .and_then(yaml_scalar_str)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        match profile.tracker {
            None | Some(serde_yaml::Value::Null) => Err(PodtrackError::ProfileMissingTrackerKey {
                slug: slug.to_string(),
                path: path.to_path_buf(),
            }),
            Some(value) => {
                let raw = scalar_to_string(&value);
                let raw = raw.trim();
                if raw.is_empty() || raw.eq_ignore_ascii_case("none") {
                    Err(PodtrackError::ProfileNoTracker {
                        name,
                        slug: slug.to_string(),
                    })
                } else {
                    let base_dir = path.parent().unwrap_or_else(|| Path::new(""));
                    // Guarda y resolución miran el mismo valor recortado: un tracker
                    // entrecomillado con espacios resuelve limpio.
                    let tracker = absolutize(Path::new(raw), base_dir).map_err(|e| {
                        PodtrackError::ProfileTrackerUnresolvable {
                            slug: slug.to_string(),
                            path: path.to_path_buf(),
                            reason: e.to_string(),
                        }
                    })?;
                    Ok(ResolvedShow {
                        slug: slug.to_string(),
                        name,
                        tracker,
                        scripts_dir,
                        presentation,
                    })
                }
            }
        }
    }

    /// Slugs de los perfiles (`*.yaml`) de los directorios candidatos, sin repetir y ordenados.
    #[must_use]
    pub fn available_slugs(&self) -> Vec<String> {
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
        slugs
    }

    /// Perfiles disponibles (`*.yaml`) en los directorios candidatos, sin repetir.
    fn available(&self) -> String {
        let slugs = self.available_slugs();
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
    /// Bloque `episodes` **sin tipar**: se navega por clave en `load`, de forma
    /// **laxa**, para no reinterpretar el esquema del perfil ni romper la carga
    /// ante variaciones de tipo (`files.futuro: [a, b]`, `dir: {slug}` como *flow
    /// mapping*, …). Igual que `tracker`, solo interesa su contenido puntual.
    #[serde(default)]
    episodes: Option<serde_yaml::Value>,
}

/// Extrae un escalar de texto de un `Value` YAML (`None` si no es escalar).
///
/// La lectura del perfil es **laxa y por clave**: un valor no escalar (secuencia,
/// mapping, nulo) se trata como **ausente**, nunca como error.
fn yaml_scalar_str(value: &serde_yaml::Value) -> Option<String> {
    match value {
        serde_yaml::Value::String(s) => Some(s.clone()),
        serde_yaml::Value::Number(n) => Some(n.to_string()),
        serde_yaml::Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
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

/// Etiqueta de un episodio para los mensajes de error del render: el `slug` si lo
/// tiene (es lo útil cuando falta el número) y, si no, el número (`—` si tampoco).
///
/// Es la etiqueta que [`render_material_template`] recibe en `label`: un
/// `EpisodeWithoutSlug` (slug ausente) nombra el número, como la línea base; un
/// `EpisodeWithoutNumber` (número ausente) nombra el slug, en vez del `—` inútil.
#[must_use]
pub fn render_episode_label(slug: Option<&str>, number: Option<u32>) -> String {
    match slug.filter(|s| !s.is_empty()) {
        Some(slug) => slug.to_string(),
        None => crate::tracker::num_label(number),
    }
}

/// Renderiza una plantilla de material del perfil: `{slug}`, `{n}`, `{n:0Nd}`.
///
/// Sustituye `{slug}` por el slug y `{n}` / `{n:0Nd}` por el número (con relleno
/// opcional a N dígitos). Falta el valor → error tipado; marcador desconocido →
/// literal. `label` es la etiqueta del episodio para el mensaje de error
/// (normalmente [`render_episode_label`]).
pub fn render_material_template(
    template: &str,
    slug: Option<&str>,
    number: Option<u32>,
    label: &str,
) -> Result<String, PodtrackError> {
    let mut out = String::new();
    let mut i = 0usize;
    while let Some(rel) = template[i..].find('{') {
        let open = i + rel;
        out.push_str(&template[i..open]);
        match template[open + 1..].find('}') {
            Some(rel_close) => {
                let close = open + 1 + rel_close;
                let marker = &template[open + 1..close];
                out.push_str(&render_marker(marker, slug, number, label)?);
                i = close + 1;
            }
            None => {
                // `{` sin cierre: se deja literal el resto.
                out.push_str(&template[open..]);
                return Ok(out);
            }
        }
    }
    out.push_str(&template[i..]);
    Ok(out)
}

/// Sustituye **un** marcador (`{…}`) por su valor, o lo deja literal si es
/// desconocido. Marcadores soportados: `slug`, `n`, `n:0Nd`.
///
/// En `n:0Nd`, el ancho `N` debe ser un `usize` **≤ 32**; un ancho no parseable o
/// mayor se considera marcador **desconocido** y se devuelve literal (evita
/// reservar una cantidad absurda de memoria en `{n:010000000000d}`).
fn render_marker(
    marker: &str,
    slug: Option<&str>,
    number: Option<u32>,
    label: &str,
) -> Result<String, PodtrackError> {
    match marker {
        "slug" => slug
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .ok_or_else(|| PodtrackError::EpisodeWithoutSlug {
                label: label.to_string(),
            }),
        "n" => number
            .map(|n| n.to_string())
            .ok_or_else(|| PodtrackError::EpisodeWithoutNumber {
                label: label.to_string(),
            }),
        _ => {
            // `{n:0Nd}`: `0` + dígitos + `d`. Cualquier otra forma es desconocida.
            // Un ancho **absurdo** (no parseable a `usize` o `> 32`, p. ej.
            // `{n:010000000000d}`) también se trata como marcador desconocido, para
            // no intentar reservar una cantidad disparatada de memoria: se devuelve
            // literal.
            if let Some(rest) = marker.strip_prefix("n:")
                && let Some(inner) = rest.strip_prefix('0')
                && let Some(digits) = inner.strip_suffix('d')
                && !digits.is_empty()
                && digits.chars().all(|c| c.is_ascii_digit())
                && let Ok(width) = digits.parse::<usize>()
                && width <= 32
            {
                return number.map(|n| format!("{n:0width$}")).ok_or_else(|| {
                    PodtrackError::EpisodeWithoutNumber {
                        label: label.to_string(),
                    }
                });
            }
            Ok(format!("{{{marker}}}"))
        }
    }
}

/// Directorio del ejecutable en curso, si el sistema lo puede resolver.
///
/// Es el segundo candidato de la cascada de perfiles (`<exe_dir>/shows`).
#[must_use]
pub fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
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
    use super::{
        ResolvedShow, ShowResolver, absolutize, normalize_lexically, render_episode_label,
        render_material_template,
    };
    use crate::error::PodtrackError;
    use std::path::{Path, PathBuf};

    /// `ResolvedShow` de conveniencia con un `tracker` ya resuelto.
    fn resolved(tracker: &str) -> ResolvedShow {
        ResolvedShow {
            slug: "demo".to_string(),
            name: "Demo".to_string(),
            tracker: PathBuf::from(tracker),
            scripts_dir: None,
            presentation: None,
        }
    }

    /// `ResolvedShow` de conveniencia con una plantilla de `scripts.dir` dada.
    fn resolved_with_dir(tracker: &str, dir: Option<&str>) -> ResolvedShow {
        ResolvedShow {
            slug: "demo".to_string(),
            name: "Demo".to_string(),
            tracker: PathBuf::from(tracker),
            scripts_dir: dir.map(str::to_string),
            presentation: None,
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

    #[test]
    fn load_recorta_el_tracker_entrecomillado_con_espacios() {
        // YAML preserva los espacios de un escalar entrecomillado: el `trim`
        // de la guarda y la resolución deben usar la MISMA cadena recortada.
        let dir = tempfile::TempDir::new().expect("tmp");
        let shows = dir.path().join("shows");
        std::fs::create_dir_all(&shows).expect("mkdir");
        let profile = shows.join("demo.yaml");
        std::fs::write(
            &profile,
            "name: Demo\ntracker: \" /data/notas/Podcasts/guiones/podcast-tracker.yaml \"\n",
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
            Path::new("/data/notas/Podcasts/guiones/podcast-tracker.yaml")
        );
        assert_eq!(
            resolved.scripts_root(),
            Path::new("/data/notas/Podcasts/guiones")
        );
    }

    #[test]
    fn un_tracker_de_solo_espacios_es_ausente() {
        let dir = tempfile::TempDir::new().expect("tmp");
        let shows = dir.path().join("shows");
        std::fs::create_dir_all(&shows).expect("mkdir");
        let profile = shows.join("demo.yaml");
        std::fs::write(&profile, "name: Demo\ntracker: \"   \"\n").expect("write profile");

        let resolver = ShowResolver {
            shows_dir: Some(shows),
            exe_dir: None,
            config_dir: None,
            default_slug: "demo".to_string(),
        };
        let err = resolver
            .resolve(Some("demo"))
            .expect_err("un tracker de solo espacios es ausente");
        assert!(
            matches!(err, PodtrackError::ProfileNoTracker { .. }),
            "esperado ProfileNoTracker, obtenido: {err:?}"
        );
    }

    #[test]
    fn render_rellena_el_numero_con_padding() {
        assert_eq!(
            render_material_template("episodio-{n:02d}", None, Some(7), "7").expect("render"),
            "episodio-07"
        );
    }

    #[test]
    fn render_sin_slug_es_error_tipado() {
        let err = render_material_template("{slug}", None, Some(1), "1").expect_err("sin slug");
        assert!(
            matches!(err, PodtrackError::EpisodeWithoutSlug { .. }),
            "err: {err:?}"
        );
    }

    #[test]
    fn render_sin_numero_es_error_tipado() {
        let err = render_material_template("{n}", Some("x"), None, "x").expect_err("sin número");
        assert!(
            matches!(err, PodtrackError::EpisodeWithoutNumber { .. }),
            "err: {err:?}"
        );
    }

    #[test]
    fn render_episode_label_prefiere_el_slug_y_cae_al_numero() {
        // Con slug, la etiqueta es el slug (nombre útil si falta el número).
        assert_eq!(
            render_episode_label(Some("episodio-siete"), Some(7)),
            "episodio-siete"
        );
        assert_eq!(
            render_episode_label(Some("episodio-siete"), None),
            "episodio-siete"
        );
        // Sin slug, cae al número; sin ninguno de los dos, a `—`.
        assert_eq!(render_episode_label(None, Some(7)), "7");
        assert_eq!(render_episode_label(Some(""), Some(7)), "7");
        assert_eq!(render_episode_label(None, None), "—");
    }

    // ── `material_dir`: regla única de la carpeta de material ──────────────────

    #[test]
    fn material_dir_con_dir_slug_renderiza_el_slug() {
        let show = resolved_with_dir("/x/y.yaml", Some("{slug}"));
        assert_eq!(
            show.material_dir(Some("la-memoria-de-valet"), Some(843))
                .expect("material_dir"),
            PathBuf::from("/x/la-memoria-de-valet")
        );
    }

    #[test]
    fn material_dir_con_dir_numerico_renderiza_con_padding() {
        let show = resolved_with_dir("/x/y.yaml", Some("episodio-{n:02d}"));
        assert_eq!(
            show.material_dir(Some("episodio-siete"), Some(7))
                .expect("material_dir"),
            PathBuf::from("/x/episodio-07")
        );
    }

    #[test]
    fn material_dir_sin_dir_cae_al_slug() {
        let sin_dir = resolved_with_dir("/x/y.yaml", None);
        assert_eq!(
            sin_dir
                .material_dir(Some("la-memoria-de-valet"), Some(843))
                .expect("material_dir"),
            PathBuf::from("/x/la-memoria-de-valet")
        );

        let dir_vacio = resolved_with_dir("/x/y.yaml", Some(""));
        assert_eq!(
            dir_vacio
                .material_dir(Some("la-memoria-de-valet"), Some(843))
                .expect("material_dir"),
            PathBuf::from("/x/la-memoria-de-valet")
        );
    }

    #[test]
    fn material_dir_rechaza_un_dir_absoluto() {
        let show = resolved_with_dir("/x/y.yaml", Some("/etc"));
        let err = show
            .material_dir(Some("la-memoria-de-valet"), Some(843))
            .expect_err("un dir absoluto debe rechazarse");
        assert!(
            matches!(err, PodtrackError::MaterialDirNotRelative { .. }),
            "err: {err:?}"
        );
    }

    #[test]
    fn material_dir_rechaza_un_dir_con_parentdir() {
        let show = resolved_with_dir("/x/y.yaml", Some("../../fuera"));
        let err = show
            .material_dir(Some("la-memoria-de-valet"), Some(843))
            .expect_err("un dir con '..' debe rechazarse");
        assert!(
            matches!(err, PodtrackError::MaterialDirNotRelative { .. }),
            "err: {err:?}"
        );
    }

    #[test]
    fn material_dir_con_plantilla_numerica_sin_numero_es_error() {
        let show = resolved_with_dir("/x/y.yaml", Some("episodio-{n:02d}"));
        let err = show
            .material_dir(Some("episodio-siete"), None)
            .expect_err("sin número");
        assert!(
            matches!(err, PodtrackError::EpisodeWithoutNumber { .. }),
            "err: {err:?}"
        );
    }

    #[test]
    fn material_dir_con_plantilla_de_slug_sin_slug_es_error() {
        let show = resolved_with_dir("/x/y.yaml", Some("episodio-{slug}"));
        let err = show.material_dir(None, Some(7)).expect_err("sin slug");
        assert!(
            matches!(err, PodtrackError::EpisodeWithoutSlug { .. }),
            "err: {err:?}"
        );
    }

    #[test]
    fn un_perfil_ilegible_se_reporta_como_no_leido() {
        // Un DIRECTORIO con nombre de perfil: `read_to_string` falla con EISDIR
        // (Is a directory) incluso ejecutando como root, a diferencia de un fichero
        // sin permisos. Reproduce el fallo de LECTURA (no de contenido).
        // Por CLI el caso alcanzable es un fichero regular sin permiso de lectura:
        // un directorio `<slug>.yaml` cae a `ProfileNotFound` porque la cascada
        // filtra con `is_file()`. Aquí se usa el directorio porque su lectura falla
        // siempre con EISDIR, incluso como root.
        let dir = tempfile::TempDir::new().expect("tmp");
        let path = dir.path().join("directorio.yaml");
        std::fs::create_dir_all(&path).expect("mkdir directorio.yaml");
        let resolver = ShowResolver {
            shows_dir: None,
            exe_dir: None,
            config_dir: None,
            default_slug: "demo".to_string(),
        };
        let err = resolver.load("directorio", &path).expect_err("debe fallar");
        assert!(
            matches!(err, PodtrackError::ProfileUnreadable { .. }),
            "err: {err:?}"
        );
        assert_eq!(err.exit_code(), 2);
        let msg = err.to_string();
        assert!(msg.contains("No se pudo leer el perfil"), "msg: {msg}");
        assert!(!msg.contains("no es válido"), "msg: {msg}");
    }
}
