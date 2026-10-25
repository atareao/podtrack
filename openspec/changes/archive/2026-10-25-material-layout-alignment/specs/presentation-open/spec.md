# Spec Delta

## Purpose

Declarar que `open` **hereda** el endurecimiento de la regla única de resolución de material (change
`material-layout-alignment`): un `episodes.scripts.dir` renderizado que no sea relativo es un error
tipado de configuración.

## MODIFIED Requirements

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
