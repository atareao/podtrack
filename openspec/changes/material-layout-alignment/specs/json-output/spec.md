# Spec Delta

## Purpose

Alinear el campo de ruta de material de los documentos `--json` con la regla única de resolución.

## MODIFIED Requirements

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
