# Proposal

## Why

`podtrack add "<título>"` deriva el `slug` del título cuando no se le pasa `--slug`;
`podtrack insert --before <n> --title "<título>"` **no lo hace**. Como el material de
guion se direcciona **por slug** (`where`, el layout del episodio, el propio pipeline
F1-F8), un episodio insertado nace inutilizable: hay que pasar un `backfill-slugs`
correctivo antes de poder trabajar con él.

Reproducido en real: `insert --before 850 --title "ZenNotes: …"` creó `#850` **sin
slug**, y `podtrack where 850` respondió `ERROR: El episodio #850 no tiene slug; no se
puede resolver la ruta de su material.` hasta pasar `backfill-slugs`.

Es la misma familia que `add`: lo natural es que `insert --title` (sin `--slug`) derive
el slug, con la misma regla de unicidad.

## What Changes

- `insert` SHALL derivar el `slug` del título (kebab-case, con el mismo sufijado
  anticolisión `-2`, `-3`, … que `add` y `backfill-slugs`) cuando no se aporte `--slug`.
- `--slug` explícito SHALL seguir mandando tal cual.
- Un título que no produzca `slug` utilizable (`slugify` vacío) y sin `--slug` SHALL
  dejar el episodio sin `slug`, igual que hace hoy `add`.
- Sin cambios en el resto de la semántica de `insert` (desplazamiento de números,
  fechas y material), ni en su `--dry-run`, ni en la forma del documento `--json`.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `tracker-write`: el requisito «`insert` desplaza números, fechas y material» se
  amplía para fijar la **derivación del slug** del episodio nuevo.

## Impact

- `src/commands/insert.rs`: el `slug` deja de ser `None` fijo cuando falta `--slug`.
- `src/model.rs`: se extrae el helper de unicidad de slug ya duplicado en `add` y
  `backfill-slugs`, para no escribir una tercera copia.
- `src/commands/add.rs`, `src/commands/backfill_slugs.rs`: pasan a usar el helper común
  (sin cambio de conducta).
- `openspec/specs/tracker-write/spec.md`: delta `MODIFIED` del requisito de `insert`.
- **Sin impacto** en el perfil de show, en el parseo del tracker ni en el resto de
  comandos.
- **Divergencia deliberada** respecto del oráculo Python (ya retirado): su `insert`
  dejaba el episodio sin `slug`. Los casos del guardián de paridad que ejercitan
  `insert` se fijan con `--slug` explícito para seguir siendo fieles al oráculo, y la
  derivación se cubre con pruebas de fixture propias. Se documenta en `design.md`.
