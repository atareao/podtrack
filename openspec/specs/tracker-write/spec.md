# tracker-write Specification

## Purpose
Definir la superficie de **escritura** de la parrilla —`add`, `update`, `publish`, `insert` y `move`— con el mismo contrato que la implementación de referencia: misma salida de texto, mismos *exit codes* y el mismo tracker resultante. Es la mitad que muta los datos, y donde vive la semántica del número provisional frente al autoritativo.

## Requirements

### Requirement: Toda escritura usa el escritor canónico y confirma en stdout

Cada comando que muta el tracker SHALL persistir mediante el escritor canónico (`save_tracker`: cabecera, orden de campos, backup `.bak` y escritura atómica) y SHALL imprimir su confirmación de guardado. Ningún comando SHALL escribir el fichero por otra vía.

#### Scenario: se crea el backup y el tracker queda reordenado
- **GIVEN** un tracker existente al que se le añade un episodio
- **WHEN** el comando termina con éxito
- **THEN** SHALL existir `<tracker>.yaml.bak` con el contenido previo
- **AND** el tracker SHALL quedar reescrito en el formato canónico y ordenado por fecha

### Requirement: `add` da de alta un episodio `scripted` con número opcional

`add` SHALL aceptar el título y, opcionalmente, un número (`add <num> "<título>"` o `add "<título>"`). Sin número SHALL asignar el siguiente hueco de parrilla (`max(number de todos) + 1`) como número **provisional**. Con un número explícito ya ocupado SHALL fallar. SHALL derivar el `slug` del título (sufijándolo si colisiona) salvo que se pase `--slug`. La fecha por defecto SHALL ser la siguiente publicable y la temporada, la actual.

#### Scenario: sin número se asigna el hueco provisional y se deriva el slug
- **GIVEN** un tracker cuyo mayor número es `868` y sin `slug` que colisione
- **WHEN** se invoca `add "Mi episodio"`
- **THEN** el episodio SHALL crearse con `number: 869`, `status: scripted` y un `slug` derivado del título

#### Scenario: número explícito duplicado falla
- **GIVEN** un tracker que ya tiene un episodio `#843`
- **WHEN** se invoca `add 843 "Otro"`
- **THEN** SHALL salir con un código distinto de cero
- **AND** NO SHALL modificar el tracker

### Requirement: `update` cambia el estado de un episodio

`update` SHALL resolver el episodio por número o por `slug` y SHALL cambiar su `status` a uno de los estados válidos. Un estado inválido o un objetivo inexistente SHALL ser un error con código distinto de cero, sin escribir.

#### Scenario: cambio de estado
- **GIVEN** un episodio `scripted`
- **WHEN** se invoca `update <slug> --status recorded`
- **THEN** su `status` SHALL pasar a `recorded` y SHALL persistirse

#### Scenario: objetivo inexistente no escribe
- **GIVEN** un identificador que no corresponde a ningún episodio
- **WHEN** se invoca `update no-existe --status recorded`
- **THEN** SHALL salir con un código distinto de cero
- **AND** el tracker SHALL permanecer inalterado

### Requirement: `publish` estampa el número público autoritativo

`publish` SHALL fijar el `number` del episodio como `max(number de los publicados) + 1` (o `1` si no hay ninguno) y su `status` como `published`. El número **NO SHALL reutilizarse**. Si el episodio traía un provisional distinto, SHALL informarlo. Si **otro** episodio no publicado ocupaba ya ese número, SHALL anularle el número (`number: null`). Un episodio ya publicado SHALL ser un error.

#### Scenario: se estampa `max(publicados)+1`
- **GIVEN** un último publicado `#842` y un no publicado con provisional `#869`
- **WHEN** se invoca `publish <slug>`
- **THEN** el episodio SHALL quedar `published` con `number: 843`
- **AND** SHALL informarse del cambio `provisional #869 → #843`

#### Scenario: no se reutiliza un número ya publicado
- **GIVEN** un episodio ya publicado con `#843`
- **WHEN** se publica otro
- **THEN** el nuevo SHALL recibir `#844`, nunca `#843`

### Requirement: `insert` desplaza números, fechas y material

