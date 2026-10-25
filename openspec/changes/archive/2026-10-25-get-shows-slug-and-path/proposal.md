# Proposal

## Why

`podtrack get` muestra el detalle de un episodio, pero omite los dos datos que identifican
su material: el `slug` (la identidad estable del episodio) y la ruta de su carpeta de
material. Hoy quien quiere abrir, mover o inspeccionar el material de un episodio tiene que
encadenar `get` con `where` —que imprime la ruta, pero no el resto del detalle— o mirar
`status`, que solo habla del siguiente a publicar. `get` es el comando «cuéntame todo de
este episodio» y le falta justamente la identidad y la localización.

## What Changes

- `podtrack get` SHALL añadir dos líneas al bloque de texto: `Slug:` y `Ruta:`.
- `Slug:` SHALL mostrar el `slug` del episodio; `Ruta:` SHALL mostrar la **ruta absoluta**
  de su carpeta de material (`<scripts.root>/<slug>`).
- La ruta SHALL mostrarse **sin comprobar si existe**: `get` describe el episodio, no
  audita el sistema de ficheros (a diferencia de `where`, que sí informa de la existencia).
- Si el episodio **no tiene `slug`**, ambas líneas SHALL mostrar `—`.
- Las cinco líneas actuales del bloque SHALL permanecer **byte a byte** idénticas; las dos
  nuevas SHALL añadirse al final, antes del separador de cierre.
- `podtrack get --json` SHALL añadir el campo `material` (aditivo) al objeto `episode`, con
  la ruta absoluta del material o `null` si el episodio no tiene `slug`.
- `schema_version` **NO SHALL** cambiar: la adición es retrocompatible.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `tracker-read`: el requisito «`get` muestra el detalle de un episodio» se amplía para
  fijar el `slug` del episodio y la **ruta absoluta** de su material, con `—` cuando no hay
  `slug`.
- `json-output`: el requisito «El documento identifica el show y su carga útil» se amplía
  para fijar el campo `material` del documento de `get`.

## Impact

- `src/commands/get.rs`: dos líneas nuevas en la salida de texto, calculadas con
  `ctx.show.scripts_root().join(&slug)` (mismo criterio que `status` y `where`).
- `src/json.rs`: `get_document` añade `material` al objeto `episode`. El helper compartido
  pasa a ser `episode_fields(episode) -> serde_json::Map<String, Value>` (cierre del hallazgo
  #1 del auditor: elimina el `if let` que podía descartar `material` en silencio) sin cambiar
  el JSON observable de `list`, `list-range`, `add`, `update`, `publish`, `insert` y `move`
  (verificado byte a byte); `material` sigue siendo exclusivo de `get`.
- `tests/parity_read.rs` y `tests/json_output.rs`: casos nuevos (texto y `--json`).
- `openspec/specs/tracker-read/spec.md` y `openspec/specs/json-output/spec.md`: deltas
  `MODIFIED` de sus requisitos.
- **Sin impacto** en el parseo del tracker, en los perfiles de show, en la escritura ni en
  el resto de los comandos de lectura.
- **Divergencia deliberada** respecto del oráculo Python (ya retirado): esta no es una
  línea del oráculo, es una extensión nuestra. Se documenta en `design.md`.
