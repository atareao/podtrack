# Spec Delta

## Purpose

Acotar el requisito de paridad del golden: la paridad conserva los **bytes, el contenido y el orden
por profundidad** del plan de renombrado, pero NO SHALL exigir el orden entre hermanos de la misma
profundidad, que lo decide el sistema de ficheros y nunca fue contrato del oráculo.

## MODIFIED Requirements

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
