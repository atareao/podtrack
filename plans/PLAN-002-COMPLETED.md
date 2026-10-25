# PLAN-002 — `profile-absolute-paths`: el tracker del perfil siempre absoluto

- **Estado:** terminado
- **Change OpenSpec:** `profile-absolute-paths`
- **Ámbito:** resolución de perfil (`src/profile.rs`) + rutas de material de `status`/`get`/`where`

## Objetivo

Que `ResolvedShow.tracker` —y por tanto `<scripts.root>`— sea **siempre absoluto y léxicamente limpio**, declare el perfil lo que declare. Con eso, el `SHALL` de `get` («la ruta del material es absoluta») deja de depender de que el `tracker:` del perfil esté escrito en absoluto, que era la deuda registrada al cerrar `get-shows-slug-and-path`.

## Problema

`profile.rs::ShowResolver::load` copia el `tracker:` del perfil a `PathBuf` **sin normalizar**. `scripts_root()` es `tracker.parent()` con un fallback a `PathBuf::from(".")`. Si el perfil declara `tracker: guiones/podcast-tracker.yaml` (relativo), o si `PODTRACK_SHOWS_DIR` es relativo, la resolución produce un tracker relativo y las rutas de material salen relativas o con `"."`. Hoy no se nota porque los dos perfiles reales declaran un `tracker:` absoluto y limpio.

## Decisiones cerradas

| # | Decisión |
|---|---|
| D1 | Se normaliza en el **constructor** (`ShowResolver::load`), no en cada consumidor: un solo punto cubre `status`, `get`, `where` y lo futuro. |
| D2 | `tracker:` absoluto → se usa tal cual, **normalizado léxicamente** (sin `.` ni `..`). |
| D3 | `tracker:` relativo → se resuelve contra el **directorio del fichero de perfil**. Si ese directorio fuera relativo, se absolutiza contra `std::env::current_dir()`. |
| D4 | La resolución es **puramente léxica**: sin E/S, sin exigir existencia, sin resolver symlinks, sin crear nada. Se descarta `canonicalize()` a propósito (cambiaría la ruta observada de un absoluto que pase por un symlink y rompería la paridad observable). |
| D5 | `scripts_root()` pierde el fallback `"."`: con tracker absoluto el `parent()` existe siempre salvo la raíz `/`, y ese caso cae a la propia ruta absoluta, no a `.`. |
| D6 | `srt-stage` queda fuera de alcance (resuelve el perfil sin exigir `tracker:` y no calcula rutas de material). Se verifica que sus tests siguen verdes. |

## Contrato de salida

Con perfil de `tracker:` **relativo**:

```
$ podtrack where la-memoria-de-valet
<base-absoluta>/guiones/la-memoria-de-valet no existe

$ podtrack get la-memoria-de-valet
  ...
  Slug:      la-memoria-de-valet
  Ruta:      <base-absoluta>/guiones/la-memoria-de-valet
  ...
```

Con perfil de `tracker:` **absoluto y limpio** (los dos perfiles reales): salida **byte a byte** idéntica a la de hoy (la normalización es la identidad).

## Tareas

### 1. Spec
- [x] 1.1 `just spec-validate profile-absolute-paths` en estricto.

### 2. TDD — RED
- [x] 2.1 Unitarios del helper léxico de absolutización en `src/profile.rs`: absoluto limpio → identidad; `.`/`..` colapsados; relativo + base absoluta; relativo + base relativa → CWD; `..` sobre la raíz.
- [x] 2.2 `tracker:` relativo → `ResolvedShow.tracker` absoluto contra el directorio del perfil.
- [x] 2.3 `tracker:` absoluto y limpio → idéntico.
- [x] 2.4 `tracker:` con `.`/`..` → normalizado.
- [x] 2.5 `tracker:` relativo a fichero inexistente → ruta absoluta igualmente, sin crear nada, y el error `TrackerMissing` nombra la absoluta.
- [x] 2.6 `scripts_root()` con `tracker:` sin directorio → NO es `"."`.
- [x] 2.7 Integración `tests/parity_read.rs`: `where`, `get` y el bloque «Siguiente a publicar» de `status` imprimen ruta absoluta con perfil relativo.
- [x] 2.8 `cargo test --locked`: los nuevos FALLAN, los preexistentes VERDES.

### 3. TDD — GREEN
- [x] 3.1 Normalizar `ResolvedShow.tracker` a absoluto en `ShowResolver::load`.
- [x] 3.2 Retirar el fallback `PathBuf::from(".")` de `scripts_root()`.
- [x] 3.3 `cargo test --locked` y `cargo check --locked` en verde.

### 4. TDD — REFACTOR
- [x] 4.1 `cargo fmt --all` y `cargo clippy --all-targets -- -D warnings` sin avisos.
- [x] 4.2 Rustdoc del helper y de `scripts_root()` (contrato «absoluto»).
- [x] 4.3 `just check-all` en verde.

### 5. Auditoría y verificación
- [x] 5.1 Auditoría con `rust-reviewer`; cerrar hallazgos.
- [x] 5.2 End-to-end con el binario real sobre fixture de `tracker:` relativo: `where`, `get`, `get --json`, `status`, y `--show` con `PODTRACK_SHOWS_DIR` absoluto.
- [x] 5.3 Verificar que `get --json` con perfil relativo lleva `material` absoluto y que los 7 documentos JSON siguen idénticos para perfil absoluto.

### 6. Documentación y cierre
- [x] 6.1 `docs/DEFINITION.md`: nota de que la ruta del tracker se normaliza a absoluta.
- [x] 6.2 `just spec-archive profile-absolute-paths` (fundir deltas en `openspec/specs/{show-profile,tracker-read}/spec.md`).
- [x] 6.3 `just plan-done 2`.

## Deuda que NO paga este plan

- Extracción del `slug` duplicada en cinco sitios y asimetría del `trim` (D8 de `get-shows-slug-and-path`); helper futuro: `Episode::slug_str()`.
- El esquema del perfil (`~/.agents/skills/podcast-srt/shows/README.md`, fuera del repo) no documenta aún la semántica de la ruta relativa; se anota fuera de este PR.

## Criterio de cierre

Para un perfil con `tracker:` relativo, `where`, `get` y el bloque «Siguiente a publicar» de `status` imprimen ruta **absoluta**; `scripts_root()` no devuelve nunca `"."`; y para un perfil de `tracker:` absoluto limpio la salida no cambia un byte.
