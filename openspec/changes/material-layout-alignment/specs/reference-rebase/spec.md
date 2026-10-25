# Spec Delta

## Purpose

Alinear `rebase-references` con la **regla única** de resolución de la carpeta de material (change
`material-layout-alignment`) y conservar su contrato de **saltar** —no abortar— ante un episodio cuya
carpeta no sea resoluble.

## MODIFIED Requirements

### Requirement: `rebase-references` solo procesa episodios no publicados con slug

`rebase-references` SHALL saltar los episodios `published` y los que no tienen `slug`, y SHALL saltar
los episodios cuya **carpeta de material no sea resoluble** (plantilla numérica y episodio sin
número). La carpeta de cada episodio procesado SHALL resolverse con la **regla única** de resolución
de material (`<raíz de material>/<dir>`, con `dir` = `episodes.scripts.dir` renderizado y `{slug}`
por defecto), la misma que `where`/`get`/`status`. SHALL saltar las carpetas inexistentes en disco.
NO SHALL tocar el tracker.

#### Scenario: los publicados no se procesan
- **GIVEN** un episodio publicado con un hook en su material
- **WHEN** se ejecuta `rebase-references`
- **THEN** su material NO SHALL reescribirse

#### Scenario: perfil con plantilla numérica
- **GIVEN** un perfil con `episodes.scripts.dir: "episodio-{n:02d}"` y un episodio no publicado de
  número `7` con material en `<raíz de material>/episodio-07/`
- **WHEN** se ejecuta `rebase-references`
- **THEN** la carpeta visitada SHALL ser `<raíz de material>/episodio-07`

#### Scenario: un episodio con slug pero sin número no aborta la cirugía
- **GIVEN** un perfil con `episodes.scripts.dir: "episodio-{n:02d}"` y un episodio no publicado con
  `slug` y sin número
- **WHEN** se ejecuta `rebase-references`
- **THEN** el comando SHALL salir con `0`
- **AND** NO SHALL reescribirse nada del material de ese episodio
