# PLAN-004 — `profile-error-taxonomy`: separar los errores de lectura, contenido y resolución del perfil

- **Estado:** terminado
- **Change OpenSpec:** `profile-error-taxonomy`
- **Ámbito:** taxonomía de errores (`src/error.rs`) + resolución de perfil (`src/profile.rs`)
- **Release objetivo:** parche `0.4.1 → 0.4.2` (no BREAKING)
- **Incluye:** deuda B (documentación externa del esquema, sin release)

## Objetivo

Que las tres causas de fallo de la resolución del perfil tengan errores tipados disjuntos y mensajes que nombren la causa real: **perfil ilegible** (fallo de lectura del fichero), **contenido mal formado** (YAML que no parsea) y **ruta del tracker irresoluble** (no se pudo absolutizar por falta de cwd). Hoy las tres confluyen en `InvalidProfile` («el perfil no es válido»), que solo es correcto para el parseo.

## Problema

`src/profile.rs::ShowResolver::load` construye `InvalidProfile` en tres sitios: `:169` (`read_to_string`, un `io::Error` de lectura), `:175` (parseo YAML, legítimo) y `:204` (`absolutize`, cuyo único fallo es `std::env::current_dir()`). Un fallo de lectura (permisos, perfil que es un directorio, symlink roto) o de entorno (cwd destruido) se reporta como «perfil inválido», engañando al operador. Deuda registrada al cerrar `profile-absolute-paths`.

## Decisiones cerradas

| # | Decisión |
|---|---|
| D1 | Dos variantes nuevas con contexto estructurado (`slug`/`path`/`reason`): `ProfileUnreadable` (lectura) y `ProfileTrackerUnresolvable` (resolución de ruta). `InvalidProfile` queda exclusivo del parseo YAML. |
| D2 | Se preserva el exit code `2` en los tres casos. Descartado mapear a `Io` (exit 1): perdería el contexto y cambiaría el código de un camino inalcanzable. |
| D3 | Nombres que describen la causa, no el síntoma. `ProfileUnreadable` es paralelo a `SrtProfileUnreadable`. |
| D4 | El test del caso ilegible usa un `<slug>.yaml` que es un directorio (`Is a directory`), portable y sin depender de permisos (que root ignoraría en CI). |
| D5 | Fuera de alcance: cascada de búsqueda, `ProfileMissingTrackerKey`, `ProfileNoTracker`, y el `io::Error` de `tracker.rs`/`material.rs` (ya usa `Io`). |

## Contrato de salida

- Perfil ilegible (directorio):
  `ERROR: No se pudo leer el perfil 'roto' (<ruta>): Is a directory (os error 21)` — exit 2.
- Perfil con YAML mal formado (sin cambios):
  `ERROR: El perfil 'roto' (<ruta>) no es válido: <causa>` — exit 2.
- Ruta del tracker irresoluble:
  `ERROR: El perfil 'demo' (<ruta>): no se pudo resolver la ruta del tracker: <causa>` — exit 2.

## Tareas

### 1. Spec
- [x] 1.1 `just spec-validate profile-error-taxonomy` en estricto.

### 2. TDD — RED
- [x] 2.1 **Unitario en `src/profile.rs`** (corregido en D4: la cascada filtra con `is_file()`, un directorio no llega a `load`): `load` con una ruta-directorio (EISDIR, root-proof) → `ProfileUnreadable`, «No se pudo leer el perfil», exit 2. Fallaba hoy.
- [x] 2.2 Unitario en `src/error.rs`: `ProfileUnreadable` y `ProfileTrackerUnresolvable` → exit 2; mensajes contienen «No se pudo leer el perfil» y «no se pudo resolver la ruta del tracker». No compila hoy.
- [x] 2.3 `cargo test --locked`: 2.1 FALLA, 2.2 no compila, resto VERDE.

### 3. TDD — GREEN
- [x] 3.1 `src/error.rs`: variantes + mensajes + brazo de exit 2; refinar rustdoc de `InvalidProfile`.
- [x] 3.2 `src/profile.rs`: re-enrutar `:169` → `ProfileUnreadable` y `:204` → `ProfileTrackerUnresolvable`.
- [x] 3.3 `cargo test --locked` y `cargo check --locked` en verde (incluido el test de YAML mal formado, que no cambia).

### 4. TDD — REFACTOR
- [x] 4.1 `cargo fmt --all` / `cargo clippy --all-targets -- -D warnings` sin avisos.
- [x] 4.2 Rustdoc de las dos variantes y de `InvalidProfile`.
- [x] 4.3 `just check-all` en verde.

### 5. Auditoría y verificación end-to-end
- [x] 5.1 Auditoría con `rust-reviewer`; cerrar hallazgos.
- [x] 5.2 Binario real: perfil-directorio → «No se pudo leer el perfil», exit 2; YAML roto → «no es válido», exit 2.

### 6. Documentación y cierre
- [x] 6.1 `docs/DEFINITION.md`: nota de la taxonomía de errores del perfil (si procede).
- [x] 6.2 `just spec-archive profile-error-taxonomy` (fundir en `openspec/specs/show-profile/spec.md`).
- [x] 6.3 `just plan-done 4`.

### 7. Deuda B (documentación externa, sin release)
- [x] 7.1 `~/.agents/skills/podcast-srt/shows/README.md`: anotar en la entrada `tracker` que una ruta relativa se resuelve contra el directorio del perfil y que se recortan los espacios envolventes.

## Release

- Rama `feature/profile-error-taxonomy` → PR a `development` → merge → PR `development → main`.
- Conventional commit `fix` (gitmoji) ⇒ parche automático del CI: `0.4.1 → 0.4.2`.
- CI: `release-prepare` (bump + CHANGELOG + tag `v0.4.2`) y `release` (crates.io + GitHub Release).

## Deuda que NO paga este plan

- Ninguna pendiente en el repo tras esto. La documentación externa de rutas (`shows/README.md`) se cubre en 7.1.
