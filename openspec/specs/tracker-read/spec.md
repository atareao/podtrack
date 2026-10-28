# tracker-read Specification

## Purpose
Definir la superficie de **solo lectura** del binario —`status`, `next`, `next-date`, `list`, `list-range`, `get`, `where` y `neighborhood`— con el mismo contrato que la implementación de referencia: misma salida de texto, mismos iconos y leyendas, mismos *exit codes*. Es la mitad del contrato total y la que no toca nada.

## Requirements

### Requirement: Los comandos de lectura no modifican nada

Ningún comando de esta capacidad SHALL escribir en el tracker, crear el material de guion ni crear el fichero `.bak`. Su única salida SHALL ser por `stdout` (éxito) o `stderr` (error).

#### Scenario: `status` no toca el fichero
- **GIVEN** un tracker con un contenido y un `mtime` conocidos
- **WHEN** se invoca `podtrack status`
- **THEN** el contenido y el `mtime` del tracker SHALL permanecer inalterados
- **AND** NO SHALL crearse ningún `.bak`

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

### Requirement: `next` informa del hueco y del número autoritativo

`next` SHALL imprimir dos líneas: el hueco provisional para planificar («Siguiente episodio») y el número autoritativo a publicar («Próximo a publicar»), calculado como `max(publicados) + 1`.

#### Scenario: hueco y autoritativo difieren
- **GIVEN** un último publicado `#842` y varios no publicados con número provisional
- **WHEN** se invoca `podtrack next`
- **THEN** SHALL mostrar «Próximo a publicar» con el número `843`

### Requirement: `next-date` calcula la próxima fecha publicable

`next-date` SHALL devolver la próxima fecha de publicación en lunes o jueves (los días del show por defecto), calculada a partir de la fecha del último episodio de la parrilla o, en su defecto, del inicio de temporada.

#### Scenario: desde un jueves el siguiente es lunes
- **GIVEN** que hoy es jueves y los días de publicación son lunes y jueves
- **WHEN** se invoca `podtrack next-date`
- **THEN** SHALL devolver el lunes siguiente

### Requirement: `list` y `list-range` marcan los no publicados

`list` y `list-range` SHALL aceptar `--status`, `--season` y `--last`, y SHALL marcar los episodios **no publicados** con un `*` al lado del número, imprimiendo al final la leyenda `* número provisional (se fija al publicar)`. Los publicados NO SHALL llevar marca.

#### Scenario: un no publicado se marca
- **GIVEN** un episodio no publicado con número `843`
- **WHEN** se invoca `podtrack list`
- **THEN** su número SHALL mostrarse como `843*`
- **AND** la leyenda SHALL imprimirse al final

#### Scenario: filtrar por estado publicado no marca nada
- **GIVEN** un tracker con episodios publicados
- **WHEN** se invoca `podtrack list --status published`
- **THEN** ningún número SHALL llevar `*`

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

### Requirement: Los errores van a `stderr` y el código de salida es contrato

En caso de error, el binario SHALL escribir el mensaje en `stderr` y SHALL salir con un código distinto de cero, reproduciendo los códigos de la implementación de referencia. En caso de éxito, SHALL salir con `0`.

#### Scenario: tracker ausente
- **GIVEN** una ruta de tracker que no existe
- **WHEN** se invoca cualquier comando de lectura
- **THEN** SHALL escribir el error en `stderr`
- **AND** SHALL salir con un código distinto de cero

### Requirement: `neighborhood` muestra la vecindad numérica de un episodio

`podtrack neighborhood <target>` SHALL resolver el episodio por número o por slug (la misma
resolución que `get`, `where` y `open`) y SHALL mostrar una ventana **centrada** en él con los
episodios que lo rodean **en el espacio numérico**: hasta `--before N` episodios con el número
inmediatamente inferior al suyo y hasta `--after M` con el número inmediatamente superior.

`--before` SHALL valer `4` por defecto y `--after` SHALL valer `4` por defecto. Ambos SHALL aceptar
`0` (ventana de un solo lado) y NO SHALL aceptar valores negativos.

Reglas de composición:

