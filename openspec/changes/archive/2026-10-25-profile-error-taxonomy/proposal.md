# Proposal

## Why

`src/profile.rs::ShowResolver::load` mapea **dos fallos de entorno** al error `InvalidProfile`,
cuyo significado es «el **contenido** del perfil no es válido»:

1. **Fallo de lectura del fichero de perfil** (`std::fs::read_to_string`, línea ~169): un
   permiso denegado, un perfil que es un directorio o un symlink roto se reporta como «perfil
   inválido», cuando el fichero no se ha llegado a leer.
2. **Fallo al resolver la ruta del `tracker:` a absoluta** (`std::env::current_dir()`, dentro de
   `absolutize`, línea ~204): si el directorio de trabajo no se puede determinar, un perfil
   perfectamente correcto se reporta como inválido.

El **único** uso legítimo de `InvalidProfile` es el parseo del YAML (línea ~175): ahí el
contenido **sí** es inválido. Confundir las tres cosas en un solo error hace que un operador que
ve «perfil inválido» no sepa si el problema es el fichero (lectura), el entorno (cwd) o el
contenido (YAML) — y los mensajes de este binario son contrato. Deuda registrada al cerrar
`profile-absolute-paths`.

## What Changes

- Se **separa la taxonomía de errores del perfil** en tres casos disjuntos:
  - **Perfil ilegible** (no se pudo leer el fichero): error tipado nuevo
    `ProfileUnreadable { slug, path, reason }` (exit 2), mensaje
    `ERROR: No se pudo leer el perfil '{slug}' ({path}): {reason}` — paralelo al ya existente
    `SrtProfileUnreadable`.
  - **Contenido mal formado** (YAML que no parsea): `InvalidProfile { slug, path, reason }`
    (exit 2), sin cambios de mensaje; se refina su documentación a «contenido inválido».
  - **Ruta del tracker irresoluble** (no se pudo absolutizar por falta de *cwd*): error tipado
    nuevo `ProfileTrackerUnresolvable { slug, path, reason }` (exit 2), mensaje
    `ERROR: El perfil '{slug}' ({path}): no se pudo resolver la ruta del tracker: {reason}`.
- **Sin cambio de exit code**: los tres casos son exit 2, como hoy.
- **Sin cambio observable** en los caminos hoy alcanzables: el parseo YAML sigue dando el mismo
  mensaje y código; los dos fallos de entorno (hoy inalcanzables salvo cwd destruido o permisos
  raros) pasan de un mensaje engañoso de «perfil inválido» a uno que nombra el problema real.
- **No es BREAKING**: no se retira ningún error; se añaden dos variantes y se re-enrutan dos
  `map_err`.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `show-profile`: el requisito «El esquema del perfil no se reinterpreta» se **extiende** para
  distinguir explícitamente **perfil ilegible** (fallo de lectura del fichero) de **perfil mal
  formado** (contenido inválido), con un escenario propio para el fallo de lectura. Nada más
  cambia de comportamiento.

## Impact

- `src/error.rs`: dos variantes nuevas (`ProfileUnreadable`, `ProfileTrackerUnresolvable`) y su
  mapeo a exit 2; se retoca la documentación de `InvalidProfile`.
- `src/profile.rs`: `ShowResolver::load` re-enruta el `map_err` de `read_to_string` y el de
  `absolutize`.
- `openspec/specs/show-profile/spec.md`: delta.
- `tests/`: escenario de perfil ilegible (fichero que es un directorio, o permisos) con el
  mensaje y exit 2.
- `docs/DEFINITION.md`: si procede, una nota sobre la taxonomía de errores del perfil.
- **Sin divergencia** con el oráculo: el Python no distinguía estos casos (y ya no existe,
  `retire-python`); el contrato de exit code se mantiene (2).
- **Fuera del repo (documentación externa):** el esquema `shows/README.md` —documentación de la
  deuda B de esta ronda— se anota aparte, sin release en podtrack.

## Criterio de cierre

1. Un perfil que no se puede leer falla con `ProfileUnreadable` (exit 2) y un mensaje que dice
   «No se pudo leer el perfil», no «no es válido».
2. Un perfil con YAML mal formado sigue fallando con `InvalidProfile` (exit 2) y el mismo
   mensaje que hoy.
3. `just check-all` en verde.
