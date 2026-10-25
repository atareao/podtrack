# Tasks

## 1. Spec

- [x] 1.1 Validar el change en estricto (`just spec-validate profile-error-taxonomy`).

## 2. TDD — RED

- [x] 2.1 **Unitario en `src/profile.rs`** (corregido en D4 al descubrir que la cascada filtra con
      `is_file()` y un directorio `<slug>.yaml` no llega a `load`): `load("directorio", <ruta>)`
      con una ruta que es un **directorio** (EISDIR, determinista y root-proof) → `ProfileUnreadable`,
      mensaje «No se pudo leer el perfil» y exit **2**. Verificación: fallaba hoy (salía «no es válido»).
- [x] 2.2 Test unitario en `src/error.rs` (`#[cfg(test)]`): `ProfileUnreadable` y
      `ProfileTrackerUnresolvable` salen con exit `2`; el mensaje de `ProfileUnreadable` contiene
      «No se pudo leer el perfil» y el de `ProfileTrackerUnresolvable` contiene «no se pudo
      resolver la ruta del tracker». Verificación: no compilaba (las variantes no existían).
- [x] 2.3 `cargo test --locked`: 2.1 FALLABA y 2.2 no compilaba; el resto VERDE.

## 3. TDD — GREEN

- [x] 3.1 `src/error.rs`: añadir `ProfileUnreadable` y `ProfileTrackerUnresolvable` (variantes +
      mensajes + brazo de exit `2`); refinar el rustdoc de `InvalidProfile`.
- [x] 3.2 `src/profile.rs`: re-enrutar el `map_err` de `read_to_string` (`:169`) a
      `ProfileUnreadable` y el de `absolutize` (`:204`) a `ProfileTrackerUnresolvable`.
- [x] 3.3 `cargo test --locked` y `cargo check --locked` en verde (2.1 y 2.2 pasan; el test de
      YAML mal formado `perfil_invalido_falla_nombrando_la_causa` sigue verde). 

## 4. TDD — REFACTOR

- [x] 4.1 `cargo fmt --all` / `cargo clippy --all-targets -- -D warnings` sin avisos.
- [x] 4.2 Rustdoc de las dos variantes nuevas y de `InvalidProfile` (causa de cada error).
- [x] 4.3 `just check-all` en verde.

## 5. Auditoría y verificación end-to-end

- [x] 5.1 Auditoría con `rust-reviewer`; cerrar hallazgos.
- [x] 5.2 Binario real: perfil-directorio → mensaje «No se pudo leer el perfil», exit 2; perfil
      con YAML roto → mensaje «no es válido», exit 2 (ambos comprobados con el binario).

## 6. Documentación y cierre

- [x] 6.1 `docs/DEFINITION.md`: nota de la taxonomía de errores del perfil (lectura / contenido /
      resolución), si procede.
- [x] 6.2 Fundir el delta en `openspec/specs/show-profile/spec.md`
      (`just spec-archive profile-error-taxonomy`).
- [ ] 6.3 Cerrar el plan en `plans/` (`just plan-done`).

## 7. Deuda B (documentación externa, sin release)

- [ ] 7.1 `~/.agents/skills/podcast-srt/shows/README.md`: anotar en la entrada `tracker` que una
      ruta **relativa** se resuelve contra el directorio del perfil y que se recortan los espacios
      envolventes.
