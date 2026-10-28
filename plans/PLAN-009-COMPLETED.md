# PLAN-009 — Cierre de `close-the-port` (decisión A) y feature `neighborhood`

> Testigo de la sesión del **2026-10-26**. Resuelve el §9.4 que quedó pendiente en
> [`PLAN-008-COMPLETED.md`](./PLAN-008-COMPLETED.md) —el testigo de la recuperación del oráculo y de la grabación del
> golden— y añade el trabajo del change nuevo `episode-neighborhood`.
>
> Convención del repo: un plan por trabajo en `plans/PLAN-NNN.md`; al cerrarlo se **renombra**
> (nunca se borra) a `-COMPLETED.md` o `-PENDING.md`.
>
> **Cerrado el 2026-10-28**: la fase A el 2026-10-26 (§9) y la fase B el 2026-10-28 (§11). Renombrado a
> `PLAN-009-COMPLETED.md`.

## 0. Estado de partida

| Dato | Valor |
|---|---|
| Rama | `development` @ `0c1f61a` (= `origin/development`), **0 commits nuevos** |
| Sin commitear | 7 ` M` en `tests/` (`common/mod.rs`, `parity_oracle.rs`, `parity_write_oracle.rs`, `parity_writer.rs`, `parity_surgery.rs`, `rebase_references.rs`, `srt_stage.rs`) + 4 sin trackear (`tests/common/golden.rs`, `tests/golden/`, `openspec/changes/close-the-port/`, `plans/PLAN-008.md`) |
| `src/` | **intacto** |
| Suite | `cargo test` → **338 passed / 1 failed**; el único fallo es `paridad_oraculo_fixture_mini`, escenario `get 100` |
| Bloqueo | **resuelto hoy** (§2, decisión 9): la divergencia de `get` se excluye del golden |
| Oráculo | `/tmp/opencode/oracle-recovery/` — huellas `sha256` en `PLAN-008` §9.11 |
| Golden | 27 ficheros / 130 escenarios / 158 428 B — huella de árbol `6df44684ca154fed58901cce03ddec6a13958c2b4a3903f8aa61e707d69c1351` (**deja de valer** en A.3) |
| Versión | `0.5.1` (`.vampus.yml`) |
| **Cierre** | fase A ejecutada el 2026-10-26: evidencia medida en **§9** |

## 1. Alcance: dos fases, en serie

| Fase | Qué | Estado |
|---|---|---|
| **A** | Cerrar `close-the-port`: auditoría de divergencias, GREEN **340/0**, retirada del arnés del oráculo, guardián de no-Python, documentación veraz, PR y release | Aprobado y **desbloqueado hoy** |
| **B** | `episode-neighborhood`: change nuevo, SDD → TDD → auditoría → PR → release `0.6.0` | **Cerrada** el 2026-10-28 — `v0.6.0` publicada y, después, el hotfix `v0.6.1` del flake del comparador del golden (**§11**) |

**Regla de orden (decisión 7):** la fase B no se empieza con la fase A a medias. Los dos trabajos
tocan `src/` y `tests/`; mezclados, la verificación de cierre de ninguno de los dos es creíble.

## 2. Decisiones cerradas hoy

| # | Decisión | Origen |
|---|---|---|
| 1 | La vecindad es **numérica** (`678` ± 1..N), no el orden de parrilla por fecha | usuario |
| 2 | El episodio de referencia **sale, centrado y marcado** con `►` | usuario |
| 3 | Menos de 4 vecinos → **muestra los que haya** (nunca falla en un extremo) | usuario |
| 4 | **`--before N` / `--after M`**, con `4`/`4` por defecto y `0` válido | usuario |
| 5 | Salida en **tabla**, reutilizando `show_episodes`; sin formato nuevo | usuario |
| 6 | Con huecos, los **N/M episodios existentes más cercanos** (sin filas vacías ni números inventados) | usuario |
| 7 | **Cerrar `close-the-port` antes** de empezar la feature | usuario |
| 8 | **§9.4 = opción A**: excluir del golden los escenarios `get 100` y `get 101` del fixture `mini` | usuario |
| 9 | Identificación del episodio por **número o slug** (como `get`/`where`/`open`) | propuesta del director |
| 10 | `--json` **aditivo** (`episode` + `before` + `after`), `schema_version` sigue `1` | propuesta del director |
| 11 | **Sin ruta de material** en la salida (para eso están `get` y `where`) | propuesta del director |
| 12 | Un episodio **sin número** como referencia → **error explícito**, nunca un fallback silencioso a vecindad por fecha | propuesta del director |

Rationale de las 9-12 (por si hay que defenderlas): (9) los cinco comandos que aceptan un episodio lo
resuelven así, con el mismo helper `resolve_episode`; (10) todos los comandos de lectura tienen
`--json` y el comando es nuevo, así que no cambia ninguna forma existente; (11) una ventana de 9 filas
con 9 rutas deja de leerse de un vistazo, y la ruta de un vecino se pide con `get`/`where`; (12)
devolver una vecindad por fecha para quien preguntó por número sería responder a otra pregunta con el
mismo formato: el peor de los silencios.

**Por qué la 8 es la opción A y no C** (medido al preparar el spec): los dos escenarios que divergen
están **cubiertos a mano** en `tests/parity_read.rs` (líneas 154-275: bloque de detalle de `get`,
orden y alineación de `Slug:`/`Ruta:`, resolución por slug y el caso sin slug con `—`, que es
exactamente lo que diverge en el fixture `mini`). Excluirlos del golden, por tanto, **no pierde
cobertura**, mantiene el golden 100 % de origen oráculo (D1 intacto) y sigue el precedente exacto de
`status`, excluido por diseño en `status-upcoming-window` (D6 de `PLAN-008`). La opción C
(congelarlos desde el binario con marca `source: binary`) habría roto D1 a cambio de una cobertura
que ya existe.

## 3. Fase A — cerrar `close-the-port`

Rama: `feature/close-the-port` (desde `development`, llevándose el árbol sin commitear tal cual:
`git checkout -b feature/close-the-port`).

### A.0 Comprobaciones de partida (sin escribir nada)

- A.0.1 El oráculo sigue en `/tmp/opencode/oracle-recovery/`: `sha256sum *.py` debe dar `bb37e842…`
  (`podcast-tracker.py`), `92fa51d7…` (`srt-stage.py`) y `c2e3b104…` (`podcast-tracker.test.py`),
  los de `PLAN-008` §9.11. Si `/tmp` se vació, reconstruirlo con el procedimiento de §9.7.
- A.0.2 `python3 --version` (3.14.8) y `python3 -c "import yaml"` (PyYAML 6.0.3).
- A.0.3 Baseline intacta: `cargo test --locked --no-fail-fast` → 338/1, con el fallo `get 100`.

### A.1 Modo auditoría temporal: **todas** las divergencias en una pasada

