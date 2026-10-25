# Spec Delta

## Purpose

Definir `backfill-slugs`, la derivación y persistencia del `slug` de los
episodios que aún no lo tienen, a partir de su título (`slugify` + desambiguación
`-2`, `-3`, …). Es **idempotente**.

## ADDED Requirements

### Requirement: `backfill-slugs` deriva el slug de los episodios sin él

Para cada episodio sin `slug`, `backfill-slugs` SHALL calcular `slugify(title)` y
persistirlo. Los episodios que ya traen `slug` NO SHALL modificarse. La
desambiguación SHALL sufijar `-2`, `-3`, … hasta encontrar un slug libre entre
los ya presentes.

#### Scenario: se asigna el slug derivado
- **GIVEN** un episodio sin slug con título `La Memoria de Valet`
- **WHEN** se ejecuta `backfill-slugs`
- **THEN** al episodio SHALL asignársele el slug `la-memoria-de-valet`

#### Scenario: un slug ya presente no se toca
- **GIVEN** un episodio con `slug: manual`
- **WHEN** se ejecuta `backfill-slugs`
- **THEN** ese slug NO SHALL cambiar

#### Scenario: colisión de slug derivado se desambigua
- **GIVEN** dos episodios sin slug con el mismo título base, uno de ellos
- **WHEN** se ejecuta `backfill-slugs`
- **THEN** al segundo SHALL asignársele el slug con sufijo `-2`

### Requirement: `backfill-slugs` avisa de títulos sin slug utilizable

Si `slugify(title)` produce una cadena vacía, `backfill-slugs` SHALL emitir un
aviso por `stderr` indicando el episodio y SHALL **omitirlo** sin abortar el
resto.

#### Scenario: título que no produce slug
- **GIVEN** un episodio sin slug cuyo título no produce slug utilizable
- **WHEN** se ejecuta `backfill-slugs`
- **THEN** SHALL avisarse por `stderr` y el episodio SHALL quedar sin slug

### Requirement: `backfill-slugs` reporta una tabla y un resumen

`backfill-slugs` SHALL imprimir una línea por slug asignado con el formato
`<num> -> <slug>` y SHALL terminar con el resumen
`N slugs asignados de M episodios`, donde `M` es el total de episodios.

#### Scenario: resumen con el total
- **GIVEN** un tracker de 10 episodios, 3 de ellos sin slug
- **WHEN** se ejecuta `backfill-slugs`
- **THEN** SHALL imprimirse `3 slugs asignados de 10 episodios`

### Requirement: `backfill-slugs` es idempotente y respeta `--dry-run`

Una segunda ejecución de `backfill-slugs` SHALL asignar `0` slugs (nada que
hacer). Con `--dry-run`, SHALL imprimir el aviso de dry-run, la misma tabla y
resumen, y NO SHALL escribir el tracker.

#### Scenario: segunda pasada sin cambios
- **GIVEN** un tracker ya con todos los slugs asignados
- **WHEN** se ejecuta `backfill-slugs`
- **THEN** SHALL imprimirse `0 slugs asignados de N episodios`

#### Scenario: dry-run no escribe
- **GIVEN** un tracker con episodios sin slug
- **WHEN** se ejecuta `backfill-slugs --dry-run`
- **THEN** el tracker NO SHALL cambiar
