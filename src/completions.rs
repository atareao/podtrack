//! Autocompletado de shell: los candidatos dinámicos y el script de registro.
//!
//! El script que emite `completions` **no** lleva candidatos incrustados: llama al binario y es el
//! propio binario quien resuelve el completado con el motor de `clap_complete` — estático (los
//! subcomandos, los flags y los valores de un `value_parser`) y dinámico (lo de este módulo).
//!
//! El binario y el ejecutable del script se nombran **literales** (`podtrack`), nunca por su ruta:
//! así el script es determinista y se puede congelar en un test.

use std::ffi::OsStr;
use std::io::Write;

use clap::ValueEnum;
use clap_complete::engine::CompletionCandidate;
use clap_complete::env::{Bash, EnvCompleter, Fish, Shells, Zsh};

use crate::model::Episode;
use crate::profile::ShowResolver;
use crate::tracker;

/// Nombre con el que el script invoca al binario.
pub const BIN: &str = "podtrack";

/// Variable de entorno que activa el protocolo de completado.
const VAR: &str = "COMPLETE";

/// Shells con script de registro soportado, en el mismo orden que anuncia el subcomando.
///
/// Es la lista que gobierna el protocolo de entorno (`CompleteEnv`): así no puede atender un shell
/// que `completions` rechaza.
pub const SHELLS: Shells<'static> = Shells(&[&Bash, &Zsh, &Fish]);

/// Shell con script de registro soportado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Shell {
    /// Bash.
    Bash,
    /// Zsh.
    Zsh,
    /// Fish.
    Fish,
}

impl Shell {
    /// Escribe en `buf` el script de registro del completado para este shell.
    ///
    /// El script se emite con `clap_complete`: al evaluarlo, el shell registra el completado y
    /// llama a `BIN` con el protocolo en `VAR`.
    ///
    /// # Errors
    ///
    /// Falla solo si la escritura en `buf` falla.
    pub fn write_registration(self, buf: &mut impl Write) -> std::io::Result<()> {
        self.completer().write_registration(VAR, BIN, BIN, BIN, buf)
    }

    /// Adaptador de `clap_complete` para este shell.
    fn completer(self) -> &'static dyn EnvCompleter {
        match self {
            Self::Bash => &Bash,
            Self::Zsh => &Zsh,
            Self::Fish => &Fish,
        }
    }
}

/// Candidatos del `target` de un comando: por cada episodio, su número y su slug.
///
/// El orden es el del tracker (orden de fichero); dentro de cada episodio va primero el número.
/// Solo se ofrecen los valores que empiezan por `current`, y sin duplicados.
#[must_use]
pub fn targets(episodes: &[Episode], current: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for episode in episodes {
        if let Some(number) = episode.number {
            push_unique(&mut out, number.to_string(), current);
        }
        if let Some(slug) = episode.slug_str() {
            push_unique(&mut out, slug.to_string(), current);
        }
    }
    out
}

/// Añade `value` si empieza por `current` y no está ya en `out`.
fn push_unique(out: &mut Vec<String>, value: String, current: &str) {
    if value.starts_with(current) && !out.contains(&value) {
        out.push(value);
    }
}

/// Completador del `target` para el motor: resuelve el show del entorno y sus episodios.
///
/// Un completado nunca debe romper el prompt del shell: si el show no resuelve, el tracker no
/// existe o el YAML no carga, devuelve una lista **vacía** y en silencio.
#[must_use]
pub fn target_completer(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return Vec::new();
    };
    let Ok(show) = ShowResolver::from_env(crate::profile::exe_dir()).resolve(None) else {
        return Vec::new();
    };
    let Ok(tracker) = tracker::load_tracker(&show.tracker) else {
        return Vec::new();
    };
    candidates(targets(&tracker.episodes, current))
}

/// Completador del valor de `--show` para el motor: los perfiles de la cascada.
///
/// Reutiliza `ShowResolver::available_slugs` —la misma fuente que el mensaje de perfiles
/// disponibles— para no duplicar el recorrido de la cascada, el deduplicado ni el orden; aquí solo
/// se filtra por el prefijo ya escrito.
#[must_use]
pub fn show_completer(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return Vec::new();
    };
    let slugs = ShowResolver::from_env(crate::profile::exe_dir()).available_slugs();
    candidates(
        slugs
            .into_iter()
            .filter(|slug| slug.starts_with(current))
            .collect(),
    )
}

/// Envuelve los candidatos de texto en los del motor.
fn candidates(values: Vec<String>) -> Vec<CompletionCandidate> {
    values.into_iter().map(CompletionCandidate::new).collect()
}

#[cfg(test)]
mod tests {
    use super::{SHELLS, Shell, targets};
    use crate::tracker::load_tracker;

    /// Tracker de tres episodios: con número y slug, sin número y sin slug.
    const TRACKER: &str = "\
season: 9
publish_days: [Monday, Thursday]
episodes:
  - number: 200
    slug: primer-episodio
    season: 9
    date: 2026-09-03
    title: Primero
    status: published
  - number: null
    slug: sin-numero
    season: 9
    date: 2026-09-07
    title: Sin número
    status: scripted
  - number: 202
    season: 9
    date: 2026-09-10
    title: Tercero
    status: recorded
";

    fn episodios() -> Vec<crate::model::Episode> {
        let dir = tempfile::TempDir::new().expect("tmp");
        let path = dir.path().join("tracker.yaml");
        std::fs::write(&path, TRACKER).expect("write");
        load_tracker(&path).expect("load").episodes
    }

    #[test]
    fn targets_ofrece_numero_y_slug_en_orden_de_fichero() {
        assert_eq!(
            targets(&episodios(), ""),
            vec!["200", "primer-episodio", "sin-numero", "202"]
        );
    }

    #[test]
    fn targets_filtra_por_el_prefijo() {
        assert_eq!(targets(&episodios(), "pri"), vec!["primer-episodio"]);
        assert_eq!(targets(&episodios(), "20"), vec!["200", "202"]);
        assert!(targets(&episodios(), "zzz").is_empty());
    }

    /// El protocolo de entorno no debe atender un shell que el subcomando rechaza.
    #[test]
    fn la_lista_de_shells_cubre_justo_el_enum() {
        for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
            let name = shell.completer().name();
            assert!(
                SHELLS.0.iter().any(|c| c.name() == name),
                "falta {name} en SHELLS"
            );
        }
        assert_eq!(SHELLS.0.len(), 3, "SHELLS anuncia shells no soportados");
    }
}
