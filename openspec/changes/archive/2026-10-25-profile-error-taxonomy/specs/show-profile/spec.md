# Spec Delta

## MODIFIED Requirements

### Requirement: El esquema del perfil no se reinterpreta

El binario SHALL leer del perfil únicamente los campos que el esquema define (`tracker`, `name`,
`episodes.*`) y NO SHALL inventar valores para los campos ausentes que el esquema marca como
obligatorios. Un perfil inválido SHALL ser un error, no un modo degradado.

Cuando la resolución del perfil falla, el binario SHALL distinguir **tres causas disjuntas** y
reportarlas con su error y su mensaje propios:

- **Perfil ilegible** — el fichero de perfil no se puede leer (permisos, es un directorio, un
  enlace roto). El mensaje SHALL decir que **no se pudo leer** el perfil, nombrando el perfil y la
  ruta, y NO SHALL presentarlo como «no válido».
- **Contenido mal formado** — el fichero se lee pero su YAML no parsea. El mensaje SHALL decir que
  el perfil **no es válido** y nombrar la causa.
- **Ruta del `tracker:` irresoluble** — el perfil se lee y parsea bien, pero su ruta de tracker no
  se puede resolver a absoluta (p. ej. no se puede determinar el directorio de trabajo). El
  mensaje SHALL nombrar el perfil y decir que **no se pudo resolver la ruta del tracker**.

Las tres causas SHALL salir con el **mismo** código de salida (el de perfil inválido, `2`).

#### Scenario: perfil sin la clave del tracker es un error
- **GIVEN** un perfil al que le falta la clave que declara la ruta del tracker
- **WHEN** se invoca cualquier comando
- **THEN** SHALL salir con un código distinto de cero
- **AND** SHALL indicar en `stderr` qué clave falta en qué perfil

#### Scenario: un perfil ilegible se distingue del mal formado
- **GIVEN** una ruta de perfil que no se puede leer (p. ej. un fichero sin permisos de lectura)
- **WHEN** el binario intenta cargar ese perfil
- **THEN** SHALL reportar el error de perfil **ilegible** (código `2`)
- **AND** el mensaje SHALL decir que **no se pudo leer** el perfil, nombrando el perfil y la ruta
- **AND** NO SHALL decir que el perfil «no es válido»

#### Scenario: un perfil mal formado nombra la causa
- **GIVEN** un perfil que se lee pero cuyo YAML no parsea
- **WHEN** se invoca un comando con `--show` apuntando a ese perfil
- **THEN** SHALL salir con el código de perfil inválido (`2`)
- **AND** el mensaje SHALL decir que el perfil **no es válido** y nombrar el perfil
