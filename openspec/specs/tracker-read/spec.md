# tracker-read Specification

## Purpose
Definir la superficie de **solo lectura** del binario —`status`, `next`, `next-date`, `list`, `list-range`, `get` y `where`— con el mismo contrato que la implementación de referencia: misma salida de texto, mismos iconos y leyendas, mismos *exit codes*. Es la mitad del contrato total y la que no toca nada.

## Requirements

### Requirement: Los comandos de lectura no modifican nada

Ningún comando de esta capacidad SHALL escribir en el tracker, crear el material de guion ni crear el fichero `.bak`. Su única salida SHALL ser por `stdout` (éxito) o `stderr` (error).

#### Scenario: `status` no toca el fichero
- **GIVEN** un tracker con un contenido y un `mtime` conocidos
- **WHEN** se invoca `podtrack status`
- **THEN** el contenido y el `mtime` del tracker SHALL permanecer inalterados
- **AND** NO SHALL crearse ningún `.bak`

### Requirement: `status` resume la parrilla y señala el siguiente a publicar

`status` SHALL mostrar el resumen del show (temporada, totales por estado, último publicado) y un bloque **«Siguiente a publicar»** con el episodio no publicado más antiguo por fecha, su `slug` y la ruta de su material. Si no hay episodios por publicar, SHALL omitir ese bloque.

Además, `status` SHALL cerrar con una **ventana** de hasta **8 filas** compuesta por el **último publicado** (por fecha) seguido de los **7 siguientes a publicar** (episodios no publicados, en orden ascendente de publicación). La ventana SHALL respetar estas reglas:

- El último publicado SHALL ir primero y NO SHALL llevar marca provisional.
- Los siguientes SHALL ir después, en orden ascendente de fecha, y SÍ SHALL llevar la marca provisional (`*`) y la leyenda `* número provisional (se fija al publicar)`.
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
- **GIVEN** un tracker con los publicados `#100` (2026-09-01) y `#105` (2026-09-15) y los no publicados `#106` (2026-09-18) y `#107` (2026-09-22)
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

`get` SHALL aceptar un número o un `slug` y SHALL mostrar el detalle del episodio correspondiente. Un episodio inexistente SHALL ser un error con código distinto de cero.

#### Scenario: se resuelve por slug
- **GIVEN** un episodio con `slug: la-memoria-de-valet`
- **WHEN** se invoca `podtrack get la-memoria-de-valet`
- **THEN** SHALL mostrar los datos de ese episodio

#### Scenario: un identificador inexistente falla
- **GIVEN** un identificador que no corresponde a ningún episodio
- **WHEN** se invoca `podtrack get no-existe`
- **THEN** SHALL salir con un código distinto de cero

### Requirement: `where` imprime la ruta del material sin crear nada

`where` SHALL imprimir la ruta del material de guion del episodio (`<scripts.root>/<slug>`) SIN crear el directorio. Si el episodio no tiene `slug` o no existe, SHALL informarlo como un caso tratado.

#### Scenario: imprime la ruta y no crea el directorio
- **GIVEN** un episodio con `slug: la-memoria-de-valet` y sin material creado
- **WHEN** se invoca `podtrack where la-memoria-de-valet`
- **THEN** SHALL imprimir la ruta `<scripts.root>/la-memoria-de-valet`
- **AND** esa ruta NO SHALL existir tras la invocación

### Requirement: Los errores van a `stderr` y el código de salida es contrato

En caso de error, el binario SHALL escribir el mensaje en `stderr` y SHALL salir con un código distinto de cero, reproduciendo los códigos de la implementación de referencia. En caso de éxito, SHALL salir con `0`.

#### Scenario: tracker ausente
- **GIVEN** una ruta de tracker que no existe
- **WHEN** se invoca cualquier comando de lectura
- **THEN** SHALL escribir el error en `stderr`
- **AND** SHALL salir con un código distinto de cero
