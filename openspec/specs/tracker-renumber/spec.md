# tracker-renumber Specification

## Purpose
Definir `renumber`, la renumeración de la **parrilla** en bloque: desplaza los
números de todos los episodios con número `>= <from>` un `--delta` (positivo o
negativo). Es una operación **solo de números**: NO SHALL tocar fechas ni el
material en disco. El material se realinea aparte con `rename-material`.

## Requirements

### Requirement: `renumber` reescribe solo los números del tracker

`renumber --from <num> --delta <±d>` SHALL sumar `delta` al `number` de cada
episodio con `number` no nulo y `number >= from`, y NO SHALL modificar las
fechas, los estados, los títulos, los slugs ni el material en disco.

#### Scenario: se desplazan los números a partir del umbral
- **GIVEN** una parrilla con `#843`, `#844`, `#845` y el umbral `--from 843 --delta 1`
- **WHEN** se ejecuta `renumber`
- **THEN** los números SHALL pasar a `#844`, `#845`, `#846`
- **AND** los episodios con número `< 843` NO SHALL cambiar
- **AND** ninguna fecha NO SHALL cambiar

#### Scenario: un delta negativo retrocede los números
- **GIVEN** una parrilla con `#845` y el umbral `--from 845 --delta -2`
- **WHEN** se ejecuta `renumber`
- **THEN** el número SHALL pasar a `#843`

### Requirement: `renumber` rechaza `--delta 0`

`renumber` con `--delta 0` SHALL fallar con un error por `stderr` y un código de
salida distinto de cero, sin escribir nada.

#### Scenario: delta cero es error
- **GIVEN** cualquier parrilla
- **WHEN** se ejecuta `renumber --from 843 --delta 0`
- **THEN** SHALL error por `stderr` con un código distinto de cero
- **AND** el tracker NO SHALL escribirse

### Requirement: `renumber` rechaza un umbral sin episodios

`renumber` SHALL fallar con un error por `stderr` y un código de salida distinto
de cero cuando no exista ningún episodio con `number >= from`.

#### Scenario: umbral por encima del último número
- **GIVEN** una parrilla cuyo mayor número es `#845`
- **WHEN** se ejecuta `renumber --from 999 --delta 1`
- **THEN** SHALL error por `stderr` con un código distinto de cero
- **AND** el tracker NO SHALL escribirse

### Requirement: `renumber` rechaza una colisión de numeración

`renumber` SHALL detectar la colisión entre los números **no afectados** (los
`< from`) y los números **resultantes**; si el conjunto unión tiene duplicados,
SHALL fallar con un error por `stderr` y un código de salida distinto de cero,
sin escribir nada.

#### Scenario: el delta solapa con números existentes
- **GIVEN** una parrilla con `#841` no afectado y `#843` afectado con `--delta -2`
- **WHEN** se ejecuta `renumber`
- **THEN** `#843` resultaría en `#841`, que ya existe
- **AND** SHALL error por `stderr` con un código distinto de cero

### Requirement: `renumber` previsualiza con `--dry-run`

Con `--dry-run`, `renumber` SHALL imprimir la misma cabecera y las mismas líneas
`#old → #new` que en una ejecución real, SHALL imprimir el aviso de dry-run, y NO
SHALL escribir el tracker.

#### Scenario: dry-run no escribe
- **GIVEN** una parrilla con `#843`
- **WHEN** se ejecuta `renumber --from 843 --delta 1 --dry-run`
- **THEN** `stdout` SHALL listar `#843 → #844`
- **AND** el tracker NO SHALL cambiar

### Requirement: `renumber` reporta el desplazamiento y guarda el tracker

Sin `--dry-run`, tras aplicar los cambios, `renumber` SHALL guardar el tracker
por la vía canónica (backup `.bak` + escritura atómica) y SHALL haber impreso,
en orden ascendente de número antiguo, una línea `#old → #new` por episodio
afectado bajo la cabecera `✓ Renumerar desde #<from> con delta <+d>-<d>:`.

#### Scenario: se listan los cambios en orden
- **GIVEN** una parrilla con `#843` y `#845` afectados con `--delta 1`
- **WHEN** se ejecuta `renumber`
- **THEN** `stdout` SHALL mostrar `#843 → #844` antes que `#845 → #846`
- **AND** el tracker SHALL quedar guardado con los nuevos números
