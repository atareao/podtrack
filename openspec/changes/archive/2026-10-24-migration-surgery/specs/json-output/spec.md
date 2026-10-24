# Spec Delta

## Purpose

Extender la salida estructurada `--json` (aditiva y opt-in) a los **comandos de
cirugía**: `renumber`, `rename-material`, `backfill-slugs`, `migrate-material` y
`rebase-references`. Sin `--json`, la salida de texto NO SHALL cambiar.

## ADDED Requirements

### Requirement: Los comandos de cirugía emiten un documento `--json`

Con `--json`, cada comando de cirugía SHALL emitir en `stdout` un único documento
JSON con `schema_version`, `show`, `action`, `changes` (lista de las entidades
afectadas) y `dry_run`. Los avisos que en texto irían a `stderr` (p. ej.
episodios sin slug, hooks sin título) SHALL representarse dentro del documento y
NO SHALL romper el contrato de un único JSON en `stdout`.

#### Scenario: `renumber --json` lista los episodios renumerados
- **GIVEN** una invocación `renumber --from 843 --delta 1 --json`
- **WHEN** el comando se ejecuta con éxito
- **THEN** `stdout` SHALL ser un único JSON válido
- **AND** SHALL incluir `action` de renumeración y en `changes` los pares `#old → #new`

#### Scenario: `backfill-slugs --json` lista los slugs asignados
- **GIVEN** una invocación `backfill-slugs --json`
- **WHEN** el comando se ejecuta con éxito
- **THEN** el JSON SHALL incluir en `changes` los episodios con su `slug` asignado

#### Scenario: `migrate-material --json` refleja las colisiones
- **GIVEN** una invocación `migrate-material --json` con una carpeta en colisión
- **WHEN** el comando termina
- **THEN** el JSON SHALL representar la colisión en su carga útil
- **AND** el código de salida SHALL seguir siendo `1`

#### Scenario: `--dry-run --json` marca la previsualización
- **GIVEN** una invocación `--dry-run --json`
- **WHEN** el comando se ejecuta
- **THEN** el documento SHALL incluir `dry_run: true`
- **AND** NO SHALL escribirse el tracker ni el material

#### Scenario: sin `--json`, la salida de texto no cambia
- **GIVEN** una invocación sin `--json` de un comando de cirugía
- **WHEN** el comando se ejecuta con éxito
- **THEN** `stdout` SHALL contener la salida de texto byte a byte, sin JSON