- La vecindad SHALL ser **numérica**: los vecinos SHALL elegirse por su `number`, NO por su fecha. El
  orden de la parrilla (fecha ascendente, número como desempate) NO SHALL usarse en este comando.
- Un episodio **sin número** NO SHALL aparecer en la ventana, ni entre los anteriores ni entre los
  siguientes.
- Los vecinos SHALL ser los N/M **episodios existentes más cercanos**: si la parrilla tiene huecos
  (`renumber`, `insert`, números provisionales), la ventana SHALL rellenarse con los números
  existentes más próximos y NO SHALL imprimir filas vacías, números inventados ni avisos de hueco.
- El episodio de referencia SHALL identificarse por **su posición en el tracker**, no por su número: los
  demás episodios que compartan su número NO SHALL aparecer en la ventana (no son ni anteriores ni
  siguientes) y SHALL marcarse con `►` **una sola fila**. El presupuesto de `--before`/`--after` SHALL
  contarse en episodios, no en posiciones.
- El episodio de referencia SHALL aparecer **en su sitio**, entre los anteriores y los siguientes, y
  SHALL marcarse con `►` en el primer carácter de la indentación de su fila, de modo que las columnas
  de todas las filas queden alineadas.
- Las filas SHALL ir en **orden ascendente de número**.
- Cuando existan menos vecinos que los pedidos (principio o final de la parrilla), SHALL mostrarse los
  que haya y NO SHALL ser un error.
- Los episodios no publicados SHALL llevar la marca provisional `*` y SHALL imprimirse la leyenda
  `* número provisional (se fija al publicar)`.
- La salida SHALL usar la tabla de `list`/`list-range` (la misma cabecera de columnas y el mismo
  formato de fila). La columna del marcador `►` SHALL existir **solo** en `neighborhood`: las filas de
  `list`, `list-range` y `status` SHALL permanecer inalteradas.
- La cabecera SHALL declarar el episodio de referencia y los vecinos **realmente** mostrados:
  `Vecindad de #<número> (antes: <a>, después: <b>):`.
- NO SHALL mostrarse la ruta de material de ningún episodio.
- Un `target` inexistente SHALL fallar con el error `ERROR: Episodio '<target>' no encontrado.` y un
  código de salida distinto de cero.
- Un `target` que resuelva a un episodio **sin número** SHALL fallar con un error explícito y un
  código de salida distinto de cero, y NO SHALL calcular una vecindad por fecha como alternativa.
- `neighborhood` NO SHALL escribir en el tracker, ni crear el material de guion, ni crear el fichero
  `.bak`.

#### Scenario: la ventana centra el episodio y marca la referencia
- **GIVEN** un tracker con episodios de números `674`, `675`, `676`, `677`, `678`, `679`, `680`, `681` y `682`
- **WHEN** se invoca `podtrack neighborhood 678`
- **THEN** la tabla SHALL tener `9` filas en orden ascendente de número
- **AND** la fila del `678` SHALL ser la quinta y SHALL llevar `►`
- **AND** la cabecera SHALL ser `Vecindad de #678 (antes: 4, después: 4):`

#### Scenario: el episodio de referencia se resuelve por slug
- **GIVEN** un episodio de número `678` y `slug: la-memoria-de-valet`
- **WHEN** se invoca `podtrack neighborhood la-memoria-de-valet`
- **THEN** la ventana SHALL ser la del `#678`
- **AND** la cabecera SHALL declarar `#678`

#### Scenario: los huecos se rellenan con los números existentes más cercanos
- **GIVEN** un tracker con episodios de números `670`, `672`, `678`, `681` y `690`, y sin ningún episodio con los números `673`-`677` ni `679`-`680`
- **WHEN** se invoca `podtrack neighborhood 678`
- **THEN** la tabla SHALL tener `5` filas: `670`, `672`, `678` (con `►`), `681` y `690`
- **AND** la cabecera SHALL ser `Vecindad de #678 (antes: 2, después: 2):`
- **AND** NO SHALL imprimirse ninguna fila para los números inexistentes

