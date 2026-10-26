# PLAN-008 — `close-the-port`: cerrar el port (los 27 tests inertes y la documentación desfasada)

**Estado:** EN CURSO — pausado el 2026-10-25 tras la fase 2 (golden grabado); pendiente de una decisión (§9.4)
**Testigo:** quien retome esto lee §9 y no necesita releer la sesión.
**Fecha:** 2026-10-25
**Ámbito:** código y documentación de `podtrack`. `/data/notas` solo se lee, nunca se escribe (el golden se graba sobre fixtures sintéticos).
**Change asociado:** `openspec/changes/close-the-port/`
**Antecesor:** `plans/PLAN-007-COMPLETED.md` (regla única de la carpeta de material, release `v0.5.1`).

## 1. Punto de partida

El roadmap del proyecto está **entero**: las cinco fases declaradas en `AGENTS.md` están archivadas en `openspec/changes/archive/` (`bootstrap-core`, `write-commands`, `migration-surgery`, `srt-stage`, `retire-python`). No hay ningún change activo y no queda ni un `.py` en `~/.agents/skills/podcast-srt/`.

Al auditarlo con datos en vez de memoria aparecieron tres cabos sueltos que la fase 5 dejó atrás. Este plan cierra los dos primeros (G y F); el tercero queda documentado como deuda viva (§7).

## 2. Hallazgos medidos (2026-10-25)

### H1 — 27 de los 339 tests ya no comprueban nada

```
cargo test                              → 339 passed, 0 failed, 0 ignored
PODTRACK_REQUIRE_ORACLE=1 cargo test    → 312 passed, 27 FAILED
```

`retire-python` borró `podcast-tracker.py`, `srt-stage.py` y `podcast-tracker.test.py`, y con ellos desapareció el oráculo contra el que el arnés de paridad en vivo comparaba. El arnés hizo lo que estaba diseñado para hacer —si no hay oráculo, `return` temprano imprimiendo el motivo—, pero **ni `just check` ni el CI definen `PODTRACK_REQUIRE_ORACLE`**: esos 27 tests reportan `passed` sin ejecutar una sola aserción. Son el 8 % de la suite y, precisamente, el contraste byte a byte que era la red de seguridad más fuerte del proyecto.

Los 27 viven en **seis** ficheros: `parity_oracle.rs` (2), `parity_write_oracle.rs` (9), `parity_writer.rs` (2), `parity_surgery.rs` (6), `srt_stage.rs` (7), `rebase_references.rs` (1).

**No es un fallo de `retire-python`.** Su spec (`openspec/specs/python-retirement/spec.md`) acotaba el alcance a la skill `podcast-srt` y a los agentes, y su criterio de cierre era un `grep` sobre la skill. Los tests de `podtrack` quedaron fuera de alcance **por diseño**. Es deuda nueva.

El change que cierra esto añade además un **guardián de no-Python** (un test de integración nuevo), así que la suite resultante es de **340** tests: los 339 de arriba, ninguno saltado, más el guardián. Los 27 no se borran — se reescriben para comprobar contra el golden.

### H2 — el oráculo es recuperable

Los tres `.py` están en el histórico de `~/.agents`, en `c70287f^` (`c70287f` es el commit que los borró):

| Fichero | Bytes | Líneas |
|---|---|---|
| `podcast-tracker.py` | 65 617 | 1 813 |
| `srt-stage.py` | 23 983 | 589 |
| `podcast-tracker.test.py` | 62 166 | 1 817 |

Recuperado a `/tmp/opencode/oracle-recovery/` y verificado: el módulo carga y expone los 17 comandos (`cmd_add`, `cmd_backfill_slugs`, `cmd_get`, `cmd_insert`, `cmd_list`, `cmd_list_range`, `cmd_migrate_material`, `cmd_move`, `cmd_next`, `cmd_next_date`, `cmd_publish`, `cmd_rebase_references`, `cmd_rename_material`, `cmd_renumber`, `cmd_status`, `cmd_update`, `cmd_where`). Entorno: Python 3.14.8 + PyYAML 6.0.3.

**Este hallazgo es el que hace bueno el plan:** el golden puede grabarlo el oráculo de verdad, una última vez, en vez de ser una foto del binario comparándose consigo mismo. Entre «bytes idénticos al oráculo de referencia» y «bytes idénticos a lo que el binario hizo hoy» hay la diferencia entre paridad y tautología.

### H3 — la documentación narra el mundo *antes* del port

