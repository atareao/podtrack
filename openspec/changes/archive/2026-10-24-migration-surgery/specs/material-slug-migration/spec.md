# Spec Delta

## Purpose

Definir `migrate-material`, la migración del material de guion de la convención
**numérica** (`0NNN/`) a la de **slug** (`<slug>/`). Reescribe la carpeta del
episodio y los tokens de número de sus ficheros internos, con **detección previa
de colisiones** que deja intacta la carpeta en conflicto.

## ADDED Requirements

### Requirement: `migrate-material` reescribe tokens de número a slug

Para cada episodio migrado con `slug`, el destino de su carpeta SHALL ser
`<scripts_root>/<slug>` y los nombres de los ficheros/subcarpetas internos SHALL
sustituir los tokens de número por el slug, reconociendo las formas
`episodio-NNN`, `eNNN`, `0NNN` y `NNN` (sin ceros), acotadas por límites
alfanuméricos. Las variantes que colapsan al mismo destino (`0NNN`, `NNN`,
`eNNN`) SHALL converger en el slug.

#### Scenario: la carpeta y los tokens internos pasan a slug
- **GIVEN** una carpeta `0843/` con `e843-articulo.md` y `0843-notas.md` para un episodio con `slug: mi-episodio`
- **WHEN** se ejecuta `migrate-material`
- **THEN** la carpeta SHALL pasar a `mi-episodio/`
- **AND** los ficheros internos SHALL pasar a `mi-episodio-articulo.md` y `mi-episodio-notas.md`

#### Scenario: `episodio-NNN` conserva la palabra
- **GIVEN** un fichero `episodio-843-guion.md` para un episodio con `slug: mi-episodio`
- **WHEN** se ejecuta `migrate-material`
- **THEN** el fichero SHALL pasar a `episodio-mi-episodio-guion.md`

### Requirement: `migrate-material` selecciona los episodios a migrar

Por defecto, `migrate-material` SHALL migrar **solo** los episodios cuyo `status`
NO sea `published`; con `--include-published` SHALL incluirlos. Un episodio sin
`slug` SHALL avisarse por `stderr` y **saltarse** sin abortar el resto. Un
episodio sin número o sin carpeta numérica en disco SHALL saltarse.

#### Scenario: los publicados quedan fuera por defecto
- **GIVEN** un episodio `published` con carpeta `0842/` y un no publicado con `0843/`
- **WHEN** se ejecuta `migrate-material`
- **THEN** SOLO SHALL migrarse el material del no publicado

#### Scenario: include-published los incluye
- **GIVEN** los mismos episodios
- **WHEN** se ejecuta `migrate-material --include-published`
- **THEN** SHALL migrarse también el material del publicado

#### Scenario: episodio sin slug se avisa y se salta
- **GIVEN** un episodio no publicado sin `slug` pero con carpeta `0843/`
- **WHEN** se ejecuta `migrate-material`
- **THEN** SHALL avisarse por `stderr` y la carpeta `0843/` NO SHALL renombrarse

### Requirement: `migrate-material` detecta colisiones antes de aplicar

Antes de renombrar una carpeta, `migrate-material` SHALL computar el plan
completo y detectar colisiones: un **destino ya existente** (distinto del
origen) o **varios orígenes** que convergen al **mismo destino**. Si hay alguna,
SHALL dejar **esa carpeta intacta** (no renombrar nada de ella), reportar la
colisión por `stderr` y **continuar** con las demás.

#### Scenario: dos orígenes convergen al mismo destino
- **GIVEN** un episodio con `slug: dup` y ficheros `e843-x.md` y `0843-x.md` en `0843/`
- **WHEN** se ejecuta `migrate-material`
- **THEN** ambos ficheros tendrían el destino `dup-x.md`
- **AND** la carpeta `0843/` NO SHALL renombrarse y la colisión SHALL reportarse por `stderr`

### Requirement: `migrate-material` devuelve 1 si hubo alguna colisión

`migrate-material` SHALL devolver código de salida `1` si detectó **alguna**
colisión y `0` si no. Las carpetas migradas con éxito SHALL reportarse por
`stdout` con la cabecera `✓ Migrar <0NNN> → <slug>:` y una línea por par.

#### Scenario: colisión implica exit 1
- **GIVEN** una migración donde una carpeta colisiona y otra no
- **WHEN** se ejecuta `migrate-material`
- **THEN** SHALL salir con `1`
- **AND** la carpeta sin colisión SHALL haberse migrado igualmente

### Requirement: `migrate-material --dry-run` no toca el disco

Con `--dry-run`, `migrate-material` SHALL imprimir el aviso de dry-run, reportar
las mismas cabeceras y pares y las mismas colisiones que en una ejecución real,
y NO SHALL renombrar ninguna carpeta.

#### Scenario: dry-run no renombra
- **GIVEN** una carpeta `0843/` migrable
- **WHEN** se ejecuta `migrate-material --dry-run`
- **THEN** `stdout` SHALL listar los pares sin renombrar
- **AND** la carpeta `0843/` SHALL seguir existiendo
