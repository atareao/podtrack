# Spec Delta

## ADDED Requirements

### Requirement: La ruta del tracker del perfil se resuelve a absoluta

La ruta del tracker que el binario resuelve a partir del perfil SHALL ser **siempre absoluta y
normalizada léxicamente**, con independencia de cómo la declare el perfil:

- Si `tracker:` es una ruta **absoluta**, SHALL usarse tal cual, normalizada léxicamente (sin
  componentes `.` ni `..`).
- Si `tracker:` es una ruta **relativa**, SHALL resolverse contra el **directorio del fichero de
  perfil** que la declara. Si ese directorio fuera a su vez relativo, SHALL resolverse contra el
  **directorio de trabajo actual**.
- La resolución SHALL ser **puramente léxica**: NO SHALL consultar el sistema de ficheros, NO
  SHALL exigir que la ruta exista, NO SHALL resolver enlaces simbólicos y NO SHALL crear nada.

En consecuencia, `<scripts.root>` (el directorio padre del tracker) SHALL ser absoluto, y las
rutas de material que imprimen `status`, `get` y `where` SHALL ser absolutas **sin condiciones**.

#### Scenario: un tracker absoluto y limpio se conserva idéntico
- **GIVEN** un perfil que declara `tracker: /data/notas/Podcasts/guiones/podcast-tracker.yaml`
- **WHEN** se resuelve el perfil
- **THEN** la ruta del tracker SHALL ser exactamente `/data/notas/Podcasts/guiones/podcast-tracker.yaml`
- **AND** `<scripts.root>` SHALL ser `/data/notas/Podcasts/guiones`

#### Scenario: un tracker relativo se resuelve contra el directorio del perfil
- **GIVEN** un perfil en `<base>/shows/demo.yaml` que declara `tracker: ../guiones/podcast-tracker.yaml`
- **WHEN** se resuelve el perfil
- **THEN** la ruta del tracker SHALL ser `<base>/guiones/podcast-tracker.yaml` (absoluta)
- **AND** `<scripts.root>` SHALL ser `<base>/guiones`

#### Scenario: los componentes `.` y `..` se normalizan léxicamente
- **GIVEN** un perfil que declara `tracker: /a/./b/../c.yaml`
- **WHEN** se resuelve el perfil
- **THEN** la ruta del tracker SHALL ser `/a/c.yaml`

#### Scenario: la resolución no exige que la ruta exista
- **GIVEN** un perfil cuyo `tracker:` relativo apunta a un fichero que NO existe
- **WHEN** se resuelve el perfil con una ruta relativa como `tracker:`
- **THEN** la ruta del tracker resuelta SHALL ser absoluta igualmente
- **AND** NO SHALL crearse ningún fichero ni directorio
- **AND** el error posterior de tracker ausente SHALL nombrar la ruta **absoluta**

#### Scenario: `scripts_root` nunca cae al directorio actual
- **GIVEN** un perfil resuelto con un `tracker:` que declara un nombre sin directorio
  (`tracker: podcast-tracker.yaml`)
- **WHEN** se calcula `<scripts.root>`
- **THEN** `<scripts.root>` SHALL ser el directorio que contiene la ruta absoluta resuelta
- **AND** NO SHALL ser `"."`
