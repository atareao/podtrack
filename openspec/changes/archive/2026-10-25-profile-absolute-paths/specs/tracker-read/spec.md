# Spec Delta

## MODIFIED Requirements

### Requirement: `get` muestra el detalle de un episodio

`get` SHALL aceptar un número o un `slug` y SHALL mostrar el detalle del episodio correspondiente. Un episodio inexistente SHALL ser un error con código distinto de cero.

Además, la salida de `get` SHALL incluir el `slug` del episodio y la ruta de su carpeta de material (`<scripts.root>/<slug>`), SIN comprobar si esa ruta existe. Esa ruta SHALL ser **absoluta** (`<scripts.root>` es el directorio padre del `tracker:` del perfil, resuelto a absoluto por la resolución de perfil). Si el episodio no tiene `slug`, SHALL mostrar `—` tanto en el `slug` como en la ruta. Las líneas preexistentes del bloque de detalle SHALL permanecer inalteradas y las nuevas SHALL añadirse a continuación.

#### Scenario: se resuelve por slug
- **GIVEN** un episodio con `slug: la-memoria-de-valet`
- **WHEN** se invoca `podtrack get la-memoria-de-valet`
- **THEN** SHALL mostrar los datos de ese episodio
- **AND** SHALL mostrar su `slug` (`la-memoria-de-valet`)
- **AND** SHALL mostrar la ruta **absoluta** de su material `<scripts.root>/la-memoria-de-valet`

#### Scenario: un identificador inexistente falla
- **GIVEN** un identificador que no corresponde a ningún episodio
- **WHEN** se invoca `podtrack get no-existe`
- **THEN** SHALL salir con un código distinto de cero

#### Scenario: un episodio sin slug muestra guiones
- **GIVEN** un episodio sin `slug`
- **WHEN** se invoca `podtrack get <número>`
- **THEN** la línea del `slug` SHALL mostrar `—`
- **AND** la línea de la ruta SHALL mostrar `—`
- **AND** NO SHALL inventarse ninguna ruta de material

#### Scenario: la ruta no depende de que el directorio exista
- **GIVEN** un episodio con `slug: la-memoria-de-valet` cuyo directorio de material NO existe
- **WHEN** se invoca `podtrack get la-memoria-de-valet`
- **THEN** SHALL mostrar la ruta `<scripts.root>/la-memoria-de-valet`
- **AND** NO SHALL crear el directorio
- **AND** NO SHALL informar de la existencia o inexistencia de la ruta

#### Scenario: la ruta es absoluta aunque el perfil declare un tracker relativo
- **GIVEN** un perfil que declara `tracker:` como ruta relativa
- **WHEN** se invoca `podtrack get <episodio>`
- **THEN** la línea de la ruta SHALL ser absoluta

### Requirement: `where` imprime la ruta del material sin crear nada

`where` SHALL imprimir la ruta del material de guion del episodio (`<scripts.root>/<slug>`), que SHALL ser **absoluta**, SIN crear el directorio. Si el episodio no tiene `slug` o no existe, SHALL informarlo como un caso tratado.

#### Scenario: imprime la ruta y no crea el directorio
- **GIVEN** un episodio con `slug: la-memoria-de-valet` y sin material creado
- **WHEN** se invoca `podtrack where la-memoria-de-valet`
- **THEN** SHALL imprimir la ruta `<scripts.root>/la-memoria-de-valet`
- **AND** esa ruta NO SHALL existir tras la invocación

#### Scenario: la ruta es absoluta aunque el perfil declare un tracker relativo
- **GIVEN** un perfil que declara `tracker:` como ruta relativa
- **WHEN** se invoca `podtrack where <episodio>`
- **THEN** la ruta impresa SHALL ser absoluta
