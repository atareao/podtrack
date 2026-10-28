# Spec Delta

## MODIFIED Requirements

### Requirement: El documento identifica el show y su carga útil

Todo documento `--json` SHALL incluir la clave `show` con el slug del show resuelto y una carga útil
propia del comando. Para `status`, la carga útil SHALL incluir el siguiente a publicar con su
`number`, `slug`, `date` y la ruta de su material. Para `get`, la carga útil SHALL incluir `episode`
con `number`, `slug`, `season`, `date`, `title`, `status` y `material` (la ruta del material del
episodio, calculada con la **regla única** de resolución de material, `<raíz de material>/<dir>`, que
SHALL ser absoluta cuando el `tracker:` del perfil lo sea; o `null` si el episodio no tiene `slug`).
Para `neighborhood`, la carga útil SHALL incluir `episode` (el episodio de referencia) y las listas
`before` y `after` (los vecinos anteriores y siguientes), con los campos `number`, `slug`, `season`,
`date`, `title` y `status` en cada episodio y **sin** ruta de material. Ambas listas SHALL ir en orden
ascendente de `number`, de modo que el **último** elemento de `before` es el vecino inmediatamente
inferior y el **primero** de `after` el inmediatamente superior. Las dos claves SHALL estar siempre
presentes, aunque la lista esté vacía.

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

#### Scenario: `neighborhood --json` parte la vecindad en anteriores y siguientes
- **GIVEN** un tracker con episodios de números `676`, `677`, `678`, `679` y `680`
- **WHEN** se invoca `podtrack neighborhood 678 --json`
- **THEN** `episode.number` SHALL ser `678`
- **AND** `before` SHALL contener los números `676` y `677`, con `677` en último lugar
- **AND** `after` SHALL contener los números `679` y `680`, con `679` en primer lugar
- **AND** `schema_version` SHALL seguir siendo `1`
- **AND** ningún episodio SHALL incluir la clave `material`

#### Scenario: `neighborhood --json` sin vecinos deja listas vacías
- **GIVEN** un tracker con un solo episodio numerado
- **WHEN** se invoca `podtrack neighborhood <ese número> --json`
- **THEN** `before` y `after` SHALL ser listas vacías
- **AND** ninguna de las dos claves SHALL omitirse del documento
