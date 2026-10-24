//! Paridad byte a byte del **writer canónico** (D1) contra el `save_tracker` del
//! oráculo Python.
//!
//! El writer a mano es el punto más delicado del port (reproduce un formato fijo
//! sin `serde`): este test lo compara, sobre el **mismo fixture**, con el
//! `save_tracker` real del oráculo y exige **bytes idénticos**. Cubre cabecera de
//! comentarios, orden de campos, `title` entrecomillado, `slug` presente y
//! ausente, y `number: null`.
//!
//! Cuando el oráculo o Python/PyYAML no están disponibles, el test **se salta**
//! imprimiendo el motivo; con `PODTRACK_REQUIRE_ORACLE=1` **falla** en vez de
//! saltar (ver `common::require_oracle`), de modo que el *skip* nunca pasa en
//! silencio si el operador exige la comparación.

mod common;

use std::path::Path;

use common::{Fixture, MINI_YAML, MINI_YAML_V2};
use podtrack::tracker::{load_tracker, save_tracker};

/// Harness que carga el módulo del oráculo por ruta y guarda el tracker in-place
/// (`m.TRACKER_PATH = …; data = m.load_tracker(); m.save_tracker(data)`).
const WRITER_HARNESS: &str = r#"
import importlib.util, sys
from pathlib import Path
spec = importlib.util.spec_from_file_location("pt", sys.argv[1])
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
m.TRACKER_PATH = Path(sys.argv[2])
m.SHOW_NAME = sys.argv[3]
data = m.load_tracker()
m.save_tracker(data)
"#;

/// Guarda con la librería de podtrack y devuelve los bytes del fichero resultante.
fn lib_save(path: &Path, name: &str) -> Vec<u8> {
    let tracker = load_tracker(path).expect("load");
    let mut out = Vec::new();
    let mut err = Vec::new();
    save_tracker(path, name, &tracker, &mut out, &mut err).expect("save");
    std::fs::read(path).expect("read")
}

/// Compara byte a byte el writer de la lib con el del oráculo sobre el mismo YAML.
fn assert_writer_parity(yaml: &str, name: &str) {
    let Some(module) = common::require_oracle() else {
        return;
    };
    let fx = Fixture::new(yaml, "demo", name);

    let harness = fx.dir.path().join("writer_harness.py");
    std::fs::write(&harness, WRITER_HARNESS).expect("harness");

    // Dos copias con el MISMO contenido de entrada.
    let oracle_tracker = fx.dir.path().join("oracle_tracker.yaml");
    let lib_tracker = fx.dir.path().join("lib_tracker.yaml");
    std::fs::write(&oracle_tracker, yaml).expect("seed oracle");
    std::fs::write(&lib_tracker, yaml).expect("seed lib");

    // (1) writer del oráculo.
    let output = common::run_python_harness(&harness, &module, &oracle_tracker, name);
    assert!(
        output.status.success(),
        "el oráculo falló: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // (2) writer de la lib.
    let lib_bytes = lib_save(&lib_tracker, name);

    // (3) bytes idénticos.
    let oracle_bytes = std::fs::read(&oracle_tracker).expect("read oracle");
    assert_eq!(
        String::from_utf8_lossy(&oracle_bytes),
        String::from_utf8_lossy(&lib_bytes),
        "el writer de la lib difiere del oráculo byte a byte"
    );
    assert_eq!(
        oracle_bytes, lib_bytes,
        "el writer de la lib difiere del oráculo byte a byte"
    );
}

#[test]
fn writer_parity_fixture_mini() {
    assert_writer_parity(MINI_YAML, "Show de prueba");
}

#[test]
fn writer_parity_fixture_v2() {
    assert_writer_parity(MINI_YAML_V2, "Show de prueba");
}

/// Fija (sin depender del oráculo) las formas canónicas que el test de paridad
/// compara: cabecera, orden de campos, `title` entrecomillado, `slug` presente y
/// ausente, y `number: null`. Así la cobertura queda garantizada incluso cuando
/// el oráculo está ausente y el comparador se salta.
#[test]
fn writer_forma_canonica_cubre_slug_y_number_null() {
    let fx = Fixture::new(MINI_YAML_V2, "demo", "Show de prueba");
    let bytes = lib_save(&fx.tracker, "Show de prueba");
    let text = String::from_utf8(bytes).expect("utf8");

    // Cabecera de comentarios literal.
    assert!(text.starts_with("# podcast-tracker.yaml\n"));
    assert!(text.contains("# Estado del podcast \"Show de prueba\"\n"));
    assert!(text.contains("# Temporadas: empiezan el 1 de septiembre\n"));

    // Orden de campos top-level.
    let i_season = text.find("\nseason: ").expect("season");
    let i_start = text.find("season_start: ").expect("season_start");
    let i_dur = text.find("episode_duration_minutes: ").expect("duration");
    let i_days = text
        .find("publish_days: [Monday, Thursday]")
        .expect("publish_days");
    let i_eps = text.find("\nepisodes:\n").expect("episodes");
    assert!(i_season < i_start && i_start < i_dur && i_dur < i_days && i_days < i_eps);

    // `number: null` y `slug` presente.
    assert!(text.contains("  - number: null\n"));
    assert!(text.contains("    slug: sin-numero\n"));

    // `title` entrecomillado.
    assert!(text.contains("    title: \"Sin número\"\n"));

    // Solo DOS episodios traen `slug` (200 y el de number null); el 202 no.
    assert_eq!(text.matches("    slug: ").count(), 2, "texto:\n{text}");

    // Y el orden por fecha: 200 (2025-09-04) < null (2026-09-07) < 202 (2026-09-10).
    let i200 = text.find("number: 200").expect("200");
    let inull = text.find("number: null").expect("null");
    let i202 = text.find("number: 202").expect("202");
    assert!(i200 < inull && inull < i202);
}