Los tests abortan en el primer `assert`, así que «338/1» **no es la lista completa** de divergencias:
los otros 26 casos de paridad pueden esconder más. Hasta saberlo, la decisión A no es verificable.

- A.1.1 `tests/common/golden.rs`: modo `PODTRACK_AUDIT_GOLDEN=1` (temporal, se retira en A.5):
  `assert_outcome`/`assert_tracker`/`assert_material`/`assert_tree`/`assert_yaml` **acumulan** la
  divergencia (identidad `área/caso`, índice, comando y las primeras líneas distintas) en lugar de
  hacer `panic!`, y cada test imprime su bloque al final. Resumen final: total de divergencias.
- A.1.2 Ejecutarlo con el oráculo **no** hace falta: la auditoría compara el golden ya grabado contra
  el binario. `PODTRACK_AUDIT_GOLDEN=1 cargo test --locked --no-fail-fast`.
- A.1.3 Clasificar **cada** divergencia:
  - **Intencionada** (una capacidad posterior a `c70287f^` que cambió la salida a propósito):
    excluir el escenario del golden y documentarlo citando el change que la introdujo.
  - **No intencionada** (regresión real del binario): arreglar el binario. Un cambio de
    comportamiento inesperado es un hallazgo de primera, no una molestia.
- A.1.4 Criterio de salida: la lista queda **vacía** salvo los dos escenarios de `get` de la decisión
  8. Si aparece algo más, se decide y se documenta antes de seguir: **no se re-graba el golden para
  tapar una divergencia no decidida**.

### A.2 Aplicar la decisión A (excluir la divergencia de `get`)

- A.2.1 `tests/parity_oracle.rs`: quitar `&["get", "100"]` y `&["get", "101"]` de `SPECS`.
- A.2.2 Ampliar el comentario de cabecera de `SPECS` (que ya explica la exclusión de `status`): los
  dos escenarios se omiten desde `close-the-port` porque el binario imprime `Slug:` y `Ruta:` desde
  el change archivado `2026-10-25-get-shows-slug-and-path` (commit `c7a15e9`) y el oráculo
  recuperado (`c70287f^`) no lo hacía; están cubiertos por `tests/parity_read.rs` (líneas 154-275).
- A.2.3 `get 999` (no encontrado) y los otros siete escenarios del fixture `mini` **se quedan**.
- A.2.4 `cargo test --locked --no-fail-fast` → el fixture `mini` pasa a tener 8 escenarios (10 − 2).

### A.3 Re-grabar el golden (el generador sigue vivo hasta A.5)

- A.3.1 `rm -rf tests/golden && PODTRACK_RECORD_GOLDEN=1 PODTRACK_REQUIRE_ORACLE=1 PODTRACK_ORACLE=/tmp/opencode/oracle-recovery/podcast-tracker.py cargo test --locked --no-fail-fast`.
- A.3.2 Determinismo: segunda grabación a `/tmp/opencode/golden-run2` y `diff -r` contra `tests/golden`
  → sin diferencias (D4 de `PLAN-008`).
- A.3.3 Higiene: `grep -rc '/tmp/\.tmp' tests/golden/` → **0** líneas con ruta temporal.
- A.3.4 Inventario y huellas nuevas, que **sustituyen** a `PLAN-008` §9.9 y §9.11:
  - escenarios totales: **128** (130 − 2);
  - `parity_oracle/paridad_oraculo_fixture_mini`: **8** escenarios (antes 10);
  - huella del árbol: `find tests/golden -type f | sort | xargs sha256sum | sha256sum`;
  - bytes: `du -sb tests/golden`.
- A.3.5 `PLAN-008.md`: anotar en §9.11 la huella vieja como **invalidada por A** y la nueva, y
  corregir la cifra de escenarios en §9.9.

### A.4 GREEN

- A.4.1 `cargo test --locked --no-fail-fast` **sin ninguna variable `PODTRACK_*`** → **340 passed /
  0 failed / 0 saltados** (339 previos + el guardián).
- A.4.2 Confirmar que no queda **ninguna** ruta de `return` temprano ni `skip` en los tests de
  paridad: un golden ausente debe **fallar**, nunca saltarse.

### A.5 REFACTOR: retirada del arnés y guardián de no-Python

- A.5.1 `tests/common/golden.rs`: retirar `recording()`, `record()` y los constructores temporales
  (`GoldenCase::outcome`, `with_tree`, `with_material`, `with_tracker`, `with_yaml`), y el modo
  auditoría de A.1. Revisar si `join_lines` queda huérfano: si no lo usa nadie, se retira.
- A.5.2 `tests/common/mod.rs`: retirar `require_oracle_env`, `oracle_path`, `python_ready`,
  `require_oracle`, `ORACLE_PREAMBLE`, `SRT_MAIN_HARNESS`, `oracle_srt_path`, `require_oracle_srt`,
  `run_python_harness`, `MAIN_HARNESS`, `run_oracle_main` y `assert_oracle_parity`. **Conservar**
  `Outcome`, `run_podtrack`, `reset_root`, `snapshot` y `material_snapshot`.
- A.5.3 Los seis ficheros: retirar las ramas `── GRABACIÓN (temporal) ──`, sus `HARNESS`/`run_oracle`
  locales y dejar la matriz de casos **intacta**.
- A.5.4 Guardián de no-Python (un test nuevo), con **dos** comprobaciones:
  (a) `tests/**/*.rs` no contiene `python3`, `PODTRACK_REQUIRE_ORACLE`, `PODTRACK_ORACLE` ni
  `PODTRACK_ORACLE_SRT`; (b) ningún fichero de `tests/golden/**` contiene una ruta `/tmp/.tmp`.
  Este test es el +1 que lleva la cuenta a 340.
- A.5.5 Cabecera de cada uno de los seis ficheros: el golden se grabó con el oráculo recuperado de
  `c70287f^` y **no es regenerable** (procedencia y huellas en `PLAN-008` §9.11 y en `design.md`).
- A.5.6 `grep -rn 'python3\|PODTRACK_REQUIRE_ORACLE\|PODTRACK_ORACLE' tests/` → vacío.
- A.5.7 `just check-all` en verde (`fmt` + `clippy -D warnings` + `test` + `doc`).

### A.6 Documentación veraz (fase F de `close-the-port`)

- A.6.1 `AGENTS.md`: líneas 7, 21-22, 27, 29 y la sección «Trabajo en fases» (43-53). El port ya no
  está en marcha: nada de «hoy vive en la skill», «se derriba al terminar» ni «el Python sigue siendo
  el que corre». La tabla de fases pasa a describir el estado **cerrado**.
- A.6.2 `README.md`: la descripción deja de prometer la retirada futura del script (ya ocurrió).
- A.6.3 `GIT_FLOW.md:5`: última release `v0.5.1` (hoy dice `v0.5.0`).
- A.6.4 Doc-comments de `src/lib.rs:3`, `src/error.rs:3,164` y `src/srt_stage.rs:3`: el port en
  pasado. **Cero cambio de comportamiento** en `src/` (solo comentarios).

