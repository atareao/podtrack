# Tasks

## 1. Spec

- [x] 1.1 Validar el change en estricto (`just spec-validate slug-str-and-tracker-trim`) con los
      deltas de `tracker-store` y `show-profile` en verde.

## 2. TDD — RED (mitad «`tracker:`», observable)

- [x] 2.1 Test unitario en `src/profile.rs` (`#[cfg(test)]`): perfil con `tracker: " /ruta/absoluta.yaml "`
      → `ResolvedShow.tracker` es `/ruta/absoluta.yaml` sin espacios. Verificación: el test falla
      hoy (la ruta sale con espacios).
- [x] 2.2 Test unitario en `src/profile.rs`: perfil con `tracker: "   "` → error tipado de tracker
      ausente (paridad con la guarda de ausencia). Verificación: el test pasa (guarda ya existente)
      o falla si el recorte lo requiere.
- [x] 2.3 Escenario de integración (`tests/`) del spec `show-profile`: perfil con `tracker:`
      entrecomillado con espacios resuelve a la ruta limpia. Verificación: falla hoy.
- [x] 2.4 Ejecutar `cargo test --locked`: los tests nuevos de 2.1/2.3 FALLAN y los preexistentes
      siguen VERDES.

## 3. TDD — RED (mitad «slug», no observable)

- [x] 3.1 Test unitario en `src/model.rs` (`#[cfg(test)]`) del accesor canónico: slug normal
      (`la-memoria-de-valet`) → `Some("la-memoria-de-valet")`; slug `" x "` construido en caja
      blanca → `Some("x")`; sin slug / vacío / de solo espacios → `None`. Verificación: falla hoy
      (el accesor no existe).
- [x] 3.2 Test de paridad texto/JSON: para episodios con y sin slug, el slug que exponen el texto
      y el documento `--json` coinciden (mismo valor canónico). Verificación: falla hoy solo si la
      asimetría se manifiesta; si no, documenta la paridad como blindaje.

## 4. TDD — GREEN (mitad «`tracker:`»)

- [x] 4.1 `src/profile.rs::ShowResolver::load`: resolver con `raw.trim()` en lugar de `&raw`.
      Verificación: 2.1 y 2.3 en verde; `cargo test --locked` y `cargo check --locked` en verde.

## 5. TDD — GREEN (mitad «slug»)

- [x] 5.1 `src/model.rs`: añadir `Episode::slug_str(&self) -> Option<&str>` (recortado y no vacío,
      o `None`) con rustdoc. Verificación: 3.1 en verde.
- [x] 5.2 `src/commands/status.rs`, `src/commands/where_cmd.rs`, `src/commands/get.rs`: usar el
      accesor en lugar de la extracción manual. Verificación: `cargo test --locked` en verde.
- [x] 5.3 `src/json.rs`: `status_document` (next_to_publish y last_published), `get_document`
      (material), `where_document` y `episode_fields` usan el accesor. Verificación: 3.2 y los
      tests de JSON en verde.
- [x] 5.4 `cargo test --locked` y `cargo check --locked` en verde.

## 6. TDD — REFACTOR

- [x] 6.1 `cargo fmt --all` / `cargo clippy --all-targets -- -D warnings` sin avisos.
- [x] 6.2 Rustdoc del accesor y de la resolución recortada del `tracker:` (contrato y nota de
      defensa). Verificación: `just check-all` en verde.

## 7. Auditoría y verificación end-to-end

- [x] 7.1 Auditoría con `rust-reviewer`; cerrar los hallazgos.
- [x] 7.2 Verificación con el binario real: perfil de fixture con `tracker: " /ruta "` →
      `status`/`get`/`where` resuelven la ruta limpia y no dan `TrackerMissing` espurio.
- [x] 7.3 Comprobar la paridad texto/JSON del slug sobre la fixture (con slug y sin slug).

## 8. Documentación y cierre

- [x] 8.1 `docs/DEFINITION.md`: nota de que el `tracker:` se interpreta recortado (si procede).
- [x] 8.2 Fundir los deltas en `openspec/specs/{tracker-store,show-profile}/spec.md`
      (`just spec-archive slug-str-and-tracker-trim`).
- [ ] 8.3 Cerrar el plan en `plans/` (`just plan-done`).
