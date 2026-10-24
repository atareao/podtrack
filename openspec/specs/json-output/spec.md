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

Todo documento `--json` SHALL incluir la clave `show` con el slug del show resuelto y una carga útil propia del comando. Para `status`, la carga útil SHALL incluir el siguiente a publicar con su `number`, `slug`, `date` y la ruta de su material.

#### Scenario: `status --json` expone el siguiente a publicar
- **GIVEN** un no publicado `#843` con `slug: la-memoria-de-valet`
- **WHEN** se invoca `podtrack status --json`
- **THEN** el JSON SHALL incluir `show: "atareao-con-linux"`
- **AND** `next_to_publish` SHALL incluir `number: 843`, `slug: "la-memoria-de-valet"` y su `material`

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