### A.7 Verificación de cierre, auditoría, PR y release

- A.7.1 `just check-all` verde y `cargo test` sin variables → **340 / 0 / 0**.
- A.7.2 Verificación contra `PLAN-008` y este plan tarea por tarea (skill `qa-plan-verifier`).
- A.7.3 Auditoría de código con el subagente **`rust-reviewer`** sobre el diff completo (tests y
  documentación; `src/` no cambia). Los hallazgos vuelven a `@rust-writer` o a un dev, nunca se
  parchean a mano.
- A.7.4 `just spec-validate close-the-port` en estricto.
- A.7.5 PR `feature/close-the-port` → `development` (conventional commits, gitmoji). Nada directo en
  `development`.
- A.7.6 Release: PR `development` → `main`. Los commits son `test:`/`docs:` (cero cambio de
  comportamiento), así que el bump lo decide `vampus`/`release-prepare` según los commits.
- A.7.7 `just spec-archive close-the-port` (funde el delta de `python-retirement` en
  `openspec/specs/`). — **hecho** el 2026-10-26 (archivado como `2026-10-26-close-the-port`).
- A.7.8 Renombrar `plans/PLAN-008.md` → `plans/PLAN-008-COMPLETED.md` (convención: se renombra,
  nunca se borra). — **hecho** el 2026-10-26
- A.7.9 Actualizar este plan con la evidencia de cierre (§9.6) — **hecho** el 2026-10-26.

## 4. Fase B — `episode-neighborhood`

Rama: `feature/episode-neighborhood` desde `development` (ya limpio tras A).

> El change proposal completo (proposal, design, tasks y los dos deltas de spec) se redacta **hoy**
> en `openspec/changes/episode-neighborhood/` y **espera aprobación explícita del usuario** antes de
> escribir una línea de código o de test (protocolo SDD del repo).

### B.0 SDD — aprobación (2026-10-28)

- B.0.1 `just spec-new episode-neighborhood` y los artefactos redactados el 2026-10-26.
- B.0.2 **Aprobado por el usuario el 2026-10-28** («adelante con la siguiente fase»), con la fase A
  ya mergeada y publicada (`v0.5.2`, `development` @ `61651e5`). Arranca la fase B.1.
- B.0.3 Rama `feature/episode-neighborhood` creada desde `development` @ `61651e5`.

### B.1 RED (tests que fallan primero)

- B.1.1 `tests/neighborhood.rs` (**fichero nuevo**; el repo ya usa un fichero por comando:
  `open_presentation.rs` para `open`): los escenarios del requisito —ventana centrada y marcada,
  resolución por slug, huecos rellenados con los existentes más cercanos, extremo de la parrilla,
  `--before 0`, episodio sin número fuera de la ventana, **vecindad numérica ≠ orden de parrilla**
  (fechas invertidas), marca provisional `*` con leyenda, `target` inexistente, `target` sin número
  y «no escribe nada» (contenido y `mtime` del tracker intactos, sin `.bak`).
- B.1.2 `tests/json_output.rs`: documento con `episode`, `before` y `after` en orden ascendente, con
  el inmediatamente inferior **al final** de `before` y el inmediatamente superior **al principio** de
  `after`; y el caso sin vecinos (listas vacías, claves presentes).
- B.1.3 `cargo test`: los nuevos fallan (el subcomando aún no existe) y los **340 existentes siguen
  verdes sin editarlos**.

### B.2 GREEN (implementación mínima)

- B.2.1 `src/commands/neighborhood.rs`: selector puro (`select_neighborhood`) + render de texto.
- B.2.2 `show_episodes` con marcador **opcional** de la referencia: la fila marcada sustituye el
  primer carácter de la indentación por `►`, de modo que **todas las columnas siguen alineadas**;
  sin marcador, los bytes de `list`, `list-range` y `status` son **idénticos a los de hoy** (lo
  garantizan los tests actuales, que no se editan).
- B.2.3 `src/error.rs`: variante del episodio sin número para la vecindad (código de salida 1).
- B.2.4 `src/json.rs`: `neighborhood_document` (`episode` + `before` + `after`).
- B.2.5 `src/commands/mod.rs` y `src/main.rs`: módulo, subcomando, flags `--before`/`--after` y
  despacho.
- B.2.6 `cargo test` → **340 + los nuevos** passed / 0 failed.

### B.3 REFACTOR

- B.3.1 `cargo fmt --all` y `cargo clippy --all-targets -- -D warnings` limpios.
- B.3.2 Sin `unwrap()`/`expect()`/`panic!()` en producción.
- B.3.3 Ningún test toca `/data/notas` (fixtures temporales).
- B.3.4 Pasada manual de **solo lectura** sobre el tracker real: `neighborhood` de un episodio
  publicado, de uno provisional y de uno sin vecinos a un lado.

### B.4 Documentación y cierre

- B.4.1 `docs/DEFINITION.md` §6: fila de `neighborhood` en la tabla de subcomandos (§6.1 no cambia).
- B.4.2 `README.md`: un ejemplo de uso con `neighborhood`.
- B.4.3 `just spec-validate episode-neighborhood` en estricto y `just check-all` verde.
- B.4.4 Auditoría con `rust-reviewer` sobre el diff.
- B.4.5 PR `feature/episode-neighborhood` → `development`; release **minor** (`0.5.1` → `0.6.0`,
  decisión de `vampus` pero el cambio es `feat:`).
- B.4.6 `just spec-archive episode-neighborhood` y `plans/PLAN-009.md` → `-COMPLETED.md`.

## 5. Contrato de `neighborhood` (resumen del spec aprobado hoy)

```
podtrack neighborhood <target> [--before N] [--after M] [--show <slug>] [--json]
```

- `target`: número o slug (misma resolución que `get`/`where`/`open`). Inexistente → el mismo error
  que `get` (`ERROR: Episodio '<target>' no encontrado.`), código ≠ 0.
- `--before`/`--after`: `4` por defecto, `0` válido (ventana de un solo lado), negativos rechazados
  por `clap`.
- Cabecera: `Vecindad de #<número> (antes: <a>, después: <b>):` con los vecinos **realmente
  mostrados**.
- Filas: tabla de `show_episodes`, en orden **ascendente de número**; la referencia con `►` en la
  indentación; los no publicados con `*` y la leyenda `* número provisional (se fija al publicar)`.
- Extremos y huecos: se muestran los N/M **existentes más cercanos**; nunca falla, nunca imprime
  filas vacías ni números inventados.
- Episodios **sin número**: invisibles a la ventana; si el `target` es uno de ellos → error explícito.
- `--json`: `{ schema_version: 1, show, episode, before: [...], after: [...] }`, cada lista en orden
  ascendente de número, **sin** ruta de material (para eso están `get` y `where`).
