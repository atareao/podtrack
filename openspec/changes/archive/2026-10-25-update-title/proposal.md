# Proposal

## Why

`podtrack update` solo cambia el **estado** de un episodio; el **título** únicamente se
fija al crear (`add`, `insert`). Si un episodio se retitula —como el #843, que pasó de
«La Memoria de Valet» a «¿Cómo se le da memoria a una IA?»— no hay vía: ni `update` lo
toca, ni editar el tracker a mano (prohibido). El `title` del tracker queda desfasado
respecto del guion y de los entregables.

## What Changes

- `podtrack update <num|slug>` SHALL aceptar, además de `--status`, un `--title <título>`.
- SHALL exigir **al menos uno** de los dos; sin ninguno SHALL ser un error de uso.
- `--title` SHALL cambiar el `title` del episodio. El `slug` NO SHALL cambiar: es la
  identidad estable y el material se direcciona por él.
- Un `--title` vacío o en blanco SHALL ser un error, sin escribir.
- `--status` y `--title` SHALL poder combinarse en una sola invocación.
- La salida de texto SHALL mantener **idéntica** la línea de estado actual y añadir una
  de título solo cuando el título cambie. El documento `--json` SHALL exponer el cambio
  con `field: "title"` en `changes`.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `tracker-write`: el requisito «`update` cambia el estado de un episodio» se amplía
  para cubrir también el **título**.

## Impact

- `src/main.rs`: `update` gana `--title`; `--status` pasa a opcional (al menos uno).
- `src/commands/update.rs`: `UpdateParams` gana `title`; `status` pasa a `Option`.
- `src/json.rs`: `update_document` emite la lista de cambios (estado y/o título).
- `src/error.rs`: variante para título vacío.
- `tests/parity_write.rs` y `tests/write_json.rs`: se ajustan las construcciones de
  `UpdateParams` y se añaden los casos nuevos.
- **Sin impacto** en `add`, `insert`, `publish`, ni en el formato del tracker.
- **Divergencia deliberada** respecto del oráculo Python (ya retirado): es capacidad
  nueva; no altera la conducta existente de `update --status` y sus casos del guardián
  de paridad siguen igual.