| Sitio | Dice | Realidad |
|---|---|---|
| `AGENTS.md:7` | el script Python «que **hoy vive** en `~/.agents/skills/podcast-srt/`» | no vive |
| `AGENTS.md:21-22` | tabla del oráculo: `podcast-tracker.py`, `podcast-tracker.test.py` | borrados |
| `AGENTS.md:27` | «Regla de oro **durante el port**» | el port terminó |
| `AGENTS.md:29` | el andaime «**se derriba** al terminar» | derribado |
| `AGENTS.md:43-53` | cinco fases y «Hasta entonces, el Python sigue siendo el que corre» | cinco fases archivadas; los cuatro agentes ya invocan `podtrack` |
| `README.md:19` | «cuando el binario lo sustituya por completo, el script se retira» | hecho |
| `GIT_FLOW.md:5` | «(última: `v0.5.0`)» | `v0.5.1` |

Más los doc-comments que hablan del port en presente: `src/lib.rs:3`, `src/error.rs:3,164`, `src/srt_stage.rs:3`.

## 3. Objetivo

Que el repositorio **diga y compruebe la verdad**: que los 27 tests vuelvan a comprobar algo (contra un golden de origen oráculo, no contra sí mismos) y que la documentación deje de describir un port en marcha.

## 4. Decisiones (aprobadas por el usuario)

| # | Decisión |
|---|---|
| **D1** | El golden lo graba el **oráculo recuperado** (`c70287f^`), no el binario. |
| **D2** | La grabación se hace **una vez** y **no se deja generador** en el repo: el código de grabación vive solo durante el change y se retira antes de commitear. Coherente con la retirada de la convención muerta `FEATURE-NNN` (`PLAN-007` §7). |
| **D3** | Los 27 se reescriben **en su sitio, con los mismos nombres**. La matriz de casos es el activo; solo cambia el lado «esperado». |
| **D4** | Un fichero JSON por caso bajo `tests/golden/`, con `code`, `stdout`, `stderr` y —donde hoy se compara— el `podcast-tracker.yaml` resultante y el árbol de material; para el writer canónico, los bytes del YAML. Legible en un `git diff`. |
| **D5** | Guardián de no-Python: un test que falle si reaparece `python3`, un `.py` del oráculo o las variables `PODTRACK_REQUIRE_ORACLE` / `PODTRACK_ORACLE` / `PODTRACK_ORACLE_SRT` en `tests/`. |
| **D6** | `status` sigue **fuera** del golden, como ya lo estaba por diseño de `status-upcoming-window`: se congela la cobertura, no se amplía. |
| **D7** | La documentación se arregla **declarando hechos verificables**, no prosa. |
| **D8** | `/data/notas` no se escribe: el golden se graba sobre fixtures sintéticos. |

## 5. Fases

### 0. Prerrequisito — reconstruir el oráculo (temporal, fuera del repo)
- Recuperar los tres `.py` de `c70287f^` a `/tmp/opencode/oracle-recovery/` y comprobar bytes/líneas.
- Verificar Python + PyYAML y los 17 `cmd_*`.
- Medir la baseline: `just check-all` verde, `cargo test` → 339/0, `PODTRACK_REQUIRE_ORACLE=1 cargo test --no-fail-fast` → 312/27 con los 27 identificados.

### 1. RED
- Definir la forma del golden (D4) y el helper de lectura.
- Reescribir los 27 en su sitio (D3) para que lean el golden.
- Comprobar que fallan **por golden ausente**.

### 2. Grabación (one-off, código temporal que NO se commitea)
- Modo temporal de grabación que serializa la salida del oráculo sobre la misma matriz.
- Grabar `tests/golden/` con el oráculo recuperado (D1).
- Revisar cobertura: las 27 y las superficies de D4.
- **Retirar el código de grabación** y verificar que no queda rastro.
- Verificar que el golden **no** se grabó desde el binario.

### 3. GREEN
- `cargo test` → **340/0** **sin** ninguna variable `PODTRACK_*` (los 339 previos + el guardián).
- Confirmar que **ningún** test se salta por falta de oráculo.

### 4. REFACTOR
- Retirar de `tests/common/mod.rs`: `oracle_path`, `python_ready`, `require_oracle`, `require_oracle_srt`, `ORACLE_PREAMBLE`, `SRT_MAIN_HARNESS` y sus dependencias; comprobar que no quedan helpers huérfanos.
- Retirar `PODTRACK_REQUIRE_ORACLE`, `PODTRACK_ORACLE`, `PODTRACK_ORACLE_SRT`.
- Guardián de no-Python (D5).
- Cabecera de cada uno de los seis ficheros: procedencia del golden y aviso de que ya no hay paridad viva.

