# Spec Delta

## Purpose

Reapuntar las tres superficies de lectura que informan de la ruta del material (`where`, `get` y
`status`) a la regla única de resolución.

## MODIFIED Requirements

### Requirement: `where` imprime la ruta del material sin crear nada

`where` SHALL imprimir la ruta del material de guion del episodio, calculada con la **regla única**
de resolución de material (la carpeta es `<raíz de material>/<dir>`, con `dir` =
`episodes.scripts.dir` renderizado y `{slug}` por defecto). La ruta SHALL ser **absoluta** y NO SHALL
crear el directorio. Si el episodio no tiene `slug` o no existe, SHALL informarlo como un caso
tratado.

#### Scenario: imprime la ruta y no crea el directorio
- **GIVEN** un episodio con `slug: la-memoria-de-valet` y sin material creado
- **WHEN** se invoca `podtrack where la-memoria-de-valet`
- **THEN** SHALL imprimir la ruta `<raíz de material>/la-memoria-de-valet`
- **AND** esa ruta NO SHALL existir tras la invocación

#### Scenario: la ruta es absoluta aunque el perfil declare un tracker relativo
- **GIVEN** un perfil que declara `tracker:` como ruta relativa
- **WHEN** se invoca `podtrack where <episodio>`
- **THEN** la ruta impresa SHALL ser absoluta

#### Scenario: perfil con plantilla numérica
- **GIVEN** un perfil con `episodes.scripts.dir: "episodio-{n:02d}"` y un episodio de número `7`
- **WHEN** se invoca `podtrack where <episodio>`
- **THEN** SHALL imprimir `<raíz de material>/episodio-07`

#### Scenario: sin slug es un caso tratado, no `material: null`
- **GIVEN** un episodio sin `slug`
- **WHEN** se invoca `podtrack where <número> --json`
- **THEN** SHALL salir con un código distinto de cero
- **AND** `stderr` SHALL ser un documento de error (`{"error": …}`)
- **AND** NO SHALL emitirse `material: null` en `stdout`

### Requirement: `get` muestra el detalle de un episodio

`get` SHALL aceptar un número o un `slug` y SHALL mostrar el detalle del episodio correspondiente. Un
episodio inexistente SHALL ser un error con código distinto de cero.

Además, la salida de `get` SHALL incluir el `slug` del episodio y la ruta de su carpeta de material,
calculada con la **regla única** de resolución de material (`<raíz de material>/<dir>`), SIN
comprobar si esa ruta existe. Esa ruta SHALL ser **absoluta** (`<raíz de material>` es el directorio
padre del `tracker:` del perfil, resuelto a absoluto por la resolución de perfil). Si el episodio no
tiene `slug`, SHALL mostrar `—` tanto en el `slug` como en la ruta. Las líneas preexistentes del
bloque de detalle SHALL permanecer inalteradas y las nuevas SHALL añadirse a continuación.

#### Scenario: se resuelve por slug
- **GIVEN** un episodio con `slug: la-memoria-de-valet`
- **WHEN** se invoca `podtrack get la-memoria-de-valet`
- **THEN** SHALL mostrar los datos de ese episodio
- **AND** SHALL mostrar su `slug` (`la-memoria-de-valet`)
- **AND** SHALL mostrar la ruta **absoluta** de su material `<raíz de material>/la-memoria-de-valet`

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
- **THEN** SHALL mostrar la ruta `<raíz de material>/la-memoria-de-valet`
- **AND** NO SHALL crear el directorio
- **AND** NO SHALL informar de la existencia o inexistencia de la ruta

#### Scenario: la ruta es absoluta aunque el perfil declare un tracker relativo
- **GIVEN** un perfil que declara `tracker:` como ruta relativa
- **WHEN** se invoca `podtrack get <episodio>`
- **THEN** la línea de la ruta SHALL ser absoluta

