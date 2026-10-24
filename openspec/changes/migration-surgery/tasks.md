# Tasks

## 0. Prerrequisito: andamiaje

- [ ] 0.1 Extender `src/material.rs` con el renombrado **por slug**: `slug_component` (tokens `episodio-NNN`, `eNNN`, `0NNN`, `NNN` → slug), `collect_slug_renames` (pares de más profundo a menos, carpeta del episodio al final) y `slug_collisions` (destinos existentes o convergentes).
- [ ] 0.2 Añadir la detección de huérfanos (`\d{4}` sin episodio en el tracker) para `rename-material --check`.
- [ ] 0.3 Crear `src/rebase.rs` con los patrones de hook (`_HOOK_TITLE_RE`, `_HOOK_BARE_RE`, `_NUM_REF_RE`) y `rebase_line` (clasificación `rewritten` / `no_title` / `None`), más la escritura atómica.
- [ ] 0.4 Añadir la dependencia `regex` (ya en la caché de `~/.cargo/registry`) y `cargo check` limpio.

## 1. RED (pruebas que fallan primero)

- [ ] 1.1 `tracker-renumber`: escenarios de desplazamiento positivo/negativo, `--delta 0`, umbral sin episodios, colisión de numeración, `--dry-run` y orden ascendente de las líneas.
- [ ] 1.2 `material-renumber`: `rename-material --from/--delta` (orden según signo), exigencia de `--from`/`--delta`, resumen final y `--check` (huérfano vs. alineado).
- [ ] 1.3 `material-slug-migration`: paso de carpeta y tokens internos a slug, `episodio-NNN`, selección de publicados, aviso de episodio sin slug, detección de colisiones (destino existente y convergencia), exit `1`, `--dry-run`.
- [ ] 1.4 `slug-backfill`: asignación derivada, no tocar slugs existentes, desambiguación `-2`, aviso de título sin slug, tabla + resumen e idempotencia (segunda pasada `0`).
- [ ] 1.5 `reference-rebase`: hook con título (con y sin tilde), prosa reportada sin tocar, hook sin título → exit `1`, idempotencia, escritura atómica y `--dry-run`.
- [ ] 1.6 `json-output`: documentos de los cinco comandos de cirugía, `--dry-run --json` y "sin `--json` no cambia el texto".
- [ ] 1.7 `cargo test`: las pruebas nuevas fallan por no existir el código.

## 2. GREEN (implementación mínima)

- [ ] 2.1 `commands/renumber.rs`: renumeración con validaciones y guardado.
- [ ] 2.2 `commands/rename_material.rs`: `--from/--delta` y `--check`.
- [ ] 2.3 `commands/backfill_slugs.rs`: derivación, desambiguación y persistencia.
- [ ] 2.4 `commands/migrate_material.rs`: planificación por slug, colisiones previas, selección de publicados.
- [ ] 2.5 `commands/rebase_references.rs`: recorrido de `.md`, reescritura de hooks, reportes de prosa y sin título.
- [ ] 2.6 `main.rs`: subcomandos y banderas (`--from`, `--delta`, `--check`, `--include-published`, `--dry-run`).
- [ ] 2.7 `json.rs`: documentos de cirugía.
- [ ] 2.8 `cargo test` en verde.

## 3. REFACTOR

- [ ] 3.1 `cargo fmt --all` y `cargo clippy --all-targets -- -D warnings` limpios.
- [ ] 3.2 Sin `unwrap()`, `expect()` ni `panic!()` en producción.
- [ ] 3.3 Ninguna escritura del tracker fuera de `save_tracker`; ningún test toca `/data/notas` (fixtures temporales).

## 4. Verificación de paridad (operador)

- [ ] 4.1 Para cada comando de cirugía, comparar con `diff` la salida, el **exit code**, el **tracker resultante** y el **árbol de material** del binario y del Python sobre el **mismo fixture** congelado; SHALL ser idénticos.
- [ ] 4.2 Comprobar `--dry-run` (ni tracker, ni material, ni `.md`) contra el oráculo.
- [ ] 4.3 Idempotencia de `backfill-slugs` y `rebase-references` (segunda pasada sin cambios) contrastada con el oráculo.
- [ ] 4.4 `podtrack <cmd> --json --show <fixture>` devuelve JSON válido (`jq .`).
- [ ] 4.5 `just check-all` en verde y guardián `PODTRACK_REQUIRE_ORACLE=1` en verde.
