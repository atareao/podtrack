# python-retirement Specification

## Purpose
Definir el invariante de **Python cero** en la skill `podcast-srt`: que no quede
ningún `.py`, que el tooling se invoque como `podtrack` y que el binario esté
instalado de forma invocable.

## Requirements

### Requirement: La skill `podcast-srt` no contiene ficheros Python

El árbol `~/.agents/skills/podcast-srt/` NO SHALL contener ningún fichero `.py`, ni
el andamiaje que los acompaña (`pytest.ini`, `__pycache__/`, `.pytest_cache/`,
`.ruff_cache/`).

#### Scenario: criterio de cierre mecánico
- **GIVEN** la skill `podcast-srt` tras la retirada
- **WHEN** se ejecuta `grep -r '\.py' ~/.agents/skills/podcast-srt/`
- **THEN** NO SHALL devolver ninguna coincidencia

#### Scenario: el andamiaje también desaparece
- **GIVEN** la skill tras la retirada
- **WHEN** se listan sus ficheros
- **THEN** NO SHALL existir `podcast-tracker.py`, `podcast-tracker.test.py`, `srt-stage.py`, `pytest.ini` ni caches de `pytest`/`ruff`

### Requirement: El tooling se invoca como `podtrack`

El `SKILL.md` de `podcast-srt` y los agentes del podcast SHALL invocar el binario
`podtrack` en lugar de `python3 …/{podcast-tracker,srt-stage}.py`. La bandera
`--show <slug>` SHALL seguir usándose como hoy (bandera global, antes o después del
subcomando).

#### Scenario: el SKILL usa el binario
- **GIVEN** el `SKILL.md` tras la conmutación
- **WHEN** se buscan invocaciones de los scripts
- **THEN** NO SHALL aparecer `python3 …/podcast-tracker.py` ni `python3 …/srt-stage.py`
- **AND** las invocaciones SHALL ser `podtrack <subcomando>` y `podtrack srt-stage`

#### Scenario: los agentes usan el binario
- **GIVEN** los agentes del podcast tras la conmutación
- **WHEN** se buscan invocaciones de los scripts
- **THEN** NO SHALL aparecer ninguna invocación `python3 …/*.py`
- **AND** SHALL invocar `podtrack`

### Requirement: El binario está instalado e invocable, y encuentra los perfiles

`podtrack` SHALL instalarse con `cargo install --path . --locked`, invocable como
`podtrack` desde el entorno de los agentes sin ruta absoluta al árbol de trabajo.
El directorio de perfiles de la skill SHALL quedar accesible por la cascada del
binario (enlace en `~/.config/podtrack/shows`), de modo que `podtrack --show <slug>`
encuentre `shows/<slug>.yaml` sin variables de entorno.

#### Scenario: invocación directa
- **GIVEN** el binario instalado
- **WHEN** se ejecuta `podtrack --version`
- **THEN** SHALL responder con la versión, sin `python3`

#### Scenario: resuelve el perfil por la cascada
- **GIVEN** los perfiles de la skill enlazados en `~/.config/podtrack/shows`
- **WHEN** se ejecuta `podtrack status --show atareao-con-linux`
- **THEN** SHALL mostrar el estado del tracker sin `PODTRACK_SHOWS_DIR`

### Requirement: Constancia del traslado de capacidades

La retirada SHALL dejar constancia en `~/.agents/openspec` de que las capacidades que vivían en los
scripts (parrilla, cirugía de material, clasificación de etapas del SRT) residen ahora en el binario
`podtrack`.

Además, el repositorio `podtrack` SHALL dejar constancia de la procedencia del golden de paridad:
grabado **una sola vez** con el oráculo recuperado del histórico de `~/.agents` en `c70287f^`, y sin
paridad viva desde entonces. Esa constancia SHALL figurar en la cabecera de los seis ficheros de
tests afectados y en `design.md`.

#### Scenario: la constancia existe

- **GIVEN** el repo `~/.agents`
- **WHEN** se revisa su `openspec`
- **THEN** SHALL existir constancia de que las capacidades pasaron a `podtrack`

#### Scenario: la procedencia del golden consta en el repositorio

- **GIVEN** el repositorio `podtrack` tras el change
- **WHEN** se revisa la cabecera de los seis ficheros de tests afectados y `design.md`
- **THEN** SHALL citarse el oráculo recuperado de `c70287f^` como origen del golden
- **AND** SHALL declararse que ya no hay paridad viva
- **AND** NO SHALL afirmarse que el golden se generó desde el binario

### Requirement: La suite de tests no invoca Python ni depende del oráculo

Ningún test del repositorio SHALL invocar `python3`, ni un `.py` del oráculo, ni leer las variables
`PODTRACK_REQUIRE_ORACLE`, `PODTRACK_ORACLE` o `PODTRACK_ORACLE_SRT`. La suite SHALL ejecutarse
completa con `cargo test` sin ninguna variable de entorno: ningún test SHALL reportarse como
`passed` sin ejecutar sus aserciones, y NO SHALL existir ninguna ruta de salto por oráculo ausente.

