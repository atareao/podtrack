# Proposal

## Why

El comparador del golden de paridad (`tests/common/golden.rs`) compara `stdout` y `stderr` **línea a
línea y en orden exacto**, y entre esas líneas hay tiradas del **plan de renombrado**
(`      · <origen>  →  <destino>`) cuya secuencia no la fija el binario sino el **orden del sistema
de ficheros**.

El plan sale de `collect_descendants` (`src/material.rs`), que recorre el árbol con `read_dir` y
ordena el resultado **estable** por profundidad decreciente (`Reverse(components().count())`). Los
empates de profundidad —varios hermanos en la misma carpeta— conservan el orden en que el sistema de
ficheros los devuelve, que **no está especificado** (`readdir` no ordena).

Ese orden nunca fue contrato: el golden lo congeló **tal como lo emitió el oráculo** en su momento, y
ese es su valor histórico. Pero compararlo en orden exacto convierte un detalle sin contrato en un
test intermitente que pone el CI en rojo de forma no reproducible.

La evidencia es concluyente —**el mismo árbol pasó y falló**:

| Run | Rama | Evento | Resultado |
|---|---|---|---|
| `40645390716` | `feature/episode-neighborhood` (PR #31) | `pull_request` | fallo; re-run `attempt: 2` → `success` |
| `40655350954` | `development` | `push` de `91fe756` | **`success`** |
| `40655346111` | `main` | `push` de `91fe756` | **`failure`** |

En los dos fallos caen **exactamente los mismos dos tests** (`oraculo_migrate_material` y
`oraculo_rename_material`), con el mismo mensaje `stdout difiere`; el **multiconjunto** de líneas del
plan coincide en ambos lados y lo único que difiere es el **orden**. El `main` quedó en rojo por un
flake, no por una regresión.

## What Changes

1. En `tests/common/golden.rs`, la comparación de `stdout`/`stderr` SHALL tratar como **multiconjunto
   por grupo de profundidad** las líneas del plan de renombrado
   (`      · <origen>  →  <destino>`) dentro de cada tirada máxima, conservando la **posición** de la
   tirada y el orden **entre** grupos de profundidad; todo lo demás SHALL compararse exacto y en
   orden.
2. El arreglo SHALL ser **solo del comparador**: NO SHALL tocar `tests/golden/` —solo lectura y de
   origen oráculo— ni el binario, cuya salida (y por tanto su fidelidad al oráculo) sigue igual.
3. Un test nuevo SHALL fijar el contrato resultante: permutar hermanos de una tirada NO SHALL hacer
   fallar la comparación, y una línea de más, de menos, alterada o movida a otro grupo de profundidad
   SHALL hacerla fallar; el orden de lo que **no** es el plan (p. ej. los registros de
   `rebase-references`, que van ordenados a propósito) SHALL seguir siendo contrato.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `python-retirement`: el requisito «La paridad byte a byte se conserva como golden congelado de
  origen oráculo» se **acota**: la paridad conserva los bytes, el contenido y el orden por profundidad
  del plan de renombrado, pero el orden entre hermanos de la misma profundidad no es contrato, porque
  lo decide el sistema de ficheros. Se añaden los escenarios correspondientes.

## Impact

- `tests/common/golden.rs`: el ayudante que canoniza las tiradas del plan y su uso en
  `assert_outcome`.
- `tests/golden_listing_order.rs` (**nuevo**): el contrato del comparador.
- `openspec/specs/python-retirement/spec.md`: delta (un requisito MODIFIED).
- **Sin impacto** en `tests/golden/` (solo lectura), en el binario, en la salida de ningún comando ni
  en el resto de la suite.
