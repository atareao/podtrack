# Spec Delta

## MODIFIED Requirements

### Requirement: El documento identifica el show y su carga útil

Todo documento `--json` SHALL incluir la clave `show` con el slug del show resuelto y una carga útil propia del comando. Para `status`, la carga útil SHALL incluir el siguiente a publicar con su `number`, `slug`, `date` y la ruta de su material. Para `get`, la carga útil SHALL incluir `episode` con `number`, `slug`, `season`, `date`, `title`, `status` y `material` (la ruta del material del episodio, `<scripts.root>/<slug>`, que SHALL ser absoluta cuando el `tracker:` del perfil lo sea; o `null` si el episodio no tiene `slug`).

#### Scenario: `status --json` expone el siguiente a publicar
- **GIVEN** un no publicado `#843` con `slug: la-memoria-de-valet`
- **WHEN** se invoca `podtrack status --json`
- **THEN** el JSON SHALL incluir `show: "atareao-con-linux"`
- **AND** `next_to_publish` SHALL incluir `number: 843`, `slug: "la-memoria-de-valet"` y su `material`

#### Scenario: `get --json` fija el material del episodio
- **GIVEN** un episodio con `slug: primer-episodio`
- **WHEN** se invoca `podtrack get primer-episodio --json`
- **THEN** el JSON SHALL incluir `episode` con `number`, `slug`, `season`, `date`, `title` y `status`
- **AND** `episode.material` SHALL ser la ruta `<scripts.root>/primer-episodio`
- **AND** `schema_version` SHALL seguir siendo `1`

#### Scenario: `get --json` sin slug deja `material` a `null`
- **GIVEN** un episodio sin `slug`
- **WHEN** se invoca `podtrack get <número> --json`
- **THEN** `episode.slug` SHALL ser `null`
- **AND** `episode.material` SHALL ser `null`