### 5. F — documentación
- `AGENTS.md` (líneas 7, 21-22, 27, 29 y sección 43-53: sustituir «Trabajo en fases» por el estado final, con puntero a `openspec/changes/archive/`).
- `README.md:19`, `GIT_FLOW.md:5` y el barrido de afirmaciones desfasadas.
- Doc-comments de `src/lib.rs:3`, `src/error.rs:3,164`, `src/srt_stage.rs:3`.

### 6. Verificación de cierre
- `cargo test` sin variables → 340/0; `just check-all` verde.
- `grep -rn 'python3\|PODTRACK_REQUIRE_ORACLE\|PODTRACK_ORACLE' tests/` → vacío.
- `openspec validate close-the-port --strict`.
- Paridad con datos reales como no-regresión: 14/14 contra `v0.5.0` (mismo procedimiento que `PLAN-007`).
- `/data/notas` con sus 9 rutas sucias ajenas de siempre.

## 6. Lo que este change es y lo que no es

El golden deja de ser **paridad viva** y pasa a ser **regresión congelada**: evidencia de una paridad verificada *una vez* —con el oráculo recuperado— y, desde ahí, candado contra regresiones. **No** vuelve a validar contra el Python, porque el Python ya no existe. Además, la paridad byte a byte sobre datos reales ya está demostrada (14/14 contra el binario `v0.5.0`, en `PLAN-007`).

Consecuencia asumida de D2: **el golden no es regenerable**. Si algún día cambia el formato, se regenera a mano o recuperando otra vez el oráculo del histórico. Se documenta en la cabecera de los tests, para que quien lo lea mañana sepa de dónde salió.

## 7. Deuda viva (fuera de alcance)

- Envoltorio `collect_slug_renames` en `#[doc(hidden)]`, que solo usan los tests de `material_renumber`: migrarlos a `collect_slug_renames_into` (`PLAN-007` §7).
- `Fixture::show()` de `tests/common/mod.rs` no refleja las plantillas del perfil (`PLAN-007` §7).
- `open` mapea los fallos de `spawn` a `PodtrackError::Io(String)` y pierde el `ErrorKind` (`PLAN-007` §7).
- `dir: "."` produce `<raíz>/.`, aceptado sin normalizar (`PLAN-007` §7).
- Deriva de títulos tracker ↔ material publicado en 827, 831, 832 y 836: requiere un pase de `update`; es trabajo de **datos** sobre `/data/notas` (`PLAN-006` §8).
- Higiene del repo `~/.agents`: 5 ficheros `.bak-*` sin trackear en la skill y la conmutación a `podtrack` sin commitear (`SKILL.md`, `shows/README.md`, dos perfiles y `wordcraft/SKILL.md`).

## 8. Criterio de cierre

- [ ] Los 27 tests comprueban de verdad, contra golden, **sin Python** y sin ninguna variable `PODTRACK_*`.
- [ ] `cargo test` → **340 passed / 0 failed / 0 saltados** (los 339 previos, ninguno saltado, más el test guardián).
- [ ] Ninguna referencia a `python3`, a un `.py` del oráculo ni a `PODTRACK_REQUIRE_ORACLE`/`PODTRACK_ORACLE` en `tests/` (guardián).
- [ ] El golden es de origen oráculo (`c70287f^`) y su procedencia consta en las cabeceras y en este plan.
- [ ] La documentación del repo declara el estado real y no queda ninguna afirmación desfasada.
- [ ] `just check-all` verde y `openspec validate close-the-port --strict` OK.
- [ ] `/data/notas` intacto: solo lectura.

> ⚠️ **2026-10-25**: el golden está grabado y el árbol compila, pero la fase 3 no se puede cerrar
> hasta resolver **una decisión pendiente** (un escenario que diverge del oráculo de forma
> deliberada). Ver §9.4.

## 9. Testigo de sesión (2026-10-25 — pausa tras la fase 2)

### 9.1 Dónde estamos

| Fase | Estado |
|---|---|
| 0 — reconstruir el oráculo + baseline | ✅ hecha y verificada |
| 1 — RED (los 27 leen el golden) | ✅ RED exacto: 312 verdes / 27 fallos, **todos** «golden ausente» |
| 2 — grabación del golden (one-off) | ✅ grabado del oráculo, determinista; **2.4 (retirar el generador) pendiente** |
| 3 — GREEN | ⏸ **338 passed / 1 failed**: el único fallo es la decisión pendiente de §9.4 |
| 4 — REFACTOR (retirar arnés + ramas de grabación + guardián) | ⬜ pendiente |
| 5 — F (documentación del repo) | ⬜ pendiente |
| 6 — verificación de cierre | ⬜ pendiente |

