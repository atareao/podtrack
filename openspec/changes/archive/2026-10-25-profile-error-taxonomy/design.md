# Design

## Context

Ver `proposal.md` — *Why*. Estado actual:

- `src/profile.rs::ShowResolver::load` construye `InvalidProfile { slug, path, reason }` en tres
  sitios:
  - `:169` — `std::fs::read_to_string(path).map_err(...)`: fallo de **lectura** (`io::Error`).
  - `:175` — `serde_yaml::from_str(&text).map_err(...)`: fallo de **parseo** del contenido.
  - `:204` — `absolutize(Path::new(raw.trim()), base_dir).map_err(...)`: `absolutize` devuelve
    `std::io::Result`, y su único error es `std::env::current_dir()` cuando `base_dir` es
    relativa (`src/profile.rs:298-308`).
- `InvalidProfile` (exit 2): `ERROR: El perfil '{slug}' ({path}) no es válido: {reason}`.
- Ya existe un error paralelo para la otra ruta (`srt-stage`): `SrtProfileUnreadable { path,
  reason }` (exit 2), `ERROR: No se pudo leer el perfil {path}: {reason}`.
- El test `tests/parity_profile.rs::perfil_invalido_falla_nombrando_la_causa` fija el caso de
  YAML mal formado → exit 2 + nombra el perfil. **No cambia.**

## Goals / Non-Goals

**Goals**

- Que las tres causas de fallo del perfil tengan errores tipados **disjuntos** y mensajes que
  nombren la causa real (lectura / contenido / resolución de ruta).
- Mantener el contrato de **exit code** en `2` para las tres.
- Blindar el nuevo caso con un test de integración (perfil que no se puede leer).

**Non-Goals**

- No se toca la cascada de búsqueda del perfil (`ProfileNotFound`) ni `ProfileMissingTrackerKey`
  ni `ProfileNoTracker`.
- No se cambia ningún mensaje de los caminos hoy alcanzables (YAML mal formado, clave ausente,
  `tracker: none`).
- No se toca el `io::Error` de `tracker.rs`/`material.rs` (ese ya usa `Io` correctamente).

## Decisions

### D1 — Dos errores nuevos, un `map_err` re-enrutado cada uno

- `ProfileUnreadable { slug: String, path: PathBuf, reason: String }` — exit **2**.
  Mensaje: `ERROR: No se pudo leer el perfil '{slug}' ({path}): {reason}`. Paralelo a
  `SrtProfileUnreadable` (consistencia de taxonomía).
- `ProfileTrackerUnresolvable { slug: String, path: PathBuf, reason: String }` — exit **2**.
  Mensaje: `ERROR: El perfil '{slug}' ({path}): no se pudo resolver la ruta del tracker: {reason}`.

`InvalidProfile` queda **exclusivo del parseo** del YAML (`:175`); su rustdoc pasa a «perfil mal
formado (contenido inválido)».

### D2 — Se preserva el exit code 2 en los tres casos

`exit_code()` mapea `ProfileUnreadable` y `ProfileTrackerUnresolvable` al mismo brazo que
`InvalidProfile` (`2`). Ningún camino cambia de código de salida: el oráculo ya no existe
(`retire-python`) pero el contrato de códigos del repo es explícito y se mantiene.

**Alternativa descartada:** mapear el fallo de entorno a `Io` (exit 1). Habría perdido el
contexto estructurado (`slug`/`path`) —`Io` es una cadena suelta— y habría cambiado el exit code
de un camino hoy inalcanzable. Dos variantes tipadas con contexto son más fieles al estilo del
repo.

### D3 — `ProfileUnreadable` y no `ProfileInvalid` genérico

Se reutiliza el patrón ya establecido por `SrtProfileUnreadable` para «no se pudo leer». El nombre
`ProfileUnreadable` describe la causa (el fichero no se lee), no el síntoma (perfil inválido).

### D4 — Test del caso ilegible: ruta-directorio pasada a `load`, no vía `--show`

**Corrección durante el RED.** La cascada de `ShowResolver::resolve` filtra los candidatos con
`path.is_file()` (`src/profile.rs:132` y `:152`), de modo que un **directorio** llamado
`<slug>.yaml` **no** llega a `load()`: cae a `ProfileNotFound`. El escenario de la spec se corrigió
para no prometer esa alcanzabilidad por `--show`.

La reproducción **determinista y root-proof** del fallo de lectura es un test **unitario** en
`src/profile.rs` que llama directamente a `load("slug", <ruta-directorio>)`: `read_to_string` sobre
un directorio falla siempre con `Is a directory` (EISDIR), incluso ejecutando como `root` — a
diferencia de un fichero sin permisos, que `root` leería igualmente. Por CLI, el caso real es un
fichero **regular** sin permiso de lectura (y una carrera TOCTOU), difícil de reproducir sin
depender del usuario; por eso la validación del escenario es a nivel de resolución.

## Risks / Trade-offs

- *Los dos caminos nuevos son prácticamente inalcanzables* (cwd destruido; perfil-directorio solo
  creado a propósito) → asumido: el valor es que el mensaje diga la verdad y que la taxonomía sea
  correcta de cara al futuro. El test de D4 hace alcanzable y verificable el caso de lectura.
- *Un consumidor que ramifique por `InvalidProfile { .. }`* → ninguno: `InvalidProfile` solo se
  construye en `profile.rs` y no se hace *matching* por variante en ningún sitio (verificado por
  grep). El mapeo de exit code por brazo se actualiza.

## Open Questions

Ninguna.