#### Scenario: en el extremo de la parrilla se muestra lo que haya
- **GIVEN** un tracker con episodios de números `100`, `101`, `102`, `103`, `104` y `105`
- **WHEN** se invoca `podtrack neighborhood 101`
- **THEN** la tabla SHALL tener `6` filas: `100`, `101` (con `►`), `102`, `103`, `104` y `105`
- **AND** la cabecera SHALL ser `Vecindad de #101 (antes: 1, después: 4):`
- **AND** el comando NO SHALL fallar

#### Scenario: `--before 0` deja la ventana a un solo lado
- **GIVEN** un tracker con episodios de números `674`, `675`, `676`, `677`, `678`, `679`, `680`, `681` y `682`
- **WHEN** se invoca `podtrack neighborhood 678 --before 0`
- **THEN** la tabla SHALL tener `5` filas: `678` (con `►`), `679`, `680`, `681` y `682`
- **AND** la cabecera SHALL ser `Vecindad de #678 (antes: 0, después: 4):`

#### Scenario: un episodio sin número no entra en la ventana
- **GIVEN** un tracker con episodios de números `677`, `678` y `679` y un episodio sin número cuya fecha cae entre las fechas de `677` y `678`
- **WHEN** se invoca `podtrack neighborhood 678`
- **THEN** la tabla SHALL tener `3` filas: `677`, `678` (con `►`) y `679`
- **AND** el episodio sin número NO SHALL aparecer, ni entre los anteriores ni entre los siguientes

#### Scenario: la vecindad es numérica, no de parrilla
- **GIVEN** un tracker con los episodios `677`, `678` y `679`, en el que el `679` emite el `2026-10-20` y el `678` el `2026-10-23` (fechas invertidas respecto del número)
- **WHEN** se invoca `podtrack neighborhood 678`
- **THEN** la tabla SHALL tener `3` filas, en el orden `677`, `678` (con `►`) y `679`
- **AND** el `679` SHALL aparecer entre los siguientes, por su número, aunque su fecha sea anterior
- **AND** la ventana NO SHALL reordenarse por fecha

#### Scenario: los no publicados llevan la marca provisional
- **GIVEN** un tracker con los episodios `676`, `677`, `678`, `679` y `680`, en el que el `679` no está publicado
- **WHEN** se invoca `podtrack neighborhood 678`
- **THEN** la fila del `679` SHALL mostrar su número con `*`
- **AND** SHALL imprimirse la leyenda `* número provisional (se fija al publicar)`

#### Scenario: un identificador inexistente falla
- **GIVEN** un número que no corresponde a ningún episodio
- **WHEN** se invoca `podtrack neighborhood 99999`
- **THEN** SHALL salir con un código distinto de cero
- **AND** el error SHALL ser `ERROR: Episodio '99999' no encontrado.`

#### Scenario: un episodio sin número no tiene vecindad numérica
- **GIVEN** un episodio sin número y con `slug: recien-anadido`
- **WHEN** se invoca `podtrack neighborhood recien-anadido`
- **THEN** SHALL salir con un código distinto de cero
- **AND** el error SHALL explicar que el episodio no tiene número
- **AND** el comando NO SHALL responder con una vecindad por fecha

#### Scenario: `neighborhood` no escribe nada
- **GIVEN** un tracker con un contenido y un `mtime` conocidos
- **WHEN** se invoca `podtrack neighborhood 678`
- **THEN** el contenido y el `mtime` del tracker SHALL permanecer inalterados
- **AND** NO SHALL crearse ningún `.bak`

#### Scenario: con números repetidos la referencia se marca una sola vez
- **GIVEN** un tracker con episodios de números `10`, `11`, `11`, `11`, `12`, `13`, `14` y `15`
- **WHEN** se invoca `podtrack neighborhood 11`
- **THEN** SHALL marcarse con `►` **exactamente una** fila, la del episodio resuelto
- **AND** las otras filas con el número `11` NO SHALL aparecer
- **AND** la tabla SHALL mostrar `10`, `11`, `12`, `13`, `14` y `15`, con la cabecera `Vecindad de #11 (antes: 1, después: 4):`
- **AND** el documento `--json` SHALL contener **los mismos** episodios que la tabla
