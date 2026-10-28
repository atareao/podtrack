# Design

## Context

El comparador del golden (`tests/common/golden.rs`) es el candado de la paridad con el oráculo
retirado: compara `code`, `stdout`, `stderr` —y, donde aplica, `tracker`, `material` y `yaml`— contra
`tests/golden/<área>/<caso>.json`. Hasta hoy `stdout`/`stderr` se comparaban **línea a línea y en
orden exacto**.

Entre esas líneas hay **tiradas del plan de renombrado**, con la forma
`      · <origen>  →  <destino>` (6 espacios, `·`, dos espacios, flecha, dos espacios; la emite
`src/material.rs` y la replican las rutas *dry-run* de `insert`, `rename-material` y
`migrate-material`). El plan lo producen `plan_number_renames` y `collect_slug_renames_into`, que
recorren el árbol con `collect_descendants` (`src/material.rs:106`) —un `read_dir` recursivo— y
ordenan **estable** por profundidad decreciente (`Reverse(components().count())`). Los empates de
profundidad conservan el orden del `read_dir`, que **no está especificado por ningún SO**.

El orden intra-profundidad nunca fue contrato: el golden lo congeló tal como lo emitió el oráculo.
Compararlo en orden exacto convierte un detalle sin contrato en un test intermitente.

Evidencia (el mismo árbol, dos resultados):

| Run | Rama | Evento | Resultado |
|---|---|---|---|
| `40645390716` | `feature/episode-neighborhood` (PR #31) | `pull_request` | fallo; re-run `attempt: 2` → `success` |
| `40655350954` | `development` | `push` de `91fe756` | `success` |
| `40655346111` | `main` | `push` de `91fe756` | `failure` |

En los dos fallos caen **los mismos dos tests** (`oraculo_migrate_material`, `oraculo_rename_material`)
con el mismo `stdout difiere`, y el multiconjunto de líneas del plan coincide en ambos lados: solo
cambia el orden.

## Goals / Non-Goals

**Goals**

- Quitar la intermitencia sin tocar el binario ni el golden.
- Que lo que **sí** es contrato —el contenido del plan y su orden por profundidad— siga comparándose.
- Que el comparador falle igual de fuerte ante un plan con una línea de más, de menos, alterada o en
  el grupo de profundidad equivocado.

**Non-Goals**

- No se regenera `tests/golden/` (D1 de `close-the-port`: evidencia histórica, solo lectura).
- No se cambia el orden de listado del binario: la fidelidad al oráculo manda sobre el determinismo
  de la presentación.
- No se relaja nada más que la comparación de las tiradas del plan: `tracker`, `material`, `yaml` y
  el resto de `stdout`/`stderr` siguen exactos.

## Decisions

### D1 — La no-determinación se acota al orden entre hermanos de una tirada del plan

`collect_descendants` es el único punto que hereda el orden del sistema de ficheros; el resto de
listas del binario van ordenadas explícitamente por ruta (`src/rebase.rs:100` para los ficheros `.md`
de `rebase-references`; `src/commands/rebase_references.rs:153,205` para la prosa). Por eso el
relajamiento se ciñe a las líneas `      · <origen>  →  <destino>` y a nada más.

**Alternativas descartadas:**

- *Ordenar el listado en el binario:* cambiaría la salida real para contentar a un test. El orden del
  oráculo es el contrato; el test debe adaptarse, no la implementación.
- *Regenerar el golden:* prohibido por D1 de `close-the-port` (evidencia histórica, nunca escrita por
  el crate).
- *Relajar **todas** las líneas `      · `:* la guarda caería también sobre los registros de
  `rebase-references` (`      · <ruta>:<línea>`), que van ordenados a propósito y por tanto **sí** son
  contrato, y sobre `      · <problema>` de `migrate-material` en `stderr` (sin flecha).
- *Ordenar la tirada entera (multiconjunto global de la tirada):* quitaría la intermitencia, pero
  además taparía una regresión del orden por profundidad. Por eso D2 lo acota a cada grupo.

### D2 — Multiconjunto **por grupo de profundidad**, no de la tirada entera

Dentro de una tirada máxima, las líneas se agrupan en **grupos consecutivos** de igual profundidad
del origen (un componente de ruta por cada `/`) y **cada grupo** se ordena antes de comparar. Los
grupos conservan su posición: una línea no puede saltar de grupo y la tirada sigue leyéndose de más
profundo a menos dentro de cada episodio (`0102` completo, luego `0101`). «Consecutivos» es
deliberado: no se trata de cubos globales por profundidad, así que dos episodios a la misma
profundidad siguen siendo grupos distintos y su orden relativo sigue siendo contrato.

Esto conserva exactamente el contrato que el binario sí garantiza y solo canoniza lo que decide el
`read_dir`. Medido antes de escribir el código: de los 12 tiradas de plan repartidas en 6 de los 27
ficheros del golden, la secuencia de profundidades es determinista (función de las rutas, no del
`read_dir`) y se agrupa por episodio —lo que hace del agrupamiento por profundidad una
canonicalización estable.

### D3 — La verificación que se pierde es la que ya no era contrato, y hay triple red

- Lo que se deja de comparar es **solo** el orden entre hermanos de la misma profundidad, decidido por
  el `read_dir`.
- El **número y el contenido** de las líneas del plan siguen fijados: una línea de más, de menos o
  alterada falla (escenarios del delta y tests de `golden_listing_order.rs`).
- El orden **por profundidad dentro de cada tirada** sigue fijado por D2; el unitario
  `plan_ordenado_de_mas_profundo_a_menos` (`src/material.rs:348`) es una red **más laxa** (solo
  comprueba que la carpeta superior va al final), no una segunda verificación del orden total.
- Y el resultado **real** de la operación no depende del orden impreso: esos mismos casos comparan el
  **árbol de material** resultante (10–21 entradas por caso) con orden exacto, de modo que unos
  renombrados mal aplicados no cuadrarían.

### D4 — El comparador es el único punto de cambio

La canonización vive en un ayudante privado de `tests/common/golden.rs` y se aplica en
`assert_outcome` (líneas 56-70), que es el único sitio por el que pasan `stdout` y `stderr`. Ni el
binario, ni el golden, ni los tests de paridad se tocan.

## Risks / Trade-offs

- **Riesgo:** que el agrupamiento por profundidad sea demasiado fino y haga fallar un caso hoy verde.
  **Mitigación:** la profundidad de cada línea es función de su ruta, no del `read_dir`; dos
  ejecuciones con las mismas rutas producen la misma secuencia de grupos. Verificado sobre los 27
  ficheros del golden antes de escribir el código.
- **Trade-off:** un golden con un plan «desordenado a propósito» ya no se detectaría. No existe: el
  plan se emite siempre por profundidad decreciente dentro de cada episodio.
- **Riesgo latente (fuera de este cambio):** `slug_collisions` (`src/material.rs`) acumula sus avisos
  en el orden de inserción, que también viene del `read_dir`, y esas líneas
  (`      · <problema>`, sin flecha) NO se canonizan. Hoy cada caso del golden lleva un único aviso,
  así que no afecta; queda anotado para futuros goldens con más de uno en el mismo caso.
