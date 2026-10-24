# Spec Delta

## Purpose

Definir `rebase-references`, la reescritura de los **hooks estructurales**
`Próximo episodio: NNN — <título>` del material de los episodios **no
publicados**, para dejar el título sin número (las referencias van por título, no
por número). La **prosa** numérica NO SHALL reescribirse: solo SHALL reportarse.

## ADDED Requirements

### Requirement: `rebase-references` reescribe los hooks con título

`rebase-references` SHALL recorrer los `.md` (recursivo) de la carpeta de cada
episodio no publicado con `slug`, y para cada línea que case el hook estructural
`Pr[oó]ximo episodio` con un número de 3–4 dígitos seguido de un separador
(`—`, `–`, `:` o `-`) y un título, SHALL eliminar el número y el separador
conservando el título y el resto de la línea. La etiqueta SHALL admitir la
variante sin tilde (`Proximo episodio`).

#### Scenario: se elimina el número del hook
- **GIVEN** una línea `## Próximo episodio: 844 — Encuadernar apuntes`
- **WHEN** se ejecuta `rebase-references`
- **THEN** la línea SHALL quedar como `## Próximo episodio: Encuadernar apuntes`

#### Scenario: se admite la etiqueta sin tilde
- **GIVEN** una línea `Proximo episodio: 844 - Algo`
- **WHEN** se ejecuta `rebase-references`
- **THEN** SHALL reescribirse eliminando número y separador

### Requirement: `rebase-references` no reescribe la prosa numérica

Una mención a un número en texto normal (que no sea un hook estructural) NO SHALL
modificarse. Los números de 3–4 dígitos en prosa que correspondan a un episodio
**no publicado** del tracker SHALL **reportarse** por `stdout` (fichero, línea y
contexto) para revisión manual, limitando la muestra por fichero.

#### Scenario: la prosa se reporta pero no se toca
- **GIVEN** una línea de prosa que menciona el número `844` de un episodio no publicado
- **WHEN** se ejecuta `rebase-references`
- **THEN** la línea NO SHALL cambiar
- **AND** SHALL reportarse bajo `Referencias en prosa (no se tocan):`

### Requirement: `rebase-references` reporta hooks sin título y devuelve 1

Un hook que case la etiqueta `Pr[oó]ximo episodio` con un número pero **sin
título** NO SHALL reescribirse: SHALL reportarse bajo `Hooks sin título (revisar a
mano):` y el comando SHALL devolver código de salida `1`. Si no hay hooks sin
título, SHALL devolver `0`.

#### Scenario: hook sin título implica exit 1
- **GIVEN** una línea `## Próximo episodio: 844` sin título
- **WHEN** se ejecuta `rebase-references`
- **THEN** la línea NO SHALL cambiar
- **AND** SHALL reportarse como hook sin título
- **AND** el comando SHALL salir con `1`

### Requirement: `rebase-references` solo procesa episodios no publicados con slug

`rebase-references` SHALL saltar los episodios `published` y los que no tienen
`slug`, y SHALL saltar las carpetas de slug inexistentes en disco. NO SHALL
tocar el tracker.

#### Scenario: los publicados no se procesan
- **GIVEN** un episodio publicado con un hook en su material
- **WHEN** se ejecuta `rebase-references`
- **THEN** su material NO SHALL reescribirse

### Requirement: `rebase-references` escribe de forma atómica, es idempotente y respeta `--dry-run`

Las reescrituras SHALL aplicarse con escritura atómica (fichero temporal + replace)
y SOLO en los ficheros que cambiaron. Una segunda ejecución SHALL no reescribir
nada. Con `--dry-run`, SHALL imprimir el aviso de dry-run y los mismos informes,
SIN escribir ningún `.md`.

#### Scenario: segunda pasada sin cambios
- **GIVEN** un material ya rebasado
- **WHEN** se ejecuta `rebase-references`
- **THEN** SHALL informarse `(sin hooks que reescribir)`

#### Scenario: dry-run no escribe los .md
- **GIVEN** un material con un hook reescribible
- **WHEN** se ejecuta `rebase-references --dry-run`
- **THEN** `stdout` SHALL listar la reescritura
- **AND** el fichero `.md` NO SHALL cambiar