Nada está commiteado: `HEAD` = `0c1f61a` (= `origin/development`). Todo el trabajo vive en el árbol
de trabajo y **es reversible con `git checkout` / borrando los ficheros sin trackear**.

### 9.2 Lo ya verificado (no hace falta repetirlo)

- **Oráculo recuperado**: `c70287f^` de `~/.agents` → `/tmp/opencode/oracle-recovery/`
  (`podcast-tracker.py` 65 617 B / 1 813 líneas, `srt-stage.py` 23 983 B / 589, `podcast-tracker.test.py`
  62 166 B / 1 817). Carga y expone los 17 `cmd_*`. Python 3.14.8 + PyYAML 6.0.3.
- **Baseline (antes del change)**: `cargo test` → 339/0; con el guardián viejo
  `PODTRACK_REQUIRE_ORACLE=1 cargo test --no-fail-fast` → 312/27.
- **RED**: `cargo test` → 312 passed / 27 failed, los 27 con «golden ausente o ilegible
  (/data/rust/podtrack/tests/golden/<área>/<caso>.json)».
- **Grabación**: modo grabación → 339/0; **27 ficheros / 130 escenarios** en `tests/golden/`.
- **Determinismo (D4)**: dos grabaciones consecutivas dan ficheros **byte a byte idénticos**
  (`diff -r` sin diferencias).
- **Higiene**: **0** líneas con ruta temporal en `tests/golden/`; 68 líneas con `<FIXTURE>`.
- **Árbol**: compila (`cargo test --no-run`), `cargo fmt --all -- --check` limpio, `clippy` sin warnings.
- **GREEN**: `cargo test` sin ninguna variable `PODTRACK_*` → **338 passed / 1 failed**.

### 9.3 Hallazgos del RED (dos; el segundo es el importante)

1. **Rutas temporales del fixture (resuelto).** El `TempDir` de cada fixture aparecía dentro del
   golden: 65 líneas de `stdout`, 3 de `stderr` y 40 `label` (los que reproducen un `argv` con la
   ruta del SRT). Congelar eso hacía el golden irreproducible. Solución: `golden::normalize()`
   sustituye la raíz del fixture por el marcador `<FIXTURE>` en `stdout`, `stderr` y el `label`.
   Solo se normaliza el prefijo correcto, así que una ruta **distinta** sigue haciendo fallar el
   caso. Documentado en `design.md` → «Detalles de borde».
