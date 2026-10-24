# Proposal

## Why

Las fases 1 y 2 cubrieron el ciclo normal —**leer** y **escribir** la parrilla—, pero el oráculo trae además una **caja de herramientas de cirugía** que el usuario invoca cuando la parrilla y el material en disco se desalinean a mano: reordenar la numeración (`renumber`), realinear las carpetas `0NNN` con el tracker (`rename-material`), rellenar los `slug` que faltan (`backfill-slugs`), migrar el material de la convención numérica a la de slug (`migrate-material`) y reescribir los hooks de «Próximo episodio» que las referencias cruzadas dejaron clavados a un número (`rebase-references`).

Es el último bloque de comandos del oráculo y el más peligroso: **toca el material en disco y reescribe ficheros `.md`**. Portarlo con paridad byte a byte cierra el port funcional del tracker y desbloquea la fase 4 (`srt-stage`) y la 5 (`retire-python`), donde se Apunta a los agentes y se borran los `.py`.

## What Changes

1. **`renumber --from <num> --delta <±d>`:** renumera **solo** los números del tracker (ni fechas ni material). `--delta 0` es error; sin episodios `>= from` es error; un `--delta` que solape con números no afectados es error de colisión. `--dry-run` previsualiza.

2. **`rename-material`:** con `--from`/`--delta` renombra las carpetas `0NNN` y los ficheros `0NNN`/`eNNN` del material (la maquinaria de `material-renumber`); con `--check` solo **detecta y lista** las carpetas numéricas huérfanas (número que no está en el tracker), sin tocar nada.

3. **`backfill-slugs`:** deriva y persiste el `slug` (con `slugify` + desambiguación `-2`, `-3`, …) de los episodios que no lo tienen; idempotente; avisa de los títulos que no producen slug utilizable.

4. **`migrate-material`:** renombra el material de `0NNN/` a `<slug>/` (y los ficheros internos `0NNN`/`NNN`/`eNNN`/`episodio-NNN` → slug). Por defecto solo no publicados; `--include-published` los incluye. **Detecta colisiones de destino antes de tocar** y, si las hay, deja esa carpeta intacta y devuelve 1.

5. **`rebase-references`:** reescribe los hooks estructurales `Próximo episodio: NNN — <título>` de los `.md` de episodios **no publicados**, dejando `<título>` sin número. La **prosa** numérica no se toca (se reporta para revisión). Un hook sin título se reporta y devuelve 1. Idempotente.

6. **`--json` extendido** a estos comandos (aditivo; la salida de texto no cambia).

## Capabilities

### New Capabilities
- `tracker-renumber`: renumeración de la parrilla (solo números) con detección de colisión.
- `material-slug-migration`: migración del material de la convención numérica a la de slug, con detección previa de colisiones.
- `slug-backfill`: derivación y persistencia de los `slug` ausentes.
- `reference-rebase`: reescritura de los hooks «Próximo episodio» del material no publicado, con reporte de la prosa numérica.

### Modified Capabilities
- `material-renumber`: se le añade la **detección de huérfanos** (`--check`) y la exposición como subcomando `rename-material` sobre su maquinaria.
- `json-output`: se añaden los documentos de los comandos de cirugía (aditivo).

## Impact

- **`src/commands/`** — comandos nuevos: `renumber`, `rename_material`, `backfill_slugs`, `migrate_material`, `rebase_references`.
- **`src/material.rs`** — extensión con el renombrado por slug (`slug_component`, `collect_slug_renames`, `slug_collisions`) y la detección de huérfanos.
- **`src/rebase.rs`** (nuevo) — el reconocimiento y reescritura de hooks (`_rebase_line`) y el reporte de prosa. Usa el crate `regex` (ya presente en el árbol de dependencias).
- **`src/json.rs`** — documentos de cirugía.
- **`tests/`** — suite de paridad de cirugía sobre fixtures temporales.
- **No** se toca `/data/notas`, ni los agentes, ni los `.py`: el Python sigue en producción hasta la fase 5.
