# json-output Specification

## Purpose
Definir la salida estructurada `--json`, **aditiva y opt-in**, para que los consumidores programáticos (los agentes del pipeline) dejen de raspar la salida de texto. En esta fase es solo para los comandos de lectura; las fases de escritura la extenderán.

## Requirements

### Requirement: `--json` es opt-in y no altera la salida por defecto

Sin `--json`, la salida SHALL ser la de texto, byte a byte. Con `--json`, el binario SHALL emitir un objeto JSON en `stdout` y NO SHALL emitir la salida de texto. La adopción de `--json` SHALL ser por consumidor: ningún consumidor existente puede verse afectado por su existencia.

#### Scenario: sin el flag, texto
- **GIVEN** una invocación sin `--json`
- **WHEN** el comando se ejecuta con éxito
- **THEN** `stdout` SHALL contener la salida de texto y NO SHALL ser válido como JSON por diseño

#### Scenario: con el flag, JSON
- **GIVEN** una invocación con `--json`
- **WHEN** el comando se ejecuta con éxito
- **THEN** `stdout` SHALL contener un único documento JSON válido

### Requirement: El documento JSON está versionado

Todo documento `--json` SHALL incluir una clave `schema_version` con un entero. Los consumidores SHALL poder discriminar la forma por esa clave.

#### Scenario: la versión viaja en el documento
- **GIVEN** cualquier invocación con `--json`
- **WHEN** el comando se ejecuta con éxito
- **THEN** el JSON SHALL incluir `schema_version` con valor `1`

### Requirement: El documento identifica el show y su carga útil

Todo documento `--json` SHALL incluir la clave `show` con el slug del show resuelto y una carga útil
propia del comando. Para `status`, la carga útil SHALL incluir el siguiente a publicar con su
`number`, `slug`, `date` y la ruta de su material. Para `get`, la carga útil SHALL incluir `episode`
con `number`, `slug`, `season`, `date`, `title`, `status` y `material` (la ruta del material del
episodio, calculada con la **regla única** de resolución de material, `<raíz de material>/<dir>`, que
SHALL ser absoluta cuando el `tracker:` del perfil lo sea; o `null` si el episodio no tiene `slug`).

#### Scenario: `status --json` expone el siguiente a publicar
- **GIVEN** un tracker con un no publicado
- **WHEN** se invoca `podtrack status --json`
- **THEN** `next_to_publish` SHALL incluir `number: 843`, `slug: "la-memoria-de-valet"` y su
  `material`

#### Scenario: `get --json` fija el material del episodio
- **GIVEN** un perfil con `episodes.scripts.dir: "{slug}"` y un episodio con
  `slug: primer-episodio`
- **WHEN** se invoca `podtrack get primer-episodio --json`
- **THEN** `episode.material` SHALL ser la ruta `<raíz de material>/primer-episodio`

#### Scenario: `get --json` sin slug deja `material` a `null`
- **GIVEN** un episodio sin `slug`
- **WHEN** se invoca `podtrack get <número> --json`
- **THEN** `episode.material` SHALL ser `null`

#### Scenario: `get --json` con plantilla numérica
- **GIVEN** un perfil con `episodes.scripts.dir: "episodio-{n:02d}"` y un episodio de número `7`
- **WHEN** se invoca `podtrack get <episodio> --json`
- **THEN** `episode.material` SHALL ser la ruta `<raíz de material>/episodio-07`

### Requirement: Los errores con `--json` son JSON en `stderr`

Con `--json`, un error SHALL emitirse como un documento JSON en `stderr` con la clave `error`, y el código de salida SHALL seguir siendo distinto de cero.

#### Scenario: error en formato JSON
- **GIVEN** una invocación con `--json` sobre un tracker ausente
- **WHEN** el comando falla
- **THEN** `stderr` SHALL contener un JSON con la clave `error`
- **AND** SHALL salir con un código distinto de cero

### Requirement: Los comandos de escritura emiten un documento `--json` con la acción y su resultado

Con `--json`, cada comando de escritura SHALL emitir un documento JSON con `schema_version`, `show`, la `action` ejecutada, el episodio afectado y los cambios aplicados. Sin `--json`, la salida de texto SHALL permanecer inalterada.

#### Scenario: `publish --json` expone el número estampado
- **GIVEN** un no publicado `#843`
- **WHEN** se invoca `podtrack publish <slug> --json`
- **THEN** el JSON SHALL incluir `action: "publish"` y el `number` estampado
- **AND** SHALL ser un único documento JSON válido

### Requirement: `--dry-run` con `--json` informa sin aplicar

Con `--dry-run`, el documento `--json` SHALL describir los cambios **previstos** y SHALL marcar que no se aplicaron, sin escribir el tracker.

#### Scenario: dry-run se refleja en el documento
- **GIVEN** una invocación con `--json --dry-run`
- **WHEN** el comando termina
- **THEN** el JSON SHALL indicar que es una previsualización
- **AND** el tracker SHALL permanecer inalterado

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
