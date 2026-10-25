# Tasks

## 1. Spec

- [x] 1.1 Validar el change en estricto (`just spec-validate profile-absolute-paths`).

## 2. TDD — RED

- [x] 2.1 Test unitario en `src/profile.rs` (`#[cfg(test)]`): helper léxico de absolutización.
      Casos: absoluto limpio → identidad; `..`/`.` → colapsados; relativo + base absoluta →
      unión absoluta; relativo + base relativa → resuelto contra el CWD.
- [x] 2.2 Escenario `show-profile`: perfil con `tracker:` **relativo** → `ResolvedShow.tracker`
      absoluto, resuelto contra el directorio del perfil.
- [x] 2.3 Escenario `show-profile`: perfil con `tracker:` absoluto y limpio → idéntico.
- [x] 2.4 Escenario `show-profile`: `tracker:` con `.`/`..` → normalizado.
- [x] 2.5 Escenario `show-profile`: `tracker:` relativo a un fichero **inexistente** → ruta
      absoluta igualmente, sin crear nada, y el error de tracker ausente nombra la absoluta.
- [x] 2.6 `scripts_root()` con `tracker:` sin directorio → NO es `"."`.
- [x] 2.7 Integración (`tests/parity_read.rs`): con perfil de `tracker:` relativo, `where` y
      `get` imprimen ruta **absoluta** (y `status`, el bloque «Siguiente a publicar»).
- [x] 2.8 Ejecutar `cargo test --locked`: los tests nuevos FALLAN y los preexistentes siguen
      VERDES.

## 3. TDD — GREEN

- [x] 3.1 `src/profile.rs`: normalizar `ResolvedShow.tracker` a absoluto en `ShowResolver::load`.
- [x] 3.2 `src/profile.rs`: retirar el fallback `PathBuf::from(".")` de `scripts_root()`.
- [x] 3.3 `cargo test --locked` y `cargo check --locked` en verde.

## 4. TDD — REFACTOR

- [x] 4.1 `cargo fmt --all` / `cargo clippy --all-targets -- -D warnings` sin avisos.
- [x] 4.2 Documentar el helper y `scripts_root()` (rustdoc, `# Panics`/contrato de absoluto).
- [x] 4.3 `just check-all` en verde.

## 5. Auditoría y verificación

- [x] 5.1 Auditoría con `rust-reviewer`; cerrar los hallazgos.
- [x] 5.2 Verificación end-to-end con el binario real sobre fixture de `tracker:` relativo:
      `where`, `get`, `get --json`, `status`, y `--show` con `PODTRACK_SHOWS_DIR` absoluto.
- [x] 5.3 Comprobar que el JSON de `get` con perfil relativo lleva `material` absoluto.

## 6. Documentación y cierre

- [x] 6.1 `docs/DEFINITION.md`: nota de que la ruta del tracker se normaliza a absoluta.
- [x] 6.2 Fundir los deltas en `openspec/specs/{show-profile,tracker-read}/spec.md`
      (`just spec-archive profile-absolute-paths`).
- [x] 6.3 Cerrar el plan en `plans/`.
