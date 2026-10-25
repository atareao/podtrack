# Spec Delta

## MODIFIED Requirements

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