- **No escribe nada**: ni tracker, ni material, ni `.bak`.

## 6. Riesgos y trade-offs

| Riesgo | Mitigación |
|---|---|
| La vecindad **numérica** choca con el invariante del dominio (SRS §4.3: el número no ordena la parrilla) | Es una pregunta distinta —«qué números rodean a este»—, no un orden nuevo. Se documenta como divergencia deliberada en el `design.md` del change, con sus dos consecuencias: un episodio sin número es invisible y el vecindario de un número **provisional** cambia al publicarse |
| El marcador `►` contamina la tabla compartida y cambia bytes de `list`/`list-range`/`status` | El marcador es **opcional** y la fila marcada sustituye el primer carácter de la indentación (columnas alineadas, sin desplazamiento). Guarda dura: los 340 tests actuales pasan **sin editarlos** |
| Re-grabar el golden en A.3 podría tapar una divergencia no decidida | A.1 (auditoría completa) va **antes** de A.3, y su criterio de salida es la lista vacía salvo la divergencia ya decidida |
| La exclusión de `get 100`/`get 101` deja el fixture `mini` con 8 escenarios | Verificado: `tests/parity_read.rs` (154-275) cubre esas dos líneas y el caso sin slug. La cifra del golden pasa de 130 a 128 escenarios y se anota en `PLAN-008` |
| Fase A y fase B mezcladas en el árbol | Decisión 7: **en serie**. La B no arranca hasta que la A esté mergeada |
| `--before`/`--after` con `usize` y valores enormes | `clap` los acota al tipo; la ventana nunca supera los episodios existentes (se recorta sola) |

## 7. Fuera de alcance (deuda viva, no se toca aquí)

- `collect_slug_renames` en `#[doc(hidden)]`.
- `Fixture::show()` no refleja las plantillas del perfil.
- `open` pierde el `ErrorKind` de `spawn`.
- `dir: "."` → `<raíz>/.`.
- Deriva de títulos 827/831/832/836 (datos, no código).
- Higiene de `~/.agents` (5 `.bak-*` y la conmutación a `podtrack` sin commitear).
- Las 60 rutas sucias de `/data/notas` (producción del episodio, ajenas a este trabajo).
- El guardián de no-Python no mira **dentro de `tests/fixtures/**`** la palabra del intérprete
  (previsión declarada por prosa legítima de transcripciones; hoy `grep -rli python tests/fixtures/`
  → 0 y 0 ejecutables: solo 12 `.srt` y 6 `.yaml`), ni el **nombre de directorios** con los tokens de
  la familia A (`importlib`, `PODTRACK_*`), que solo se detectan en el texto de ficheros. Es **límite
  declarado** del guardián, con la evidencia que lo justifica (ver §9.7): un script plantado a mano
  bajo `fixtures/` o un directorio malintencionado quedarían fuera.

## 8. Criterios de cierre

**Fase A**

1. `cargo test` **sin variables `PODTRACK_*`** → 340 passed / 0 failed / 0 saltados.
2. `grep -rn 'python3\|PODTRACK_REQUIRE_ORACLE\|PODTRACK_ORACLE' tests/` → vacío (lo verifica el
   guardián y lo confirma el `grep` a mano).
3. `tests/golden/` sin una sola ruta temporal y con huella nueva anotada.
4. `just check-all` verde; change archivado; PR mergeado; documentación sin promesas de futuro.
5. `PLAN-008` renombrado a `-COMPLETED.md`.
6. No-regresión con datos reales: la matriz de `PLAN-007` §5 ejecutada contra `v0.5.1` (la última
   release), **solo lectura** y sin escribir nada en `/data/notas`.

**Fase B**

1. Los tests nuevos cubren **todos** los escenarios del requisito, texto y `--json`.
2. Los **340** tests previos pasan **sin editar un solo fichero de test existente** (guarda de que el
   marcador no contamina nada).
3. `just check-all` verde; `docs/DEFINITION.md` §6 con la fila nueva.
4. `rust-reviewer` sin hallazgos abiertos; PR mergeado; release `0.6.0` publicada.
5. `plans/PLAN-009.md` renombrado a `-COMPLETED.md`.

## 9. Testigo de ejecución — fase A (2026-10-26)

Rama `feature/close-the-port`, creada desde `development` @ `0c1f61a`. Cifras de cierre:
`cargo test --locked` **sin ninguna variable `PODTRACK_*`** → **340 passed / 0 failed / 0 ignorados**;
`just check-all` verde (fmt + clippy `-D warnings` + test + rustdoc) y `RUSTDOCFLAGS="-D warnings"
cargo doc --no-deps` limpio. Nada de `src/` cambió de comportamiento: el diff de `src/` no deja **ni
una** línea que no empiece por `//` (solo doc-comments, en A.6).

### 9.1 A.0 — reconstrucción del oráculo

`/tmp` se vació durante la noche, así que el oráculo se **re-reconstruyó** con la receta de
`PLAN-008` §9.7 y salió con las **huellas `sha256` idénticas** (`bb37e842…` para `podcast-tracker.py`,
`92fa51d7…` para `srt-stage.py`, `c2e3b104…` para `podcast-tracker.test.py`) en
`/tmp/opencode/oracle-recovery/`. Entorno: `python3` 3.14.8 + PyYAML 6.0.3; el módulo recuperado
expone los 17 `cmd_*`. Baseline antes de tocar nada: **338 passed / 1 failed**, con el único fallo en
`paridad_oraculo_fixture_mini` (escenario `get 100`).

### 9.2 A.1 — auditoría de divergencias, con denominador

Modo temporal `PODTRACK_AUDIT_GOLDEN` en `tests/common/golden.rs`: enumeraba divergencias en vez de
abortar en la primera y, además, **contaba los escenarios visitados** —el denominador que faltaba—,
con una línea `AUDIT-COUNT <caso>: <N> visitados, <D> con divergencia` por cada caso recorrido y
`AUDIT-SUMMARY` cuando había divergencias.

Resultado: **130 de 130 escenarios visitados en 27 casos** —la suma de `AUDIT-COUNT` cuadra con el
recuento independiente del golden, así que **no se quedó ningún escenario sin comparar**— y **solo 2
divergencias**, las dos en `parity_oracle/paridad_oraculo_fixture_mini`, índices **5** (`get 100`) y
**6** (`get 101`), con **una sola causa**: el binario imprime `Slug:` y `Ruta:` y el oráculo
recuperado no. Los otros 26 casos, byte a byte idénticos. Criterio de salida de A.1: cumplido (la
lista queda vacía tras la decisión de §9.4 de `PLAN-008`, y **no se re-grabó nada para tapar una
divergencia no decidida**).

### 9.3 A.2–A.4 — exclusión por test, re-grabación con el oráculo y GREEN

- **A.2**: la exclusión se pasa **por test** a `assert_parity` (`mini` excluye `get 100` y `get 101`;
  `v2` no excluye nada). `SPECS` siguió siendo una constante compartida: quitarlas de ahí habría
  dejado a `v2` sin dos escenarios que allí **sí** están en paridad (la salida es «no encontrado»).
