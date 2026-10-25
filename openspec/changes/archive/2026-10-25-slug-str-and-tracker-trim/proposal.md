# Proposal

## Why

Quedan **dos asimetrías del `trim`** en el binario —ambas registradas como deuda tras los
changes `get-shows-slug-and-path` (design.md, D8) y `profile-absolute-paths`— y son de la
misma familia: el código compara o lee un valor «recortado» en un sitio y «sin recortar» en
otro, de modo que dos caminos que deberían coincidir pueden divergir.

1. **La extracción del `slug` está duplicada en varios sitios y con `trim` asimétrico.** El
   texto de `status` (`next_to_publish_block`), `where` y `get` recorta el slug (`.trim()`)
   antes de exponerlo; los documentos JSON (`status_document`, `where_document`,
   `episode_fields`, el `material` de `get_document`) NO lo recortan (`s.as_str()` a secas).
   Hoy no se manifiesta porque el cargador ya normaliza el slug, pero la asimetría es real y
   frágil: un futuro camino que construya un `Episode` en memoria sin pasar por la carga
   haría que texto y JSON discrepasen.
2. **El `tracker:` del perfil se compara recortado pero se resuelve sin recortar.** En
   `ShowResolver::load` la guarda usa `raw.trim()` (`¿vacío?`, `¿none?`) pero la ruta se
   construye con `Path::new(&raw)` a partir del valor **sin** recortar. Un perfil con
   `tracker: " /a/b "` (entrecomillado, que preserva los espacios) pasa la guarda —no está
   vacío— y produce una ruta con espacios alrededor, que no existe → `TrackerMissing`
   apuntando a una ruta contaminada. **Este sí es observable y testeable hoy.**

Ambas se pagan ahora, en un único change acotado, antes de publicar la release de parche
`0.4.1`.

## What Changes

- Se extrae un **accesor canónico del slug en el modelo** (`Episode`): el slug **recortado y
  no vacío, o nada**. Las extracciones de `status`, `where`, `get` y los documentos JSON
  pasan a usarlo, de modo que texto y JSON exponen **el mismo** valor del slug y desaparece
  la asimetría del `trim`.
- El `tracker:` del perfil SHALL interpretarse **sin espacios envolventes**: la ruta se
  resuelve a partir del valor **recortado**, coherente con la guarda que ya lo compara
  recortado. Un `tracker: " /a/b "` SHALL resolver a la misma ruta que `tracker: /a/b`.
- **Sin cambios de contrato externo observables** salvo el caso del `tracker:` con espacios
  envolventes: el formato de salida, las claves del JSON, los códigos de error y los
  mensajes no cambian.
- **No es BREAKING**: no se retira ni renombra nada; no se toca el parseo del tracker, el
  writer, ni la resolución de rutas.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `tracker-store`: se **añade** un requisito que fija la **lectura canónica del slug** en el
  modelo —el slug expuesto es el recortado y no vacío, o nada—, del que depende que todos
  los consumidores (texto y JSON) coincidan. Sus escenarios son de **modelo** (caja blanca):
  el cargador ya garantiza el invariante para datos cargados, de modo que este requisito
  **no cambia la salida observable del CLI**; se registra como contrato defensivo del modelo.
- `show-profile`: se **añade** un requisito que fija que el valor de `tracker:` se interpreta
  **sin espacios envolventes** antes de resolverlo a ruta absoluta. **Sí** cambia la salida
  observable en el caso del `tracker:` entrecomillado con espacios.

## Impact

- `src/model.rs`: nuevo accesor `Episode::slug_str()` (rustdoc + tests unitarios `#[cfg(test)]`).
- `src/commands/status.rs`, `src/commands/where_cmd.rs`, `src/commands/get.rs`: sustituyen la
  extracción manual por el accesor.
- `src/json.rs`: `status_document`, `get_document` (el `material`), `where_document` y
  `episode_fields` usan el accesor (unificación del `trim`).
- `src/profile.rs`: `ShowResolver::load` resuelve la ruta desde `raw.trim()`; test unitario del
  caso entrecomillado.
- `openspec/specs/tracker-store/spec.md` y `openspec/specs/show-profile/spec.md`: deltas.
- `tests/` de paridad: escenario de perfil con `tracker: " /ruta "` que resuelve a la ruta
  limpia; paridad texto/JSON del slug.
- `docs/DEFINITION.md`: nota de que el `tracker:` se interpreta recortado (si procede).
- **Sin divergencia** respecto de la implementación de referencia: el oráculo ya no existe
  (`retire-python`) y el caso del slug cargado nunca diverge; el `tracker:` entrecomillado con
  espacios no lo ejercita ningún perfil real, así que la paridad observable se mantiene.

## Criterio de cierre

1. Un perfil con `tracker: " /data/notas/Podcasts/guiones/podcast-tracker.yaml "` resuelve el
   tracker a `/data/notas/Podcasts/guiones/podcast-tracker.yaml` (sin espacios).
2. El slug que exponen las salidas de texto (`status`, `where`, `get`) y los documentos JSON
   (`status`, `where`, `get`) es **el mismo** valor canónico para un mismo episodio.
3. `just check-all` en verde.