2. **El binario divergió del oráculo de forma deliberada en `get` (sin resolver).** El escenario
   `get 100` de `parity_oracle/paridad_oraculo_fixture_mini` falla: el binario imprime dos líneas
   (`Slug:` y `Ruta:`) que el oráculo de `c70287f^` no imprimía. Es **deliberado y especificado**:
   change archivado `get-shows-slug-and-path` (PR #15, commit `c7a15e9`, 2026-10-25), cuya propuesta
   exige esas dos líneas («`Slug:` SHALL mostrar el `slug` del episodio; `Ruta:` SHALL mostrar la ruta
   absoluta de su carpeta de material»; si no hay `slug`, ambas muestran `—`). **Nadie lo notó porque
   ese change se mergeó con la red de paridad ya inerte** — es exactamente el defecto que este change
   repara.

   Nota de alcance medido: en el fixture `mini` divergen `get 100` y `get 101` (aparecen `Slug: —` /
   `Ruta: —`, porque los episodios del fixture no tienen `slug`); en `v2` **no** divergen, porque sus
   `get` son «episodio no encontrado». Los tests abortan en el primer assert de cada test, así que
   «338/1» significa «un fallo visible», **no** «una sola divergencia»: la lista exhaustiva se obtiene
   con el modo auditoría de §9.5.

### 9.4 Decisión resuelta: opción A (excluir la divergencia de `get`)

El usuario eligió **A** y se aplicó el **2026-10-26**:

- La exclusión es **por test**, no quitando entradas de `SPECS`: `SPECS` es una **constante
  compartida** por los dos tests de `tests/parity_oracle.rs`, y en el fixture `v2` esos dos
  escenarios **sí están en paridad** (allí los episodios no existen y la salida es «no
  encontrado», sin divergencia). Quitarlos de `SPECS` habría dejado a `v2` sin cobertura legítima.
- `assert_parity` acepta la lista de escenarios excluidos **de su propio test** (por etiqueta, el
  `argv` unido por espacios) y filtra la matriz antes de recorrerla, de modo que los índices del
  golden corresponden a la matriz **filtrada**.
- `paridad_oraculo_fixture_mini` pasa `&["get 100", "get 101"]`; `paridad_oraculo_fixture_v2` pasa
  lista vacía y conserva sus 10 escenarios.
- El comentario de cabecera de `SPECS` cita el change archivado `2026-10-25-get-shows-slug-and-path`
  (commit `c7a15e9`) como causa de la divergencia y remite a `tests/parity_read.rs` (154-275), donde
  las dos líneas `Slug:`/`Ruta:` —y el caso sin `slug` con `—`— están cubiertas a mano.

La **opción C** queda descartada: exigía marcar caso a caso `source: binary` y matizar el escenario
«el golden es de origen oráculo» del delta.

### 9.5 Paso 0 al retomar: enumerar TODAS las divergencias

Como los tests abortan en el primer assert, hace falta un modo **auditoría temporal** (se retira
junto con las ramas de grabación, en la fase 4): en `tests/common/golden.rs`, cuando exista la
variable `PODTRACK_AUDIT_GOLDEN`, los `assert_*` **imprimen** cada diferencia con su `label` y **no**
hacen `panic`, y al final se lee un recuento. Un solo `cargo test` con esa variable da la lista
completa. Esperado: `get` en `mini` (2 escenarios) y nada más; cualquier otra divergencia es un
hallazgo que se documenta igual que este.

### 9.6 Orden de trabajo al retomar

1. Decidir §9.4 (A o C).
2. Modo auditoría (§9.5) → lista completa de divergencias → aplicar la decisión.
3. **Fase 3 (GREEN)**: `cargo test` **sin** ninguna variable `PODTRACK_*` → **340 passed / 0 failed /
   0 saltados** (los 339 previos + el guardián; hoy hay 339 porque el guardián todavía no existe).
4. **Fase 4 (REFACTOR)**:
   - retirar de los seis ficheros de test las ramas marcadas `── GRABACIÓN` y sus helpers locales de
     oráculo;
   - retirar de `tests/common/mod.rs`: `require_oracle_env`, `oracle_path`, `python_ready`,
     `require_oracle`, `ORACLE_PREAMBLE`, `SRT_MAIN_HARNESS`, `oracle_srt_path`, `require_oracle_srt`,
     `run_python_harness`, `MAIN_HARNESS`, `run_oracle_main` y `assert_oracle_parity`;
     **conservar** `Outcome`, `run_podtrack`, `reset_root`, `snapshot` y `material_snapshot` (los usan
     también los tests que nunca tocaron el oráculo);
   - retirar `PODTRACK_RECORD_GOLDEN` (y el modo auditoría, si se añadió);
   - **guardián de no-Python** (D5), con **dos** comprobaciones: (a) agujas `python3`, los tres nombres
     `.py` del oráculo y `PODTRACK_ORACLE` en `tests/**/*.rs` (no en `tests/golden/`), construidas en
     tiempo de ejecución para no dispararse con sus propios literales; y (b) **cero rutas temporales
     en `tests/golden/**`** (la higiene que este change tuvo que reparar a mano).
5. **Fase 5** (documentación del repo) y **6** (verificación), como en `tasks.md`.
6. Auditoría con `rust-reviewer`, PR a `development` y release **sin bump de versión** (test/docs).

### 9.7 Cómo reconstruir el oráculo si se pierde `/tmp`

```bash
mkdir -p /tmp/opencode/oracle-recovery
git -C ~/.agents cat-file -p 'c70287f^:skills/podcast-srt/podcast-tracker.py' \
  > /tmp/opencode/oracle-recovery/podcast-tracker.py
git -C ~/.agents cat-file -p 'c70287f^:skills/podcast-srt/srt-stage.py' \
  > /tmp/opencode/oracle-recovery/srt-stage.py

# re-grabar el golden (una vez, y solo desde el oráculo)
rm -rf tests/golden
PODTRACK_RECORD_GOLDEN=1 PODTRACK_REQUIRE_ORACLE=1 \
  PODTRACK_ORACLE=/tmp/opencode/oracle-recovery/podcast-tracker.py \
  cargo test --locked --no-fail-fast
```

### 9.8 Ficheros tocados por este trabajo (sin commitear)

- Modificados: `tests/common/mod.rs`, `tests/parity_oracle.rs`, `tests/parity_write_oracle.rs`,
  `tests/parity_writer.rs`, `tests/parity_surgery.rs`, `tests/rebase_references.rs`, `tests/srt_stage.rs`.
- Nuevos sin trackear: `tests/common/golden.rs`, `tests/golden/` (27 JSON),
  `openspec/changes/close-the-port/`, `plans/PLAN-008.md`.
- `src/` **sin tocar** (salvo lo que haga la fase 5: doc-comments). `/data/notas` **sin tocar**.

### 9.9 Inventario del golden (27 ficheros / 128 escenarios tras §9.4)

Superficies que compara cada grupo: `o` = `code` + `stdout` + `stderr`; `+T`/`+M` = además el
`podcast-tracker.yaml` resultante y el árbol de material; `+A` = además el árbol completo
(`tracker` + `material`); `Y` = solo los bytes del YAML canónico (writer).

| Fichero del golden | Esc. | Sup. |
|---|---|---|
| `parity_oracle/paridad_oraculo_fixture_mini` | 8 | o |
| `parity_oracle/paridad_oraculo_fixture_v2` | 10 | o |
| `parity_write_oracle/oraculo_add_numero_explicito` | 1 | o+T+M |
| `parity_write_oracle/oraculo_add_sin_numero` | 1 | o+T+M |
| `parity_write_oracle/oraculo_add_slug_explicito_y_derivado` | 2 | o+T+M |
| `parity_write_oracle/oraculo_add_duplicado` | 1 | o+T+M |
| `parity_write_oracle/oraculo_update_numero_y_slug` | 3 | o+T+M |
| `parity_write_oracle/oraculo_publish` | 5 | o+T+M |
| `parity_write_oracle/oraculo_insert` | 4 | o+T+M |
| `parity_write_oracle/oraculo_insert_material_anidado` | 1 | o+T+M |
| `parity_write_oracle/oraculo_move` | 5 | o+T+M |
| `parity_writer/writer_parity_fixture_mini` | 1 | Y |
| `parity_writer/writer_parity_fixture_v2` | 1 | Y |
| `parity_surgery/oraculo_renumber` | 6 | o+A |
| `parity_surgery/oraculo_rename_material` | 6 | o+A |
| `parity_surgery/oraculo_rename_material_check_huerfano` | 1 | o+A |
| `parity_surgery/oraculo_backfill_slugs` | 5 | o+A |
| `parity_surgery/oraculo_backfill_idempotente` | 1 | o+A (segunda ejecución) |
| `parity_surgery/oraculo_migrate_material` | 7 | o+A |
| `srt_stage/oraculo_srt_por_contenido` | 24 | o |
| `srt_stage/oraculo_episode` | 12 | o |
| `srt_stage/oraculo_una_sola_etapa` | 3 | o |
| `srt_stage/oraculo_errores` | 6 | o |
| `srt_stage/oraculo_perfiles_rotos` | 2 | o |
| `srt_stage/oraculo_anclaje_multibyte` | 6 | o |
| `srt_stage/oraculo_intro_vacio` | 1 | o |
| `rebase_references/oraculo_rebase_references` | 5 | o+A |

Notas:
- La multiplicación de escenarios por test viene de **bucles internos** (matrices de `SPECS`, fixtures
  o episodios): por eso `oraculo_srt_por_contenido` tiene 24 y `oraculo_migrate_material` 7. **No se
  partió ningún test en varios ficheros** (D4: el golden guarda el agregado del caso).
- `status` **no aparece**: sigue fuera del golden por diseño de `status-upcoming-window` (D6).
- La identidad `<área>/<caso>` es exactamente la del nombre del test, y `idx` es la posición del
  escenario dentro del fichero, **en el orden en que el test lo compara**.
- `tests/golden/` ocupa **156 709 bytes** en total, en **27 ficheros / 128 escenarios**: las **130**
  de la primera grabación incluían los dos escenarios de `get` del fixture `mini` que §9.4 excluyó.
  Huella de árbol vigente:
  `a02b6a69d041e13163455c5e81fb46e6e2e0a42d7b1f272fd618e1753dcf851f`.

### 9.10 API del módulo del golden tras la retirada (fase 4, ejecutada el 2026-10-26)

`tests/common/golden.rs` quedó en **203 líneas** (venía de 650) y es **solo lector**:

**Lo que queda** (el camino normal de los tests): `GoldenFile`, `GoldenCase`, las cinco aserciones
(`assert_outcome`, `assert_tracker`, `assert_material`, `assert_tree`, `assert_yaml`), `read`,
`case`, `dir`, `normalize`, `lines`, `utf8`, `lines_map`.

**Lo que se retiró**: `recording`, `record`, el modo de auditoría completo (`auditing`, `CaseId`,
los dos `thread_local`, `Summary` y su `Drop`, `flush`, `visit`, `set_current`, `current_case`,
`audit_target`, `write_line`, `emit`, `short`, `diff_lines`, `audit_*`), los cinco constructores
temporales de `GoldenCase` (`outcome`, `with_tree`, `with_material`, `with_tracker`, `with_yaml`)
y las ramas `if auditing()` de las cinco aserciones. `join_lines` se retiró por **huérfano**
(verificado con `grep`).

**Endurecimiento estructural**: se fueron `Serialize` y `skip_serializing_if` (los campos opcionales
quedan con `#[serde(default)]` a secas). La **deserialización es idéntica** —los 27 ficheros se leen
igual— y ahora el crate **no puede escribir** el golden: «el binario no regenera el golden» pasa de
convención a imposibilidad estructural.

Los seis ficheros de test perdieron sus ramas
`── GRABACIÓN (temporal: se retira al cerrar el change close-the-port) ──`, sus `HARNESS`/`run_oracle`
locales y los imports huérfanos. **La matriz quedó intacta**: mismos golden, mismos índices, mismo
orden.

### 9.11 Evidencia literal (para no tener que repetir la medición)

**Huellas del oráculo recuperado** (`sha256`, para verificar que una re-recuperación es la misma):

| Fichero | sha256 |
|---|---|
| `podcast-tracker.py` | `bb37e842484e211ed174798481ad1c5f4ab4b50ca7331ac3d27a98394cb75ca3` |
| `srt-stage.py` | `92fa51d786fa87c4deb6568d11e8b92909dd4248bfe0bbf031bce513ab72c505` |
| `podcast-tracker.test.py` | `c2e3b104800d701346f90960f5896a2ce4e333c7efd37f746a503d53110a3a0f` |

**Huella del árbol del golden** (un solo hash de los 27 ficheros, ordenados por ruta):

- `6df44684ca154fed58901cce03ddec6a13958c2b4a3903f8aa61e707d69c1351` — **invalidada por A.3**:
  incluía los dos escenarios de `get` del fixture `mini` que §9.4 excluyó.
- `a02b6a69d041e13163455c5e81fb46e6e2e0a42d7b1f272fd618e1753dcf851f` — **vigente**, con
  **27 ficheros / 128 escenarios / 156 709 B**, re-grabada **con el oráculo** (nunca con el binario).

**Comandos y resultados observados** (en el orden en que se ejecutaron):

```bash
cargo test --locked --no-fail-fast                     # baseline: 339 passed / 0 failed
PODTRACK_REQUIRE_ORACLE=1 cargo test --locked --no-fail-fast   # anomalía: 312 passed / 27 failed
cargo test --locked --no-fail-fast                     # RED: 312 passed / 27 failed, todos
                                                       # «golden ausente o ilegible (tests/golden/<área>/<caso>.json)»
rm -rf tests/golden
PODTRACK_RECORD_GOLDEN=1 PODTRACK_REQUIRE_ORACLE=1 \
  PODTRACK_ORACLE=/tmp/opencode/oracle-recovery/podcast-tracker.py \
  cargo test --locked --no-fail-fast                   # grabación: 339 passed / 0 failed
diff -r /tmp/opencode/golden-run1 tests/golden          # sin diferencias -> determinista (D4)
grep -rc '/tmp/\.tmp' tests/golden/                     # 0 líneas con ruta temporal
cargo test --locked --no-fail-fast                     # GREEN: 338 passed / 1 failed
```

**El fallo que queda (la divergencia deliberada de `get`)**, literal:

```
thread 'paridad_oraculo_fixture_mini' panicked at tests/common/golden.rs:114:9:
assertion `left == right` failed: stdout difiere [get 100]
  left: [ …, "  Temporada: 8", "  ═══…", "", "" ]                      ← golden (oráculo c70287f^)
 right: [ …, "  Temporada: 8", "  Slug:      —", "  Ruta:      —",     ← binario (hoy)
          "  ═══…", "", "" ]
```

**Y la prueba de que la normalización de rutas actúa** (fallo observado *antes* de re-grabar, con el
golden viejo en un lado y la salida ya normalizada en el otro):

```
thread 'oraculo_renumber' panicked at tests/common/golden.rs:112:9:
assertion `left == right` failed: stdout difiere [--show demo renumber --from 101 --delta 5]
  left: [..., "✓ Tracker actualizado: /tmp/.tmpDEFNwo/guiones/podcast-tracker.yaml", ""]
 right: [..., "✓ Tracker actualizado: <FIXTURE>/guiones/podcast-tracker.yaml", ""]
```

Además: `right:` **nunca** conserva una ruta `/tmp/.tmp` en ninguno de los tres binarios afectados
(`parity_surgery`, `parity_write_oracle`, `srt_stage`), luego el `dir` elegido en cada fichero es el
correcto.

**Cierre de la fase A (2026-10-26)** — comandos y resultados observados, en orden:

```bash
cargo test --locked --no-fail-fast  (sin ninguna variable PODTRACK_*)
    # 339 passed / 0 failed / 0 ignorados, ningún target fallido   (A.4 GREEN)
cargo test --locked --no-fail-fast  (con tests/no_python.rs)
    # 340 passed / 0 failed / 0 ignorados                          (A.5, el guardián)
just check-all
    # verde: fmt + clippy -D warnings + test + rustdoc
grep -rn 'python3\|PODTRACK_REQUIRE_ORACLE\|PODTRACK_ORACLE' tests/   # sin coincidencias
find tests/golden -type f | sort | xargs sha256sum | sha256sum
    # a02b6a69d041e13163455c5e81fb46e6e2e0a42d7b1f272fd618e1753dcf851f
```

### 9.12 Aviso sobre `/data/notas` (no confundir con este trabajo)

Al cerrar la sesión, `git -C /data/notas status --porcelain` mostraba **60 rutas sucias**
(28 borrados, 23 sin trackear, 9 modificadas), con marcas de tiempo de las 20:38–20:48, concentradas
en `Podcasts/guiones/podman-ips-reales-y-la-liga/` (ejemplos, `presentacion-*.typ`/`.pdf`, `setup.sh`),
`Podcasts/atareao…` (2) y `transcripciones/e843.srt`, más cuatro residuos sueltos `r.out`/`r.err`/
`o.out`/`o.err`. **Son de la producción del episodio, no de este change**: ninguna es fixture, golden
ni ruta temporal, y ninguna contiene el marcador `<FIXTURE>`. Este change **no ha escrito** en
`/data/notas` (todas las pruebas usan `TempDir`, y el golden se grabó sobre fixtures sintéticos: D8).

### 9.13 Registro de la sesión (traza de auditoría)

1. **Fase 0**: oráculo recuperado de `c70287f^` y huellas comprobadas; baseline 339/0 y anomalía 312/27.
2. **Paso A**: creados `tests/common/golden.rs` (módulo y lectores) y `assert_golden_parity` en
   `tests/common/mod.rs`, sin tocar ningún test. Suite intacta: 339/0.
3. **RED, tres grupos en paralelo** (ficheros disjuntos): `parity_oracle` (2) + `parity_writer` (2) +
   `rebase_references` (1); `parity_write_oracle` (9) + `parity_surgery` (6); `srt_stage` (7). Cada uno
   verificó su target: los de oráculo fallan por golden ausente, el resto sigue verde.
4. **RED global verificado**: 312/27 y `tests/golden/` sin existir.
5. **Grabación 1** (sin normalización) → detectado el defecto de las rutas temporales (19 de 27
   ficheros).
6. **Corrección**: `golden::normalize` + `dir` en `outcome`/`assert_outcome`; punto nuevo en
   `Risks / Trade-offs` de `design.md`; migración de los 9 puntos de llamada en los seis ficheros (el
   árbol quedó unos minutos sin compilar entre medias, a propósito y documentado).
7. **Grabación 2** → quedaban 40 `label` con ruta temporal (el `label` no estaba normalizado) → se
   normaliza también el `label` (una línea, en `outcome`).
8. **Grabación ×2 y `diff -r`** → determinista; 0 rutas temporales.
9. **GREEN**: 338/1, con el fallo de `get` identificado, documentado y **sin tapar**.
10. **Documentación**: `PLAN-008` §9 (este testigo), riesgo nuevo en `design.md`, nota de pausa y
    marcas en `tasks.md`.

Nada de esto está commiteado: `HEAD` = `0c1f61a` en `development`, 0 commits nuevos, 11 entradas en
`git status --porcelain` (7 modificados + 4 sin trackear).

### 9.14 Estado final del testigo

El change `close-the-port` se ejecutó completo el **2026-10-26** en la rama
`feature/close-the-port`: decisión A aplicada (§9.4), golden re-grabado con el oráculo (27 ficheros /
128 escenarios / 156 709 B), arnés retirado, guardián de no-Python (`tests/no_python.rs`) y
documentación puesta al día. La suite queda en **340 passed / 0 failed / 0 ignorados** sin ninguna
variable de entorno y `just check-all` en verde.

El testigo de esa sesión es [`PLAN-009.md`](./PLAN-009.md) (fase A: cierre del port; fase B:
`episode-neighborhood`). Este fichero se renombra a `PLAN-008-COMPLETED.md` al cerrar el change.