- **A.3**: golden re-grabado **con el oráculo** (nunca con el binario) → **27 ficheros / 128
  escenarios** (`mini` 10 → 8; `v2` sigue con 10) / **156 709 B**; huella de árbol
  `a02b6a69d041e13163455c5e81fb46e6e2e0a42d7b1f272fd618e1753dcf851f`, que **sustituye** a la
  `6df44684…` de `PLAN-008` §9.11 (invalidada). El `diff` de huellas por fichero contra la
  instantánea previa da **una sola línea** (el fichero del fixture `mini`): la re-grabación no movió
  nada más. Segunda grabación en `/tmp/opencode/golden-run-b` y `diff -r` → **IDÉNTICO**
  (determinismo de D4). `grep -rn '/tmp/\.tmp' tests/golden/` → **0**.
- **A.4**: `cargo test --locked --no-fail-fast` **sin ninguna variable `PODTRACK_*`** → **339 passed
  / 0 failed / 0 saltados**, ningún target fallido, y `cargo fmt --all --check` limpio. Ningún test
  se salta: no queda ni una ruta de `return` temprano por oráculo ausente.

### 9.4 A.5 — retirada del arnés y guardián de no-Python

**Retirado de `tests/common/golden.rs`** (650 → **203 líneas**): `recording`, `record`, el modo de
auditoría completo (`auditing`, `CaseId`, los dos `thread_local`, `Summary` y su `Drop`, `flush`,
`visit`, `set_current`, `current_case`, `audit_target`, `write_line`, `emit`, `short`, `diff_lines`,
`audit_*`), los cinco constructores temporales de `GoldenCase` y las ramas `if auditing()` de las
cinco aserciones, conservando el camino normal intacto (mismos `assert_eq!`, mismos `panic!`).

**Endurecimiento estructural**: fuera `Serialize` y `skip_serializing_if` (`#[serde(default)]` a
secas). La deserialización es idéntica —los 27 ficheros se leen igual— y el crate **ya no puede
escribir** el golden. `join_lines` se retiró por huérfano (verificado con `grep`).

**Retirado de `tests/common/mod.rs`** (637 → **424 líneas**): `require_oracle_env`, `oracle_path`,
`python_ready`, `require_oracle`, `ORACLE_PREAMBLE`, `SRT_MAIN_HARNESS`, `oracle_srt_path`,
`require_oracle_srt`, `run_python_harness`, `MAIN_HARNESS`, `run_oracle_main`,
`assert_oracle_parity` y la rama de grabación de `assert_golden_parity`. **Conservados**: `Outcome`,
`run_podtrack`, `reset_root`, `snapshot` y `material_snapshot`.

**Los seis ficheros de test** (`parity_oracle`, `parity_write_oracle`, `parity_writer`,
`parity_surgery`, `srt_stage`, `rebase_references`): fuera las ramas `── GRABACIÓN ──`, sus
`HARNESS`/`WRITER_HARNESS`/`run_oracle` locales y los imports huérfanos. **La matriz de casos quedó
intacta** (mismos golden, mismos índices, mismo orden).

**Guardián `tests/no_python.rs`** (el +1 que llevó la cuenta de 339 a **340**): un solo `#[test]` con
tres comprobaciones —(a) ningún `*.rs` bajo `tests/` contiene el intérprete, `importlib`, ni las
variables del oráculo o de los dos modos retirados; (b) no existe ningún fichero `.py` bajo `tests/`;
(c) ningún fichero de `tests/golden/**` contiene una ruta temporal—. Las agujas se construyen con
`concat!` **a propósito**: el guardián se escanea a sí mismo, así que su propio texto no puede
contener las cadenas prohibidas. Recorre en orden determinista y hace un único `assert!` que lista
`<ruta>: <referencia>`.

**El guardián, probado por mutación** (esto es lo que demuestra que sirve):

| Mutación | Resultado |
|---|---|
| `tests/zz_probe.rs` con una referencia prohibida en un comentario | **falla** nombrando `tests/zz_probe.rs: python3` |
| `tests/zz_probe.py` (fichero vacío) | **falla** nombrando `tests/zz_probe.py` |
| `tests/golden/zz_probe.json` con una ruta temporal dentro | **falla** nombrando `tests/golden/zz_probe.json: ruta temporal sin normalizar` |

Los tres probes se borraron y el golden volvió **byte a byte** a `a02b6a69…` (27 ficheros /
156 709 B). Criterio de cierre del spec cumplido: `grep -rn` de las tres referencias sobre `tests/`
→ **vacío** (y el guardián no ensucia esos greps, porque sus agujas están fragmentadas).

### 9.5 A.6 — documentación veraz

`AGENTS.md` (línea 7, la sección «Fuente de verdad» con su tabla, la «Regla de oro», la línea del
andaime y la sección «Trabajo en fases» completa), `README.md` (línea 19), `GIT_FLOW.md` (línea 5:
`v0.5.0` → **`v0.5.1`**) y los doc-comments de `src/lib.rs`, `src/error.rs` (líneas 3 y 164) y
`src/srt_stage.rs`, con **cero** cambio de comportamiento: `git diff -U0 src/` no deja ni una línea
que no empiece por `//`.

**Ampliación de alcance (hallazgo del barrido)**: `docs/DEFINITION.md` **se contradecía**. Su §12
declaraba la retirada de Python cerrada, pero §1, §2, §10 y §11 seguían hablando del port en presente
y daban `v0.5.0` como última release. Se corrigieron esos puntos (§1, §2 completo —de «fuentes de
verdad durante el port» a fuentes **históricas** y **vivas**—, §10, §11 y el paso 2 de §12). El
barrido queda **a cero en documentos vivos**; las coincidencias que sobreviven son la lista blanca de
históricos (`CHANGELOG.md`, `plans/*-COMPLETED.md`, `openspec/changes/archive/`) y las **citas en
negativo** de los testigos y artefactos del change (`plans/PLAN-008-COMPLETED.md`, `plans/PLAN-009-COMPLETED.md`,
`openspec/changes/close-the-port/{proposal,design,tasks}.md` y su delta de spec), que enuncian el
vocabulario **para prohibirlo**, no para afirmarlo.

### 9.6 Cierre — auditoría, PR y release

Comprobado y ejecutado el **2026-10-26**, en este orden:

**A.7.1** `cargo test --locked` **sin ninguna variable `PODTRACK_*`** → **340 passed / 0 failed / 0
ignorados**, ningún target fallido; `just check-all` verde (fmt + clippy `-D warnings` + test +
rustdoc).

**A.7.2** Verificación contra este plan y contra `PLAN-008`, mecánica: los **nombres de los tests**
de los seis ficheros de paridad son **idénticos** a los del `HEAD` de partida (ninguno borrado ni
renombrado: D3), los 27 golden suman **128 escenarios** y cada etiqueta coincide con su `argv`, y
`openspec validate close-the-port --strict` → `Change 'close-the-port' is valid`.