Un guardián mecánico SHALL fallar si cualquiera de esas referencias reaparece bajo `tests/`,
nombrando el fichero infractor.

#### Scenario: criterio de cierre mecánico

- **GIVEN** el árbol de tests tras el change
- **WHEN** se ejecuta `grep -rn 'python3\|PODTRACK_REQUIRE_ORACLE\|PODTRACK_ORACLE' tests/`
- **THEN** NO SHALL devolver ninguna coincidencia

#### Scenario: la suite corre completa sin variables

- **GIVEN** el árbol tras el change
- **WHEN** se ejecuta `cargo test` sin ninguna variable `PODTRACK_*`
- **THEN** SHALL ejecutarse la suite completa
- **AND** SHALL terminar con 0 fallos
- **AND** NINGÚN test SHALL saltarse por falta de oráculo

#### Scenario: el guardián falla si el Python reaparece

- **GIVEN** un fichero bajo `tests/` que vuelve a invocar `python3`, cita un `.py` del oráculo o lee
  una variable `PODTRACK_ORACLE*`
- **WHEN** se ejecuta `cargo test`
- **THEN** el guardián SHALL fallar
- **AND** el mensaje SHALL nombrar el fichero infractor y la referencia encontrada

### Requirement: La paridad byte a byte se conserva como golden congelado de origen oráculo

Los 27 casos que hoy dependen del oráculo SHALL comparar contra un golden versionado en
`tests/golden/`, con un fichero JSON por caso que contenga el código de salida, la salida estándar y
la salida de error; y, donde hoy se comparan, el tracker resultante y el árbol de material. Para el
writer canónico, el golden SHALL contener los bytes del YAML.

El golden SHALL haberse grabado con el **oráculo recuperado** del histórico de `~/.agents`
(`c70287f^`), NO con el binario. SHALL ser de solo lectura para el binario: ninguna ejecución de
tests SHALL generarlo ni actualizarlo. Un golden ausente SHALL fallar el test, nunca saltarlo. Su
lectura NO SHALL introducir ninguna dependencia de Python.

La comparación de `stdout` y `stderr` SHALL ser **exacta y en orden**, con una única excepción
acotada: dentro de una **tirada máxima** de líneas del plan de renombrado
(`      · <origen>  →  <destino>`), las líneas SHALL compararse como **multiconjunto dentro de cada
grupo consecutivo de igual profundidad**, conservando la posición de la tirada, el orden **entre**
grupos y la separación entre bloques de la misma profundidad. Esa no-determinación la
introduce `collect_descendants` (`src/material.rs`), que hereda el orden del sistema de ficheros
(`read_dir`) al ordenar **estable** solo por profundidad. El número y el contenido de las líneas del
plan, la posición de cada tirada y el orden **entre** grupos de profundidad SÍ SHALL seguir
comparándose; solo se canoniza el orden entre líneas del mismo grupo. NO SHALL relajarse ninguna otra
línea —en particular, NO SHALL relajarse los registros de `rebase-references`
(`      · <ruta>:<línea>`), que van ordenados a propósito y cuyo orden SÍ es contrato— ni la
comparación de `tracker`, `material` o `yaml`.

#### Scenario: los 27 comprueban contra el golden

- **GIVEN** un caso de la matriz de paridad
- **WHEN** se ejecuta su test
- **THEN** SHALL comparar `code`, `stdout` y `stderr` contra `tests/golden/<área>/<caso>.json`
- **AND** SHALL comparar el tracker y el material donde el caso ya lo hacía
- **AND** el test SHALL ejecutar aserciones reales, no un `return` temprano

#### Scenario: el golden es de origen oráculo

- **GIVEN** el golden grabado
- **WHEN** se revisa su procedencia
- **THEN** SHALL constar que se grabó con el oráculo de `c70287f^`
- **AND** NO SHALL constar que se generó desde el binario

#### Scenario: un golden ausente falla

- **GIVEN** un caso cuyo fichero de golden no existe
- **WHEN** se ejecuta su test
- **THEN** SHALL fallar con un mensaje que nombre el fichero que falta
- **AND** NO SHALL reportarse como `passed`
- **AND** NO SHALL saltarse

#### Scenario: el binario no regenera el golden

- **GIVEN** el árbol con el golden versionado
- **WHEN** se ejecuta la suite completa
- **THEN** `tests/golden/` SHALL permanecer sin cambios byte a byte

#### Scenario: el writer canónico conserva sus bytes

- **GIVEN** un caso del writer canónico
- **WHEN** se compara el YAML escrito con el golden
- **THEN** los bytes SHALL ser idénticos a los que produjo el oráculo

#### Scenario: el orden entre hermanos de una tirada no depende del sistema de ficheros

