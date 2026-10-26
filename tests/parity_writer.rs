//! Paridad byte a byte del **writer canónico** (D1) contra el **golden congelado**.
//!
//! El writer a mano es el punto más delicado del port (reproduce un formato fijo
//! sin `serde`): este test lo compara, sobre el **mismo fixture**, con el YAML
//! canónico del golden y exige **bytes idénticos**. Cubre cabecera de
//! comentarios, orden de campos, `title` entrecomillado, `slug` presente y
//! ausente, y `number: null`.
//!
//! El golden (`tests/golden/parity_writer/*.json`) se grabó **una sola vez** con el
//! oráculo recuperado del histórico de `~/.agents` (commit `c70287f^`), que ya no
//! existe: el oráculo es el **origen** del golden, no el contrato vivo. La paridad
//! **no es en vivo** y el golden **no se regenera solo** ni se generó nunca desde el
//! binario; es evidencia histórica y candado contra regresiones.

mod common;

use std::path::Path;

use common::{Fixture, MINI_YAML, MINI_YAML_V2};
use podtrack::tracker::{load_tracker, save_tracker};

/// Guarda con la librería de podtrack y devuelve los bytes del fichero resultante.
fn lib_save(path: &Path, name: &str) -> Vec<u8> {
    let tracker = load_tracker(path).expect("load");
    let mut out = Vec::new();
    let mut err = Vec::new();
    save_tracker(path, name, &tracker, &mut out, &mut err).expect("save");
    std::fs::read(path).expect("read")
}

/// Compara byte a byte el writer de la lib con el YAML canónico del golden.
fn assert_writer_parity(yaml: &str, name: &str, golden: &str) {
    let fx = Fixture::new(yaml, "demo", name);
    let lib_bytes = lib_save(&fx.tracker, name);
    let expected = common::golden::case(golden, 0);
    expected.assert_yaml(&lib_bytes);
}

#[test]
fn writer_parity_fixture_mini() {
    assert_writer_parity(
        MINI_YAML,
        "Show de prueba",
        "parity_writer/writer_parity_fixture_mini",
    );
}

#[test]
fn writer_parity_fixture_v2() {
    assert_writer_parity(
        MINI_YAML_V2,
        "Show de prueba",
        "parity_writer/writer_parity_fixture_v2",
    );
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