**A.7.3** Auditoría de código con `rust-reviewer` sobre el diff completo, en **dos pasadas**: los seis
invariantes del change salieron limpios y el guardián se cerró con **14/14 sondas**. El detalle está
en **§9.7**; no queda ningún hallazgo abierto.

**A.7.4** Validación en estricto **antes** de archivar. Tras el archivado, `openspec validate --specs
--strict` → **13 passed / 0 failed**.

**A.7.5** PR `feature/close-the-port` → `development`: **[#29](https://github.com/atareao/podtrack/pull/29)**,
con cinco commits (`test(parity)`, `test(guard)`, `docs`, `docs(spec)`, `docs(plan)`), CI **verde**
(fmt, clippy, test, doc) y mergeada como **`6205c11`**. La rama se borró tras el merge, como las
anteriores.

**A.7.6** Release `development` → `main`: **[#30](https://github.com/atareao/podtrack/pull/30)**, CI
verde. `release-prepare` dedujo **patch** de los conventional commits y publicó **`v0.5.2`**: tag,
`CHANGELOG.md`, `.vampus.yml` y `Cargo.toml` a `0.5.2` y commit `chore: release v0.5.2`; `release.yml`
en verde, con el **crate publicado en crates.io** y la **GitHub Release**. `development` quedó
sincronizado en **`61651e5`**.

> El `tasks.md` del change anticipaba «no hay bump de versión»: el pipeline deduce **patch** de los
> commits `test`/`docs`, así que la release real fue la **0.5.2**, que es lo correcto.

**A.7.7** Change archivado como **`openspec/changes/archive/2026-10-26-close-the-port/`**, con el
delta de `python-retirement` fundido en `openspec/specs/python-retirement/spec.md` (+3 añadidos,
~1 modificado) y las **31 tareas** de su checklist cerradas.

**A.7.8** `plans/PLAN-008.md` → **`plans/PLAN-008-COMPLETED.md`** (renombrado, nunca borrado), con
las referencias actualizadas para que no quede ningún enlace colgando.

**A.7.9** Este plan actualizado con la evidencia de cierre.

**No-regresión end-to-end** (tarea 6.5 del change): **14/14** invocaciones **idénticas** frente al
binario de la última release de entonces (**`v0.5.1`**, compilada en un worktree sobre el tag, ambos
en release) —`stdout`, `stderr` y código de salida byte a byte—, toda la matriz de **solo lectura** y
con `/data/notas` **intacto** (61 paths sucios preexistentes, de la producción del episodio, `diff`
vacío).

**Estado final de la fase A:** suite en **340 / 0 / 0** sin variables de entorno, `tests/golden/`
congelado (27 ficheros / 128 escenarios / huella `a02b6a69…`), **cero** invocaciones de Python bajo
`tests/`, documentación sin afirmaciones desfasadas y **release `0.5.2`** publicada. La fase B
(`episode-neighborhood`) queda **pendiente de la aprobación del usuario**:
`openspec/changes/episode-neighborhood/` y este plan siguen **sin trackear**, porque la convención del
repo solo trackea planes cerrados.

### 9.7 Auditoría del change (2026-10-26)

Auditoría del `rust-reviewer` sobre el diff completo (A.7.3). Invariantes que dio por **limpios**: la
**matriz de tests no se movió** (mismos ficheros, mismos nombres, mismos índices y orden); el
**golden es de origen oráculo y de solo lectura** (los tipos no implementan `Serialize`: el crate no
puede reescribirlo); la **retirada del arnés está completa** (sin `oracle_path`, `require_oracle`,
`ORACLE_PREAMBLE` ni `SRT_MAIN_HARNESS`, y sin ninguna ruta de `return` por oráculo ausente);
**`src/` no cambia de comportamiento** (el diff no deja una línea que no empiece por `//`); y la
**documentación de los sitios tocados declara hechos verificables**, sin promesas ni futuro.

**El guardián de no-Python (D5) no pasó la auditoría.** Reproducido **fuera del árbol** con `rustc` y
un `CARGO_MANIFEST_DIR` de prueba, escaparon **nueve sondas** (lista literal del informe): `.sh`,
`.md`, `.toml`, `.json`, un shebang sin extensión, `Command::new("python")`, un directorio `x.py`, un
fichero ilegible y un directorio ilegible. Se cerraron **cuatro agujeros**:

| # | Agujero | Reparación |
|---|---|---|
| H-1 | una invocación del intérprete sin el literal `python3` (`Command::new("python")`) escapaba | aguja del intérprete por **nombre sin versión** |
| H-2 | solo se escaneaban `*.rs`, así que un script de shell bajo `tests/` pasaba | escaneo de **todo** fichero de `tests/` |
| H-3 | los errores de lectura se tragaban en silencio y el guardián pasaba en vacío ante un fichero o directorio ilegible | **un error de lectura es una infracción** |
| H-4 | un directorio llamado `*.py` no se veía | extensión del oráculo en nombre de **ficheros y directorios**, sin distinguir mayúsculas |
| **2.ª pasada** | el revisor vuelve sobre el guardián **ya reparado** y lo da por cerrado: **14/14 sondas detectadas**, ningún agujero nuevo de severidad bloqueante ni importante, nada de lo declarado limpio roto (340 / 0 / 0, golden `a02b6a69…` intacto, `src/` solo comentarios, `clippy -D warnings` y `fmt --check` limpios) | **sin reparación**: era la vuelta de verificación; el invariante queda **cerrado**, con las dos reservas menores de abajo |

El diseño resultante son **dos familias** (D5, reescrito): familia A —referencias inequívocas
(`importlib` y las cuatro variables `PODTRACK_*`) en el texto de todo fichero bajo `tests/`, con
cualquier nombre y extensión, recursivo— y familia B —el intérprete por nombre sin versión, en todo
fichero salvo `tests/fixtures/**` y `tests/golden/**`—. Tras el arreglo las **nueve** sondas se
detectan, con el fichero infractor en el mensaje y sin un solo falso positivo en el árbol real
(340 / 0 / 0).

**Las cuatro imprecisiones de los artefactos del change y de un documento vivo:**

| # | Imprecisión | Resolución |
|---|---|---|
| H-5 | el escenario «barrido final sin afirmaciones desfasadas» daba por vivos «los `plans/` **no** cerrados» y su lista blanca no cubría los testigos del change: insatisfacible por construcción, y a punto de fundirse como criterio permanente | se **amplió** el escenario (y la frase del requisito): la lista blanca admite los **artefactos del propio change** y los **testigos de sesión** (`plans/PLAN-NNN.md`) con **citas en negativo**, manteniendo que en los documentos vivos de verdad no quede ninguna afirmación del port en marcha. Un change en vuelo **necesita** nombrar lo que prohíbe |
| H-6 | el criterio de no-regresión citaba `v0.5.0` cuando la baseline real es `v0.5.1` (la última release) | `v0.5.0` → **`v0.5.1`** en `proposal.md` y `design.md`; la comprobación 14/14 se ejecutó contra `v0.5.1` |
| H-7 | `docs/DEFINITION.md` §12 decía que la skill quedó «solo con `shows/` y `SKILL.md`», pero quedan además dos backups (`SKILL.md.bak-puntos`, `SKILL.md.bak-story`) | reformulado para ser exacto **sin debilitar el invariante**: no queda ningún `.py`; quedan `shows/`, `SKILL.md` y los dos backups |
| H-9 | observación: `assert_tree` tolera un `material` ausente (`unwrap_or_default()`) en vez de hacer `panic!` como las otras aserciones | se **documentó** con un comentario declarativo en `tests/common/golden.rs` (escenarios sin material), **sin cambiar una línea de lógica** |

Con las cuatro cerradas, la auditoría de A.7.3 queda sin hallazgos abiertos: el guardián detecta las
cuatro clases de agujero y los artefactos del change no afirman nada desfasado.

**Segunda pasada (mismo día).** El `rust-reviewer` volvió sobre el guardián ya reparado y lo dio por
**cerrado**: **14/14 sondas detectadas** —las nueve originales más las que añadió el arreglo—, sin un
solo agujero nuevo de severidad bloqueante ni importante y sin romper nada de lo que había declarado
limpio (**340 / 0 / 0**, golden `a02b6a69…` intacto, `src/` solo comentarios, `clippy -D warnings` y
`fmt --check` limpios, `openspec validate close-the-port --strict` → valid). Dejó **dos reservas
menores**:

| # | Reserva | Qué se ha hecho con ella |
|---|---|---|
| N-1 | la familia B excluye `tests/fixtures/**` y hoy los datos **no** justifican esa exclusión (`grep -rli python tests/fixtures/` → 0; bajo `fixtures/` solo hay 12 `.srt` y 6 `.yaml`, **nada ejecutable**) | **se mantiene**, pero pasa de estar implícita a ser **previsión declarada** (una transcripción futura puede citar el intérprete en minúsculas en prosa); el residuo —un script plantado a mano bajo `fixtures/`— queda anotado como **límite conocido**, no como descuido. La asimetría con la familia A es deliberada: esta **sí** escanea `fixtures/`, porque las referencias inequívocas del oráculo no caben en una transcripción; la palabra del intérprete sí |
| N-2a | un fichero llamado **exactamente `.py`** (sin tallo) escapaba porque la comprobación usaba la **extensión** | **cerrado** en el código: se comprueba el **nombre completo** («termina en `.py`», sin distinguir mayúsculas) en **ficheros y directorios** |
| N-2b | un **directorio** llamado con un token de la familia A (p. ej. `tests/PODTRACK_ORACLE/`) no se detecta: los directorios no se leen como texto y solo se les mira el nombre | **observación asumida, sin reparación**: exigiría mala fe; se anota y no se arregla |

Veredicto del auditor: el invariante del guardián queda **cerrado**, con **N-1 como límite declarado**
y **N-2b como observación** (ambas acompañadas de la evidencia que explica por qué no se reparan); la
única reserva con reparación, **N-2a**, quedó cerrada en el código.

## 10. Checkpoint de ejecución — fase B (2026-10-28)

Rama `feature/episode-neighborhood` desde `development` @ `61651e5`, con la suite en **340 / 0 / 0**
(guarda dura: los 340 tests actuales no se editan; a `tests/json_output.rs` solo se le **añaden**
funciones nuevas, ninguna existente se modifica).

**Orden de ejecución y criterio de salida por paso:**

| Paso | Qué se hace | Criterio de salida medido |
|---|---|---|
| B.1 RED | `tests/neighborhood.rs` (nuevo) + dos casos nuevos en `tests/json_output.rs` | los nuevos **fallan** (el subcomando no existe) y los **340 previos pasan** |
| B.2 GREEN | `src/commands/neighborhood.rs`, marcador opcional en `show_episodes`, `Variante` de error, `neighborhood_document`, despacho en `main.rs` | `cargo test --locked` → **340 + nuevos**, 0 failed |
| B.3 REFACTOR | fmt, clippy `-D warnings`, cero `unwrap`/`expect`/`panic` en producción, pasada manual de solo lectura sobre el perfil real | `just check-all` verde |
| B.4 cierre | docs (§6 y README), auditoría `rust-reviewer`, PR, release `0.6.0`, archivado del change, plan a `-COMPLETED.md` | CI verde, `v0.6.0` publicada, `openspec/changes/episode-neighborhood/` archivado |

**Matriz RED (13 casos de `tests/neighborhood.rs`):** ventana centrada y marcada (9 filas, `►` en la
quinta, cabecera `Vecindad de #678 (antes: 4, después: 4):`); resolución por slug con la cabecera
declarando el número; huecos rellenados con los existentes más cercanos (`670, 672, 678, 681, 690`,
`antes: 2, después: 2`, sin filas para números inexistentes); extremo de la parrilla (`101` con
`antes: 1, después: 4`, sin error); `--before 0` (`antes: 0, después: 4`, cinco filas); episodio sin
número invisible a la ventana aunque su fecha caiga en medio; **vecindad numérica ≠ parrilla** con las
fechas de `678` y `679` invertidas; marca provisional `*` con su leyenda; `target` inexistente con el
error literal de `get`; `target` sin número (por slug) con error explícito y sin fallback; y «no
escribe nada» (contenido y `mtime` del tracker idénticos, sin `.bak`). **Más dos de JSON:** `episode` +
`before` + `after` con el inmediatamente inferior al final de `before` y el superior al principio de
`after`, sin `material` en ningún episodio y `schema_version: 1`; y el caso sin vecinos, con las dos
listas vacías y presentes.

**Riesgo que vigila la matriz:** el marcador `►` vive en `show_episodes`, compartida con `list`,
`list-range` y `status`. Se añade como parámetro **opcional** (`None` = sin marcador) y la guarda es
que los 340 tests actuales pasan **sin editar un solo fichero de test existente**.

## 11. Cierre de la fase B y el flake del golden (2026-10-28)

Rama `feature/episode-neighborhood` desde `development` @ `61651e5`, en serie tras la fase A (decisión 7).

### 11.1 `episode-neighborhood` — PR #31 y release `v0.6.0`

- **SDD**: `openspec/changes/episode-neighborhood/` redactado el 2026-10-26 y **aprobado por el usuario el
  2026-10-28**; `just spec-validate episode-neighborhood` en estricto → valid.
- **RED → GREEN → REFACTOR**: `tests/neighborhood.rs` (fichero nuevo, 13 casos) y dos casos nuevos en
  `tests/json_output.rs`; `src/commands/neighborhood.rs` con el selector puro `select_neighborhood`, el
  marcador `►` **opcional** en `show_episodes`, la variante de error del episodio sin número,
  `neighborhood_document` en `src/json.rs` y el despacho en `src/main.rs`.
- **Guarda dura cumplida**: los **340** tests previos pasan **sin editar un solo fichero de test
  existente**; la suite queda en **363 passed / 0 failed / 0 ignorados** (340 previos + 14 de integración
  + 9 unitarios). `just check-all` verde.
- **Auditoría `rust-reviewer`** (una pasada): **H1** —la referencia se marcaba por *número* y con números
  repetidos el `►` podía caer en la fila equivocada— **reparado** identificando el episodio por
  **identidad** (`std::ptr::eq`), con tres tests nuevos de números repetidos. H4/H5 asumidos sin acción;
  O1/O2/O3 sin reparar.
- **Documentación**: `docs/DEFINITION.md` §6 (fila de `neighborhood`) y `README.md` (ejemplo de uso).
- **PR [#31](https://github.com/atareao/podtrack/pull/31)** `feature/episode-neighborhood` →
  `development`, CI verde, mergeada como **`df5e1f6`**.
- **Change archivado** como `openspec/changes/archive/2026-10-28-episode-neighborhood/`; deltas fundidos
  (+1 requisito en `tracker-read`, ~1 en `json-output`); `openspec validate --specs --strict` → **13 / 0**.
  El `## Purpose` del delta se ignora al fundir (aviso esperado) y el de `tracker-read` se actualizó a
  mano (4.8).
- **Release**: PR [#32](https://github.com/atareao/podtrack/pull/32) `development` → `main`, mergeada como
  **`5d16745`**; `release-prepare` dedujo **minor** y publicó **`v0.6.0`** (tag, `CHANGELOG.md`,
  `Cargo.toml`, `.vampus.yml`, crate en crates.io y GitHub Release). `development` sincronizado en
  `91fe756`.

### 11.2 El flake del comparador y el hotfix `v0.6.1`

El pipeline de release **no ejecuta tests** (`release-prepare.yml` solo bumpea, escribe el changelog, hace
commit+tag y sincroniza `development`; `release.yml` solo publica), así que el flake no podía tumbar
`v0.6.0`: el único job con `cargo test` es `ci.yml`.

**El flake, con el mismo árbol y dos resultados:**

| Run | Rama | Evento | Resultado |
|---|---|---|---|
| `40645390716` | `feature/episode-neighborhood` (PR #31) | `pull_request` | fallo; re-run `attempt: 2` → `success` |
| `40655350954` | `development` | `push` de `91fe756` | `success` |
| `40655346111` | `main` | `push` de `91fe756` | `failure` |

En los dos fallos caían **los mismos dos tests** (`oraculo_migrate_material`, `oraculo_rename_material`)
con el mismo `stdout difiere` y el **multiconjunto** de líneas del plan idéntico: solo cambiaba el orden.

**Causa raíz**: `collect_descendants` (`src/material.rs`) recorre el material con `read_dir` y ordena
**estable** solo por profundidad (`Reverse(components().count())`), de modo que los empates conservan el
orden del sistema de ficheros, que nunca fue contrato del oráculo. `plan_number_renames` y
`collect_slug_renames_into` heredan ese orden; `insert`, `rename-material` y `migrate-material` lo
replican en sus rutas *dry-run*.

**Arreglo** (change `fix-golden-listing-order`, SDD completo): en `tests/common/golden.rs`, dentro de
cada tirada máxima del plan de renombrado (`      · <origen>  →  <destino>`) las líneas se agrupan en
**grupos consecutivos de igual profundidad** y cada grupo se compara como **multiconjunto**,
conservando la posición de la tirada y el orden entre grupos. El contenido, el número de líneas, la
posición de la tirada y todo lo que no es el plan siguen comparándose exactos. `tests/golden_listing_order.rs`
(nuevo, 6 casos) fija el contrato resultante.

**No** se tocó el binario (el orden de listado sigue siendo el del sistema de ficheros, fiel al oráculo)
**ni** `tests/golden/` (D1 de `close-the-port` intacto).

- Suite: **363 + 6 = 369 passed / 0 failed**; `just check-all` verde; determinismo con `TMPDIR` por
  defecto y `/dev/shm` (369/0 en `tmpfs` y 10 pasadas alternando sobre los cuatro binarios de paridad
  afectados, 57/0 en cada una).
- **Auditoría `rust-reviewer`**: sin hallazgos bloqueantes ni mayores; los menores (redacción de D2/D3, la
  fuerza real del unitario `plan_ordenado_de_mas_profundo_a_menos`, la independencia del predicado del
  test y el riesgo latente de `slug_collisions`) quedaron corregidos o anotados.
- **PR [#33](https://github.com/atareao/podtrack/pull/33)** `hotfix/golden-listing-order` → `main` (la vía
  que define `GIT_FLOW.md` para producción), CI verde, mergeada como **`4db696e`**.
- **Release patch `v0.6.1`** (`chore: release v0.6.1`, **`6514ad9`**): tag, `CHANGELOG.md`
  (`### Bug Fixes`), `Cargo.toml`/`.vampus.yml` a `0.6.1`, crate en crates.io y GitHub Release. **El CI de
  `main` quedó verde**: run `40753974938` `success` (antes, ese mismo escenario en `v0.6.0` había dado
  `failure`); `development` (`40753983858`) `success`; workflow Release `40753977380` con Preflight,
  Publish crate y GitHub Release en `success`.
- **Change archivado** como `openspec/changes/archive/2026-10-28-fix-golden-listing-order/` (PR
  [#34](https://github.com/atareao/podtrack/pull/34) → `development`, mergeado como `cd599c0`); delta
  fundido en `openspec/specs/python-retirement/spec.md` (~1 requisito, +4 escenarios);
  `openspec validate --specs --strict` → **13 / 0**.

### 11.3 Criterios de cierre de la fase B (§8)

| # | Criterio | Evidencia |
|---|---|---|
| 1 | Los tests cubren todos los escenarios del requisito, texto y `--json` | 13 casos en `tests/neighborhood.rs` + 2 en `tests/json_output.rs` |
| 2 | Los 340 previos pasan sin editar un test existente | 363 / 0 / 0, con los 340 intactos |
| 3 | `just check-all` verde y `docs/DEFINITION.md` §6 con la fila nueva | hecho |
| 4 | `rust-reviewer` sin hallazgos abiertos; PR mergeado; release `0.6.0` publicada | H1 reparado; PR #31 y #32; `v0.6.0` en crates.io |
| 5 | `plans/PLAN-009.md` renombrado a `-COMPLETED.md` | este cierre |

**Trabajo no previsto**: el flake del comparador del golden (§11.2), que destapó el propio CI al publicar
`v0.6.0` y que se cerró con un hotfix y una release patch (`v0.6.1`).
