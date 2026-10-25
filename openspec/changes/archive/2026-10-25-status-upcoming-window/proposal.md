# Proposal

## Why

El bloque final de `podtrack status` («Últimos 8 episodios») muestra una ventana por
orden de parrilla invertida: mezcla publicados y no publicados y no responde a la
pregunta que el operador necesita al abrir el panel de trabajo —*¿qué está publicado
hasta ahora y qué toca publicar después?*—. Como `status` se usa a diario para decidir
qué grabar a continuación, esa ventana es ruido y obliga a leerla entera para extraer
los dos datos que de verdad importan.

## What Changes

Sustituir el bloque «Últimos 8 episodios» por una ventana de **hasta 8 filas** con las
dos mitades útiles:

1. el **último episodio publicado** (por fecha), y
2. los **7 siguientes a publicar** (no publicados, en orden ascendente de publicación).

Reglas de composición:

- **Orden:** el último publicado primero, después los próximos en orden ascendente de
  fecha. Nunca se mezclan ni se reordenan.
- **Marcas:** los próximos conservan la marca provisional `*` (y la leyenda habitual);
  el último publicado **no** lleva marca.
- **Degradación:** sin publicado → solo los próximos; sin próximos → solo el último
  publicado; sin ninguno → «No hay episodios.»
- **Cabecera del bloque** actualizada para reflejar el nuevo contenido.
- **Solo salida de texto:** el documento `--json` no cambia (ya expone `last_published`
  y `next_to_publish` por separado).

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `tracker-read`: el requisito «`status` resume la parrilla y señala el siguiente a
  publicar» se amplía para fijar la **composición de la ventana final** (último publicado
  + 7 siguientes), su orden y su degradación.

## Impact

- `src/commands/status.rs`: el cálculo del tail (`ordered.iter().rev().take(8)`) se
  sustituye por la ventana último-publicado + próximos.
- `openspec/specs/tracker-read/spec.md`: delta `MODIFIED` del requisito de `status`.
- **Sin impacto** en `--json`, en el parseo del tracker, en los perfiles de show ni en la
  escritura.
- **Divergencia deliberada** respecto del oráculo Python (ya retirado): la ventana ya no
  reproduce su «últimos 8 por orden de parrilla». Se documenta en `design.md`.
