# Spec Delta

## Purpose

Definir la superficie de **solo lectura** del binario —`status`, `next`, `next-date`, `list`,
`list-range`, `get`, `where` y `neighborhood`— con el mismo contrato que la implementación de
referencia: misma salida de texto, mismos iconos y leyendas, mismos *exit codes*. Es la mitad del
contrato total y la que no toca nada.

## ADDED Requirements

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