- **GIVEN** un caso cuyo `stdout` contiene una tirada del plan de renombrado
- **WHEN** se permutan entre sí las líneas de esa tirada que comparten profundidad
- **THEN** la comparación contra el golden SHALL pasar
- **AND** el resultado NO SHALL depender del orden en que el sistema de ficheros devuelva los hermanos

#### Scenario: un plan con una línea de más, de menos o alterada falla

- **GIVEN** un caso cuyo `stdout` contiene una tirada del plan de renombrado
- **WHEN** se duplica, se elimina o se altera el destino de una de sus líneas
- **THEN** la comparación SHALL fallar

#### Scenario: una línea que cambia de grupo de profundidad falla

- **GIVEN** un caso cuya tirada del plan de renombrado tiene grupos de profundidad distintos
- **WHEN** se intercambia una línea de un grupo con una de otro grupo
- **THEN** la comparación SHALL fallar

#### Scenario: lo que no es el plan se compara en orden exacto

- **GIVEN** una salida cuyas líneas no son del plan de renombrado (por ejemplo, los registros de `rebase-references` o una cabecera)
- **WHEN** se permutan entre sí
- **THEN** la comparación SHALL fallar

### Requirement: La documentación del repositorio declara el estado real

La documentación del repositorio SHALL declarar el estado real: el port terminó y las cinco fases
están archivadas. `AGENTS.md` NO SHALL afirmar que el script Python vive en la skill, ni que el
andaime está por derribar, ni que el Python sigue siendo el que corre; SHALL presentar las cinco
fases como archivadas y remitir a `openspec/changes/archive/`. `README.md` NO SHALL prometer la
retirada futura del script. `GIT_FLOW.md` SHALL citar la última release real (`v0.5.1`). Los
doc-comments de `src/lib.rs`, `src/error.rs` y `src/srt_stage.rs` NO SHALL describir el port en
presente ni prometer trabajo futuro. Un barrido final SHALL confirmar que no queda ninguna
afirmación desfasada en los documentos **vivos**; los artefactos del propio change y los testigos de
sesión (`plans/PLAN-NNN.md`) quedan fuera de esa exigencia, porque un change en vuelo **necesita**
nombrar lo que prohíbe.

#### Scenario: `AGENTS.md` describe el estado final

- **GIVEN** el `AGENTS.md` tras el change
- **WHEN** se revisan sus líneas 7, 21-22, 27, 29 y la sección 43-53
- **THEN** NO SHALL afirmarse que el script Python vive en la skill
- **AND** NO SHALL afirmarse que el andaime está por derribar
- **AND** NO SHALL afirmarse que el Python sigue siendo el que corre
- **AND** las cinco fases SHALL presentarse como archivadas, con remisión a `openspec/changes/archive/`

#### Scenario: `README.md` y `GIT_FLOW.md` sin promesas ni versiones viejas

- **GIVEN** el `README.md` y el `GIT_FLOW.md` tras el change
- **WHEN** se revisan la línea 19 del primero y la línea 5 del segundo
- **THEN** `README.md` NO SHALL prometer la retirada futura del script
- **AND** `GIT_FLOW.md` SHALL citar `v0.5.1` como última release

#### Scenario: los doc-comments hablan del port en pasado

- **GIVEN** los doc-comments de `src/lib.rs:3`, `src/error.rs:3,164` y `src/srt_stage.rs:3`
- **WHEN** se leen tras el change
- **THEN** NO SHALL describir el port en presente
- **AND** NO SHALL prometer trabajo futuro

#### Scenario: barrido final sin afirmaciones desfasadas

- **GIVEN** el repositorio tras el change
- **WHEN** se busca el vocabulario del port en marcha («hoy vive», «durante el port», «se derriba»,
  «sigue siendo el que corre», «v0.5.0»)
- **THEN** NO SHALL quedar ninguna coincidencia en los documentos **vivos de verdad**, que incluyen
  al menos `AGENTS.md`, `README.md`, `GIT_FLOW.md`, `docs/DEFINITION.md`, los doc-comments de `src/`
  y cualquier documento que **afirme** el estado del proyecto
- **AND** las únicas coincidencias permitidas SHALL estar en la lista blanca de documentos
  **históricos**: `CHANGELOG.md`, `plans/*-COMPLETED.md`, `openspec/changes/archive/` y
  `openspec/specs/python-retirement/spec.md` (que **cita** los `.py` para prohibirlos: «NO SHALL
  contener ningún fichero `.py`»)
- **AND** SHALL admitirse también, sin ser afirmaciones desfasadas, los **artefactos del propio
  change** (`openspec/changes/close-the-port/`) y los **testigos de sesión** (`plans/PLAN-NNN.md`),
  donde se admiten **citas en negativo** —el vocabulario entrecomillado para prohibirlo o para
  corregirlo— y **no** afirmaciones: un change en vuelo **necesita** poder nombrar lo que prohíbe