#### Scenario: perfil con plantilla numérica
- **GIVEN** un perfil con `episodes.scripts.dir: "episodio-{n:02d}"` y un episodio de número `7`
- **WHEN** se invoca `podtrack get <episodio>`
- **THEN** la línea de la ruta SHALL ser `<raíz de material>/episodio-07`

### Requirement: `status` resume la parrilla y señala el siguiente a publicar

`status` SHALL mostrar el resumen del show (temporada, totales por estado, último publicado) y un
bloque **«Siguiente a publicar»** con el episodio no publicado más antiguo por fecha, su `slug` y la
ruta de su material, calculada con la **regla única** de resolución de material. Si no hay episodios
por publicar, SHALL omitir ese bloque.

Además, `status` SHALL cerrar con una **ventana** de hasta **8 filas** compuesta por el **último
publicado** (por fecha) seguido de los **7 siguientes a publicar** (episodios no publicados, en orden
ascendente de publicación). La ventana SHALL respetar estas reglas:

- El último publicado SHALL ir primero y NO SHALL llevar marca provisional.
- Los siguientes SHALL ir después, en orden ascendente de fecha, y SÍ SHALL llevar la marca
  provisional (`*`) y la leyenda `* número provisional (se fija al publicar)`.
- Si no hay ningún publicado, la ventana SHALL contener solo los siguientes.
- Si no hay ningún episodio por publicar, la ventana SHALL contener solo el último publicado.
- Si no hay ni publicado ni próximos, SHALL imprimirse `No hay episodios.`

#### Scenario: hay pendientes
- **GIVEN** un tracker cuyo no publicado más antiguo es `#843` con `slug: la-memoria-de-valet`
- **WHEN** se invoca `podtrack status`
- **THEN** la salida SHALL incluir una línea `▶ Siguiente a publicar: #843 «<título>»`
- **AND** SHALL mostrar su `slug` y la ruta de su material

#### Scenario: no hay pendientes
- **GIVEN** un tracker en el que todos los episodios están publicados
- **WHEN** se invoca `podtrack status`
- **THEN** la salida NO SHALL incluir el bloque «Siguiente a publicar»

#### Scenario: la ventana abre con el último publicado y sigue con los próximos
- **GIVEN** un tracker con los publicados `#100` (2026-09-01) y `#105` (2026-09-15) y los no
  publicados `#106` (2026-09-18) y `#107` (2026-09-22)
- **WHEN** se invoca `podtrack status`
- **THEN** la primera fila de la ventana SHALL ser `#105` (el publicado más reciente)
- **AND** NO SHALL llevar marca provisional
- **AND** las filas siguientes SHALL ser `#106` y `#107`, en ese orden, ambas con marca `*`
- **AND** `#100` NO SHALL aparecer en la ventana

#### Scenario: sin publicados la ventana muestra solo los próximos
- **GIVEN** un tracker sin ningún episodio publicado y con al menos un no publicado
- **WHEN** se invoca `podtrack status`
- **THEN** la ventana SHALL contener los no publicados en orden ascendente, con marca `*`
- **AND** NO SHALL contener ninguna fila sin marca

#### Scenario: sin próximos la ventana muestra solo el último publicado
- **GIVEN** un tracker en el que todos los episodios están publicados
- **WHEN** se invoca `podtrack status`
- **THEN** la ventana SHALL contener únicamente el último publicado, sin marca

#### Scenario: la ventana nunca supera 8 filas
- **GIVEN** un tracker con muchos episodios publicados y más de 7 no publicados
- **WHEN** se invoca `podtrack status`
- **THEN** la ventana SHALL contener exactamente 1 publicado y 7 próximos como máximo

#### Scenario: perfil con plantilla numérica
- **GIVEN** un perfil con `episodes.scripts.dir: "episodio-{n:02d}"` y un episodio no publicado de
  número `7`
- **WHEN** se invoca `podtrack status`
- **THEN** el bloque «Siguiente a publicar» SHALL mostrar la ruta
  `<raíz de material>/episodio-07`
