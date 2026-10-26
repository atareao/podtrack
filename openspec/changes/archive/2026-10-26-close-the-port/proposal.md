# Proposal

## Why

El port terminó, pero el repositorio no lo sabe.

La fase 5 (`retire-python`) borró `podcast-tracker.py`, `srt-stage.py` y
`podcast-tracker.test.py` de la skill y conmutó agentes y `SKILL.md` al binario. Los tests de
`podtrack` quedaron **fuera de su alcance por diseño**: su spec acotaba el invariante a
`~/.agents/skills/podcast-srt/` y su criterio de cierre era un `grep` sobre la skill.

La consecuencia, medida hoy: **27 de los 339 tests no comprueban nada**. Cuando el oráculo no está,
el arnés de paridad imprime el motivo y hace `return`; como ni `just check` ni el CI definen
`PODTRACK_REQUIRE_ORACLE`, esos 27 reportan `passed` sin ejecutar una sola aserción:

- `cargo test` → 339 passed / 0 failed.
- `PODTRACK_REQUIRE_ORACLE=1 cargo test --no-fail-fast` → 312 passed / 27 failed.

Son el 8 % de la suite y, precisamente, el contraste byte a byte que era la red de seguridad más
fuerte del proyecto. Los 27 viven en seis ficheros: `parity_oracle.rs` (2),
`parity_write_oracle.rs` (9), `parity_writer.rs` (2), `parity_surgery.rs` (6), `srt_stage.rs` (7) y
`rebase_references.rs` (1).

A la vez, la documentación narra el mundo **antes** del port: `AGENTS.md` sigue diciendo que el
script Python «hoy vive» en la skill, que el andaime «se derriba al terminar» y que «Hasta
entonces, el Python sigue siendo el que corre»; `README.md` promete retirar el script «cuando el
binario lo sustituya por completo»; `GIT_FLOW.md` da `v0.5.0` como última release; y cuatro
doc-comments hablan del port en presente. Nada de eso es cierto desde `v0.5.1`.

No es un fallo de `retire-python`: es deuda nueva que su propio alcance dejó abierta. Este change
la cierra.

## What Changes

1. **Congelar la paridad como golden de origen oráculo.** Los 27 tests SHALL dejar de invocar
   Python y SHALL comparar contra un golden versionado en `tests/golden/`, grabado **una vez** con
   el oráculo recuperado del histórico de `~/.agents` (`c70287f^`). El golden NO SHALL grabarse
   desde el binario: entre «bytes idénticos al oráculo de referencia» y «bytes idénticos a lo que el
   binario hizo hoy» está la diferencia entre paridad y tautología.
2. **Forma del golden.** Un fichero JSON por caso con `code`, `stdout` y `stderr` y, donde hoy se
   compara, el `podcast-tracker.yaml` resultante y el árbol de material; para el writer canónico,
   los bytes del YAML. Legible en un `git diff`.
3. **Reescribir los 27 en su sitio, con los mismos nombres.** La matriz de casos es el activo; solo
   cambia el lado «esperado». Un golden ausente SHALL fallar el test, nunca saltarlo.
4. **Retirar el arnés en vivo.** `tests/common/mod.rs` SHALL perder `oracle_path`, `python_ready`,
   `require_oracle`, `require_oracle_srt`, `ORACLE_PREAMBLE`, `SRT_MAIN_HARNESS` y sus dependencias;
   `tests/` SHALL quedar sin `python3`, sin `.py` del oráculo y sin `PODTRACK_REQUIRE_ORACLE`,
   `PODTRACK_ORACLE` ni `PODTRACK_ORACLE_SRT`.
5. **Guardián de no-Python.** Un test SHALL fallar si cualquiera de esas referencias reaparece en
   `tests/`.
6. **Grabación de un solo uso.** El código que graba el golden SHALL vivir solo durante el change y
   SHALL retirarse antes de commitear: el repositorio no SHALL quedarse con generador. El golden no
   es regenerable por diseño; su procedencia SHALL constar en la cabecera de los seis ficheros y en
   `design.md`.
7. **Documentación veraz.** `AGENTS.md` (líneas 7, 21-22, 27, 29 y sección 43-53), `README.md:19`,
   `GIT_FLOW.md:5` y los doc-comments de `src/lib.rs:3`, `src/error.rs:3,164` y
   `src/srt_stage.rs:3` SHALL declarar el estado real, sin promesas ni futuros.
8. **Cero cambio de comportamiento.** El binario NO SHALL cambiar: ni código de producción ni
   salidas. Solo tests, golden y documentación.

## Capabilities

### New Capabilities
Ninguna.

### Modified Capabilities
- `python-retirement`: el invariante de «Python cero» se extiende de la skill al repositorio. La
  suite SHALL comprobar de verdad contra un golden congelado de origen oráculo, sin invocar Python
  ni depender de variables `PODTRACK_*`; la documentación SHALL dejar de describir el port como en
  marcha; y un guardián mecánico SHALL impedir que el Python reaparezca en `tests/`.

## Impact

- `tests/common/mod.rs` — se retira el arnés en vivo y se añade el lector del golden.
- `tests/parity_oracle.rs`, `tests/parity_write_oracle.rs`, `tests/parity_writer.rs`,
  `tests/parity_surgery.rs`, `tests/srt_stage.rs`, `tests/rebase_references.rs` — los 27 casos
  leen el golden en vez del oráculo; cabecera con la procedencia.
- `tests/golden/` — nuevo; un JSON por caso.
- Guardián de no-Python — un test de integración nuevo.
- `AGENTS.md`, `README.md`, `GIT_FLOW.md` — afirmaciones desfasadas.
- `src/lib.rs`, `src/error.rs`, `src/srt_stage.rs` — solo doc-comments.
- **No** se toca el comportamiento del binario. **No** se escribe `/data/notas`: el golden se graba
  sobre fixtures sintéticos. **No** se commitean los `.py` recuperados ni el generador.

## Criterio de cierre

- cargo test sin variables PODTRACK_* → **340 passed / 0 failed / 0 saltados** (los 339 previos, ninguno saltado, más el test guardián).
- `grep -rn 'python3\|PODTRACK_REQUIRE_ORACLE\|PODTRACK_ORACLE' tests/` → vacío.
- `just check-all` verde y `openspec validate close-the-port --strict` OK.
- Paridad con datos reales como no-regresión: 14/14 contra `v0.5.1` (procedimiento de `PLAN-007`).
- `/data/notas` intacto: solo lectura.
