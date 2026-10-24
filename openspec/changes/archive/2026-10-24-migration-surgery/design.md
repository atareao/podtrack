# Design

## Context

Las fases 1–2 dejaron el modelo tipado, la lectura, el escritor canónico y los cinco comandos del ciclo normal. `material.rs` ya contiene el renombrado por **número** (`0NNN`/`eNNN`), con `plan_number_renames` (orden de más profundo a menos) y `apply_renames`. Esta fase añade la **cirugía**: reordenar, realinear, migrar a slug y rebasar referencias.

Restricciones que dan forma al diseño:

- La salida de texto es **contrato con los agentes**: byte a byte, incluidos los avisos a `stderr` y los resúmenes finales.
- Los tests **nunca** tocan `/data/notas`; todo sobre fixtures temporales.
- La cirugía **reescribe ficheros del usuario** (`.md`) y **renombra en disco**: debe ser previsible, reportada y con `--dry-run` real.

## Goals / Non-Goals

**Goals:**

- Paridad byte a byte de `renumber`, `rename-material` (`--from/--delta` y `--check`), `backfill-slugs`, `migrate-material` y `rebase-references` (texto + *exit codes* + tracker/material resultante).
- Detección **previa** de colisiones en `migrate-material` (no dejar una carpeta a medias).
- Reescritura de hooks que **no toque la prosa** numérica y que sea **idempotente**.
- `--dry-run` sin efectos (ni tracker, ni material, ni ficheros `.md`). `--json` aditivo.

**Non-Goals:**

- Reapuntar los agentes al binario (fase 5).
- Nueva interfaz de argumentos: se reproduce la del oráculo (`--from`, `--delta`, `--check`, `--include-published`, `--dry-run`).

## Decisions

### D1. Los dos renombrados de material conviven en `material.rs`

`rename-material` usa el renombrado **por número** ya existente. `migrate-material` usa el renombrado **por slug**, nuevo: `slug_component` (reescribe `episodio-NNN`, `eNNN`, `0NNN` y `NNN` → slug, con inicios alternos que colapsan en el mismo destino), `collect_slug_renames` (pares de más profundo a menos, con la carpeta del episodio al final) y `slug_collisions` (destinos existentes o varios orígenes al mismo destino). Se mantiene todo en `material.rs` porque comparten el orden y la aplicación.

### D2. `migrate-material` detecta colisiones **antes** de aplicar

A diferencia del renombrado por número (par-a-par, como el oráculo), `migrate-material` **planifica toda la carpeta, computa las colisiones y, si hay alguna, no renombra nada de esa carpeta** y pasa a la siguiente. Es lo que hace el oráculo (`_slug_collisions` antes de `_apply_slug_renames`) y lo que conviene: una carpeta a medias en una migración de convención es difícil de revertir. Devuelve `1` si hubo **alguna** colisión.

### D3. `rebase-references` usa `regex`, con los patrones del oráculo

Los hooks estructurales se reconocen con los mismos patrones que el oráculo (`_HOOK_TITLE_RE`, `_HOOK_BARE_RE`) y la prosa con `_NUM_REF_RE` (`\b\d{3,4}\b`), compilados con el crate `regex` (ya en el árbol de dependencias). **No** se usan lookbehinds (Rust `regex` no los soporta); los patrones del oráculo tampoco los usan. `_rebase_line` clasifica cada línea en `rewritten` / `no_title` / `None`, y solo la prosa numérica que coincide con un número de episodio **no publicado** se reporta (nunca se reescribe). La reescritura es **atómica** (temp + replace).

### D4. `--dry-run` idéntico al oráculo, incluido el «no escribir el tracker»

`renumber`, `rename-material`, `backfill-slugs`, `migrate-material` y `rebase-references` imprimen su línea de dry-run, previsualizan y **no** escriben: ni `save_tracker`, ni renombrado de material, ni reescritura de `.md`. Los resúmenes finales que el oráculo emite **solo sin dry-run** (`✓ N elemento(s) renombrado(s).` en `rename-material`) se respetan.

### D5. Los códigos de salida son contrato

`renumber`, `rename-material` y los errores de uso salen con `1`; el parser de `clap` con `2`. `migrate-material` devuelve `1` si hubo colisión (y `0` si no). `rebase-references` devuelve `1` si quedó algún hook sin título. `backfill-slugs` siempre `0`.

### D6. `--json` de cirugía: acción + cambios

Cada comando emite `{ schema_version, show, action, changes: [...], dry_run }` con las entidades afectadas (episodios reenumerados, pares renombrados, slugs asignados, hooks reescritos). Aditivo: sin `--json` la salida de texto no cambia.

## Risks / Trade-offs

- **Cirugía destructiva.** `migrate-material` y `rebase-references` reescriben el material del usuario. *Mitigación:* fixtures temporales en test, `--dry-run`, detección previa de colisiones, reescritura atómica, y no reapuntar producción hasta la fase 5.
- **Regex vs. parsing a mano.** Los patrones de hook son los del oráculo; `regex` garantiza la misma semántica sin lookbehind. Alternativa (a mano) más código y más riesgo.
- **Idempotencia.** Una segunda pasada de `backfill-slugs` o `rebase-references` no debe cambiar nada. Se prueba explícitamente.
- **Deriva de paridad.** El oráculo puede cambiar mientras se porta. *Mitigación:* suite + contraste en vivo y `--dry-run` para comparar sin efectos.
