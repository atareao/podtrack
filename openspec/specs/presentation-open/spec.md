# presentation-open Specification

## Purpose
Definir `presentation-open`: la resolución de la ruta de la **fuente** de la presentación Typst de
un episodio a partir del layout de material declarado por el perfil, y su apertura desacoplada con
**typshow**, con su contrato de salida (texto/JSON), `--dry-run` y errores tipados.

## Requirements

### Requirement: `open` resuelve la presentación de un episodio desde el layout del perfil

`podtrack open <num|slug>` SHALL resolver el episodio por número o por slug (misma resolución que
`get`/`where`) y SHALL derivar la ruta de la **fuente** de su presentación a partir del layout de
material del perfil:

- La carpeta SHALL ser `<raíz de material>/<scripts.dir>`, con `scripts.dir` **renderizado**; si el
  perfil no declara `dir`, el binario SHALL asumir `{slug}`.
- La **raíz de material** SHALL ser el directorio padre del tracker resuelto
  (`ResolvedShow::scripts_root()`), la misma convención que `where`/`get`; el binario NO SHALL leer
  la clave `episodes.scripts.root` del perfil (evita divergir de `where`/`get`).
- El nombre de fichero SHALL derivarse de `episodes.scripts.files.presentacion`, **renderizado**,
  con su extensión **reescrita a `.typ`**.
- El binario NO SHALL inventar la ruta cuando el perfil no declara `files.presentacion`; en ese
  caso SHALL fallar con un error explícito.

#### Scenario: episodio resuelto por número
- **GIVEN** un perfil con `scripts.dir: "{slug}"` y `files.presentacion: "presentacion-{slug}.pdf"`
- **AND** un episodio `843` de slug `la-memoria-de-valet`
- **WHEN** se invoca `open 843`
- **THEN** la ruta resuelta SHALL ser
  `<raíz de material>/la-memoria-de-valet/presentacion-la-memoria-de-valet.typ`

#### Scenario: episodio resuelto por slug
- **GIVEN** el mismo perfil y episodio
- **WHEN** se invoca `open la-memoria-de-valet`
- **THEN** SHALL resolver la misma ruta que `open 843`

#### Scenario: `dir` parametrizado por número
- **GIVEN** un perfil con `scripts.dir: "episodio-{n:02d}"` y `files.presentacion: "presentacion-{slug}.typ"`
- **AND** un episodio de número `7`
- **WHEN** se invoca `open`
- **THEN** la carpeta SHALL ser `<raíz de material>/episodio-07/`

#### Scenario: el perfil no declara presentación
- **GIVEN** un perfil cuyo `episodes.scripts.files` no tiene la clave `presentacion`
- **WHEN** se invoca `open <target>`
- **THEN** SHALL salir con un código distinto de cero
- **AND** SHALL indicar en `stderr` que el show no declara una presentación
- **AND** NO SHALL inventar una ruta

### Requirement: `open` lanza typshow sin bloquear

`open` SHALL buscar el ejecutable `typshow` en el `PATH` y, tras resolver la ruta, SHALL lanzarlo
con la ruta del `.typ` como único argumento **en segundo plano**: NO SHALL esperar a que se cierre la
ventana. El *stdio* del visor NO SHALL contaminar la salida del binario. Tras lanzarlo con éxito,
`open` SHALL salir con `0`.

#### Scenario: lanzamiento correcto
- **GIVEN** un episodio cuya presentación existe en disco
- **AND** `typshow` disponible en el `PATH`
- **WHEN** se invoca `open <target>`
- **THEN** SHALL lanzar `typshow` con la ruta resuelta y salir con `0` sin esperar al visor

#### Scenario: sin typshow en el PATH
- **GIVEN** un `PATH` sin `typshow`
- **WHEN** se invoca `open <target>`
- **THEN** SHALL salir con un código distinto de cero
- **AND** el mensaje SHALL nombrar el lanzador que falta

### Requirement: `--dry-run` resuelve sin lanzar

`open <target> --dry-run` SHALL resolver la ruta y escribirla en `stdout` **sin** ejecutar typshow, y
SHALL salir con `0`. Es la vía determinista de verificar la resolución sin abrir una GUI.

#### Scenario: dry-run no lanza nada
- **GIVEN** un episodio cuya presentación existe en disco
- **WHEN** se invoca `open <target> --dry-run`
- **THEN** SHALL escribir la ruta resuelta en `stdout`
- **AND** NO SHALL ejecutar typshow
- **AND** SHALL salir con `0`

### Requirement: errores tipados de `open` con su código

`open` SHALL reportar cada causa de fallo con su error tipado y su código de salida:

- `PresentationNotDeclared` — el perfil no declara `files.presentacion` — código **2**.
- `PresentationNotFound` — el `.typ` declarado no existe en disco — código **1**, nombrando el
  episodio y la ruta.
- `LauncherNotFound` — `typshow` no está en el `PATH` — código **1**.
- `EpisodeWithoutSlug` — el layout necesita `{slug}` y el episodio no tiene — código **1**.
- `EpisodeWithoutNumber` — el layout necesita `{n}` y el episodio no tiene número — código **1**.
- `MaterialDirNotRelative` — el `episodes.scripts.dir` renderizado es **absoluto** o contiene un
  componente `..` — código **2**. `open` lo **hereda** de la regla única de resolución de material
  (`ResolvedShow::material_dir`), la misma que usan `where`/`get`/`status`.

#### Scenario: el `.typ` declarado no existe
- **GIVEN** un perfil que declara una presentación
- **AND** un episodio cuyo `.typ` no está en disco
- **WHEN** se invoca `open <target>`
- **THEN** SHALL salir con el código `1`
- **AND** el mensaje SHALL nombrar el episodio y la ruta esperada
- **AND** NO SHALL lanzar typshow

#### Scenario: no se lanza nada si la resolución falla
- **GIVEN** cualquier fallo de resolución (presentación no declarada, `.typ` inexistente, falta de
  slug/número, `dir` no relativo)
- **WHEN** se invoca `open <target>`
- **THEN** typshow NO SHALL ejecutarse

#### Scenario: `dir` no relativo
- **GIVEN** un perfil con `episodes.scripts.dir: "/etc"` que declara la presentación
- **WHEN** se invoca `open <target> --dry-run`
- **THEN** SHALL salir con `2`
- **AND** NO SHALL lanzar typshow

### Requirement: `--json` aditivo de `open`

Con `--json`, `open` SHALL emitir un único documento con el `show`, el episodio (`number`, `slug`),
la ruta resuelta (`presentation`), el lanzador (`launcher`) y si se lanzó (`launched`). Sin `--json`,
la salida de texto NO SHALL cambiar. En `--dry-run`, `launched` SHALL ser `false`.

#### Scenario: documento JSON de open
- **GIVEN** un episodio resuelto
- **WHEN** se invoca `open <target> --json`
- **THEN** SHALL emitir un JSON con `schema_version`, `show`, `episode`, `presentation`, `launcher`
  y `launched`
- **AND** en `--json --dry-run`, `launched` SHALL ser `false`
