# Spec Delta

## Purpose

Fijar la **regla única** de resolución de la carpeta de material de un episodio, para que todos los
subcomandos —los que informan de la ruta y los que operan sobre el material— usen la misma.

## ADDED Requirements

### Requirement: La carpeta de material de un episodio se resuelve con una única regla

El binario SHALL resolver la carpeta de material de un episodio con **una sola** regla, compartida
por todos los subcomandos: `<raíz de material>/<dir>`, donde `<dir>` es `episodes.scripts.dir` del
perfil **renderizado** con los valores del episodio (`{slug}`, `{n}`, `{n:0Nd}`). Si el
perfil no declara `episodes.scripts.dir` o lo declara vacío, el binario SHALL asumir `{slug}`. La
raíz de material SHALL ser el directorio padre del tracker resuelto
(`ResolvedShow::scripts_root()`).

Ningún subcomando SHALL derivar la carpeta de material cableando el `slug` por su cuenta.

#### Scenario: perfil que declara `dir: "{slug}"` (comportamiento actual)
- **GIVEN** un perfil con `episodes.scripts.dir: "{slug}"` y un episodio con
  `slug: la-memoria-de-valet`
- **WHEN** se resuelve su carpeta de material
- **THEN** SHALL ser `<raíz de material>/la-memoria-de-valet`

#### Scenario: perfil sin `dir`
- **GIVEN** un perfil que no declara `episodes.scripts.dir`
- **WHEN** se resuelve la carpeta de material de un episodio con `slug: la-memoria-de-valet`
- **THEN** SHALL ser `<raíz de material>/la-memoria-de-valet`

#### Scenario: perfil con plantilla numérica
- **GIVEN** un perfil con `episodes.scripts.dir: "episodio-{n:02d}"` y un episodio de número `7`
- **WHEN** se resuelve su carpeta de material
- **THEN** SHALL ser `<raíz de material>/episodio-07`

#### Scenario: la lectura y la cirugía coinciden
- **GIVEN** un perfil con `episodes.scripts.dir: "episodio-{n:02d}"`
- **WHEN** se informa de la ruta del material de un episodio y se planifica la migración de su
  material
- **THEN** ambas rutas SHALL ser la misma

### Requirement: El `dir` renderizado no puede escapar de la raíz de material

Si el `dir` renderizado de un episodio es una ruta **absoluta** o contiene un componente `..`, el
binario SHALL fallar con un error tipado propio y SHALL salir con el código de error de configuración
(`2`), sin resolver ni tocar ninguna ruta fuera de la raíz de material.

#### Scenario: `dir` absoluto
- **GIVEN** un perfil con `episodes.scripts.dir: "/etc"`
- **WHEN** se invoca un subcomando que resuelve la carpeta de material de un episodio
- **THEN** SHALL salir con `2`
- **AND** el mensaje SHALL nombrar el perfil y el `dir`

#### Scenario: `dir` con `..`
- **GIVEN** un perfil con `episodes.scripts.dir: "../../fuera"`
- **WHEN** se invoca un subcomando que resuelve la carpeta de material de un episodio
- **THEN** SHALL salir con `2`
- **AND** NO SHALL resolverse ninguna ruta fuera de la raíz de material
