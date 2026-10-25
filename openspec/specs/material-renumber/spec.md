# material-renumber Specification

## Purpose
Definir el renombrado de los **tokens de número** del material de guion cuando un episodio cambia de número (`0NNN` → `0MMM`, `eNNN` → `eMMM`). Es la maquinaria que `insert` usa al desplazar la parrilla y que la fase `migration-surgery` reutilizará para `rename-material`.

## Requirements

### Requirement: Solo se reescriben los tokens de número reconocidos

El renombrado SHALL reconocer la forma rellena a cuatro dígitos (`0NNN`) y la forma con prefijo `e` (`eNNN`), acotadas por límites alfanuméricos, y NO SHALL tocar secuencias de dígitos más largas ni identificadores que las contengan.

#### Scenario: el token `0NNN` se reescribe
- **GIVEN** un material con la carpeta `0843` y el fichero `e843-articulo.md`
- **WHEN** se renombra el material de `843` a `844`
- **THEN** la carpeta SHALL pasar a `0844` y el fichero a `e844-articulo.md`

#### Scenario: no se toca una secuencia más larga
- **GIVEN** un nombre que contiene `08430`
- **WHEN** se renombra de `843` a `844`
- **THEN** ese nombre NO SHALL cambiar

### Requirement: El plan de renombrado se ordena de más profundo a menos

El renombrado SHALL planificarse como pares origen→destino ordenados **de más profundo a menos** (primero ficheros y subcarpetas, y la carpeta del episodio al final), de modo que se puedan aplicar en secuencia sin invalidar rutas intermedias.

#### Scenario: se renombran hijos antes que el padre
- **GIVEN** un árbol `0843/` con ficheros dentro
- **WHEN** se planifica el renombrado a `0844`
- **THEN** los ficheros SHALL aparecer antes que la carpeta en el plan

### Requirement: El renombrado aborta par a par si un destino ya existe

Al aplicar el plan, el binario SHALL validar cada par **por separado** (como la
implementación de referencia: `dst.exists()` par a par en
`_apply_material_renames`) y, en cuanto un destino ya exista, SHALL abortar con un
error y un código distinto de cero e informar de la ruta en conflicto por
`stderr`. NO SHALL haber atomicidad de árbol: los pares anteriores al conflicto
MAY quedar ya aplicados (el aborto es par a par, no atómico).

#### Scenario: destino ocupado en un par
- **GIVEN** un material donde un destino del plan ya existe
- **WHEN** se aplica el renombrado
- **THEN** SHALL error con un código distinto de cero e informar de la ruta en conflicto por `stderr`
- **AND** los pares anteriores al conflicto MAY quedar aplicados (no SHALL asumirse atomicidad de árbol)

### Requirement: La aplicación reporta cada renombrado

Al aplicar el plan (sin `--dry-run`), cada renombrado SHALL reportarse en `stdout` con la ruta relativa al árbol de guiones, origen y destino.

#### Scenario: se reporta cada par
- **GIVEN** un renombrado aplicado con éxito
- **WHEN** termina
- **THEN** cada par origen→destino SHALL aparecer en `stdout`

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
