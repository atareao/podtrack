# PLAN-001 — `get` muestra slug y ruta del material

- **Estado:** en curso
- **Change OpenSpec:** `get-shows-slug-and-path`
- **Ámbito:** tracker (solo lectura) + salida `--json`

## Objetivo

Que `podtrack get <número|slug>` muestre, además del detalle actual, el **slug** del episodio y la **ruta absoluta del material de guion**; y que `get --json` añada `material` al episodio.

## Decisiones cerradas con el usuario

| # | Decisión |
|---|---|
| 1 | **Ruta** = absoluta, mismo criterio que `status`: `<scripts.root>/<slug>`. **No** se comprueba su existencia. |
| 2 | **JSON**: sí, aditivo → `episode.material` en el documento de `get`. |
| 3 | **Episodio sin slug**: `Slug: —` y `Ruta: —`. Nunca se inventa ruta. |
| 4 | `schema_version` permanece `1` (cambio aditivo, no rompe consumidores). |

## Contrato de salida (texto)

Se **añaden** dos líneas al final del bloque; las cinco existentes quedan byte a byte:

```
  ═══════════════════════════════════════════════════════
  Episodio #843
  ═══════════════════════════════════════════════════════
  Título:    La memoria de Valet
  Estado:    ✅ published
  Fecha:     2026-09-15
  Temporada: 4
  Slug:      la-memoria-de-valet
  Ruta:      /data/notas/Podcasts/guiones/la-memoria-de-valet
  ═══════════════════════════════════════════════════════
```

- Alineación de etiqueta a 11 columnas, igual que `Temporada: `.
- Precedente de etiquetas `slug:` / `ruta:`: `status.next_to_publish_block` (`src/commands/status.rs`).
- Etiquetas en el bloque de `get`: capitalizadas (`Slug:`, `Ruta:`), coherentes con `Título:`, `Estado:`, `Fecha:`, `Temporada:`.

## Superficie a tocar

| Fichero | Cambio |
|---|---|
| `src/commands/get.rs` | dos líneas nuevas en la rama de texto; helper puro para derivar la ruta absoluta |
| `src/json.rs` → `get_document` | añadir `material` dentro del objeto `episode`; **solo** `get` (el helper compartido `episode_json` NO se toca) |
| `openspec/specs/tracker-read/spec.md` | delta `MODIFIED` del requisito de `get` |
| `openspec/specs/json-output/spec.md` | delta `MODIFIED` de la carga útil: `get --json` |
| `docs/DEFINITION.md` | fila de `get` en la tabla de comandos |
| `tests/parity_read.rs` | casos nuevos + `get 202` (episodio sin slug) |
| `tests/json_output.rs` | caso `get --json` con `material` |

**Sin impacto:** parseo del tracker, perfiles de show, escritura, cirugía, `srt-stage`.

## Riesgos y verificaciones

- **Paridad:** el oráculo Python está retirado (`retire-python`); esto es divergencia deliberada y se documenta en `design.md`, igual que hizo `status-upcoming-window`.
- **`get 202`** del fixture `MINI_YAML_V2` no tiene slug: es el caso natural de `—`.
- Los tests existentes (`get_cli_por_slug`, `cmd_get_por_slug`) solo comprueban substring: deben seguir verdes.

## Tareas (TDD)

- **0. Prerrequisito:** `git status` limpio; `cargo check` limpio de partida.
- **1. RED:** `tests/parity_read.rs`: (a) `get 200` → contiene `Slug: primer-episodio` y la ruta absoluta `<root>/primer-episodio`; (b) `get 202` (sin slug) → `Slug: —` y `Ruta: —`; (c) la ruta se muestra aunque el directorio no exista (no se crea nada). `tests/json_output.rs`: (d) `episode.material` con la ruta; (e) `material: null` cuando el episodio no tiene slug. `cargo test` en rojo por las nuevas.
- **2. GREEN:** implementación mínima en `src/commands/get.rs` y `src/json.rs`; `cargo test` en verde.
- **3. REFACTOR:** `cargo fmt --all`; `cargo clippy --all-targets -- -D warnings`; cero `unwrap()`/`expect()`/`panic!()` en producción; ningún test toca `/data/notas` ni `/data/podcasts`; pasada manual de solo lectura sobre el tracker real.
- **4. Archivo:** `just spec-validate get-shows-slug-and-path`; `just check-all`; `just spec-archive get-shows-slug-and-path`; marcar `tasks.md` y renombrar este plan a `-COMPLETED` con `just plan-done 1`.