`insert --before <num> --title "<título>"` SHALL colocar el episodio nuevo en el número y la fecha del que hoy ocupa `<num>`, SHALL avanzar `number + 1` y un *slot* de fecha a todos los de número `>= <num>`, y SHALL renombrar el material asociado de los desplazados. Una fecha aportada que no coincida con el slot SHALL ser un error. Un `<num>` inexistente SHALL ser un error.

El episodio nuevo SHALL derivar su `slug` del título (`slugify` a kebab-case),
sufijándolo `-2`, `-3`, … si colisiona con un `slug` ya presente en el tracker, salvo que
se aporte `--slug`, en cuyo caso SHALL usarse tal cual. Si el título no produce un `slug`
utilizable y no se aporta `--slug`, el episodio SHALL quedar sin `slug`.

#### Scenario: desplaza y renombra
- **GIVEN** un tracker con `#843` en `2026-10-26` y su material `0843/`
- **WHEN** se invoca `insert --before 843 --title "Urgente"`
- **THEN** el nuevo SHALL quedar `#843` en `2026-10-26`
- **AND** el antiguo SHALL pasar a `#844` con la fecha del slot siguiente
- **AND** su material SHALL renombrarse de `0843` a `0844`

#### Scenario: deriva el slug del título
- **GIVEN** un tracker sin ningún `slug` que colisione
- **WHEN** se invoca `insert --before 843 --title "Cómo montar un NAS"`
- **THEN** el episodio nuevo SHALL quedar con `slug: como-montar-un-nas`

#### Scenario: el slug derivado se sufija si colisiona
- **GIVEN** un tracker que ya tiene un episodio con `slug: urgente`
- **WHEN** se invoca `insert --before 843 --title "Urgente"`
- **THEN** el episodio nuevo SHALL quedar con `slug: urgente-2`

#### Scenario: `--slug` explícito manda
- **GIVEN** un tracker cualquiera
- **WHEN** se invoca `insert --before 843 --title "Urgente" --slug mi-slug`
- **THEN** el episodio nuevo SHALL quedar con `slug: mi-slug`

#### Scenario: título sin slug utilizable y sin `--slug`
- **GIVEN** un título que `slugify` reduce a vacío
- **WHEN** se invoca `insert --before 843 --title "¡¡¡"`
- **THEN** el episodio nuevo SHALL quedar sin `slug`

### Requirement: `move` cambia la fecha de emisión a un día publicable

`move <num|slug> --date <YYYY-MM-DD>` SHALL cambiar la fecha del episodio. La fecha SHALL ser un día de publicación (lunes o jueves); otra fecha SHALL ser un error. Una fecha ya ocupada por otro episodio SHALL ser un error de colisión, sin escribir.

#### Scenario: fecha no publicable falla
- **GIVEN** que los días de publicación son lunes y jueves
- **WHEN** se invoca `move <slug> --date 2026-10-28` (miércoles)
- **THEN** SHALL salir con un código distinto de cero
- **AND** el tracker SHALL permanecer inalterado

### Requirement: `--dry-run` no escribe nada

`publish`, `insert` y `move` SHALL aceptar `--dry-run`. Con él, SHALL imprimir la previsualización de los cambios y NO SHALL escribir ni el tracker ni renombrar material alguno.

#### Scenario: dry-run no toca el tracker ni el disco
- **GIVEN** un tracker y un árbol de material con estado conocido
- **WHEN** se invoca cualquiera de los comandos con `--dry-run`
- **THEN** el contenido del tracker SHALL permanecer inalterado
- **AND** el árbol de material SHALL permanecer inalterado
- **AND** NO SHALL crearse ningún `.bak`

### Requirement: Los errores van a `stderr` y el código de salida es contrato

En caso de error, el comando SHALL escribir el mensaje en `stderr` y SHALL salir con un código distinto de cero, reproduciendo los códigos de la implementación de referencia. En caso de éxito, SHALL salir con `0`.

#### Scenario: tracker ausente
- **GIVEN** una ruta de tracker que no existe
- **WHEN** se invoca cualquier comando de escritura
- **THEN** SHALL escribir el error en `stderr` y SHALL salir con un código distinto de cero
