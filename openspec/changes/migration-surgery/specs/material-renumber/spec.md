# Spec Delta

## Purpose

Extender el renombrado del material de guion (la maquinaria de tokens de número
`0NNN`/`eNNN`) con el subcomando `rename-material`: aplicarlo en bloque a todos
los episodios `>= <from>` con un `--delta`, y añadir el modo `--check` que
**detecta** desajustes entre las carpetas numéricas en disco y el tracker sin
tocar nada.

## ADDED Requirements

### Requirement: `rename-material` reutiliza el renombrado por número

`rename-material --from <num> --delta <±d>` SHALL renombrar, para cada número
`>= from` presente en el tracker, el material de guion por la misma maquinaria
de tokens de número que `insert` (`0NNN`/`eNNN`, acotados por límites
alfanuméricos). NO SHALL reescribir el texto de los `.md`. El orden de proceso
SHALL ser descendente si `delta > 0` (para no pisar destinos) y ascendente si
`delta < 0`.

#### Scenario: se renombra el material de los episodios afectados
- **GIVEN** una carpeta `0843/` con `e843-articulo.md` y el umbral `--from 843 --delta 1`
- **WHEN** se ejecuta `rename-material`
- **THEN** la carpeta SHALL pasar a `0844` y el fichero a `e844-articulo.md`

#### Scenario: la raíz es el directorio padre del tracker
- **GIVEN** el tracker en `<dir>/podcast-tracker.yaml`
- **WHEN** se ejecuta `rename-material`
- **THEN** el renombrado SHALL operar sobre el material bajo `<dir>`

### Requirement: `rename-material` exige `--from` y `--delta` o `--check`

`rename-material` sin `--check` SHALL exigir `--from` y `--delta`; si falta
alguno, SHALL fallar por `stderr` con un código distinto de cero. SI
`--delta == 0`, SHALL fallar por `stderr` con un código distinto de cero.

#### Scenario: faltan from/delta
- **GIVEN** una invocación `rename-material` sin `--from`, `--delta` ni `--check`
- **WHEN** se ejecuta
- **THEN** SHALL error por `stderr` con un código distinto de cero

### Requirement: `rename-material` reporta el recuento final sin dry-run

Sin `--dry-run`, tras aplicar los renombrados, `rename-material` SHALL imprimir
el resumen `✓ N elemento(s) renombrado(s).`; si no hubo ningún par, SHALL
imprimir `(sin material que renombrar)` en lugar de una lista vacía.

#### Scenario: resumen de renombrados
- **GIVEN** un `rename-material` con material que renombrar
- **WHEN** termina sin `--dry-run`
- **THEN** `stdout` SHALL terminar con `✓ N elemento(s) renombrado(s).`
- **AND** con `--dry-run` NO SHALL imprimir ese resumen

### Requirement: `rename-material --check` detecta carpetas numéricas huérfanas

Con `--check`, `rename-material` SHALL recorrer las entradas del árbol de
guiones, considerar las que casan **exactamente** con cuatro dígitos (`\d{4}`) y
listar como **desajuste** aquellas cuyo número no esté presente en el tracker.
NO SHALL tocar el disco. Si no hay desajustes, SHALL informar de que el material
está alineado.

#### Scenario: carpeta numérica sin episodio
- **GIVEN** una carpeta `0999/` y un tracker sin el número `999`
- **WHEN** se ejecuta `rename-material --check`
- **THEN** `stdout` SHALL listar `0999` como desajuste indicando que el número `999` no está en el tracker

#### Scenario: material alineado
- **GIVEN** un árbol cuyas carpetas numéricas casan todas con episodios del tracker
- **WHEN** se ejecuta `rename-material --check`
- **THEN** `stdout` SHALL informar de que el material está alineado con el tracker
