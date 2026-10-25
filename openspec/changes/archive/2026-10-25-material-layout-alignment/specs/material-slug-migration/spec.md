# Spec Delta

## Purpose

Alinear el destino de la migración de material con la regla única de resolución: la carpeta migrada
debe quedar exactamente donde luego la busca `where`/`get`/`open`.

## MODIFIED Requirements

### Requirement: `migrate-material` reescribe tokens de número a slug

Para cada episodio migrado con `slug`, el destino de su carpeta SHALL ser el que fija la **regla
única** de resolución de material, `<raíz de material>/<dir>` (con `dir` = `episodes.scripts.dir`
renderizado y `{slug}` por defecto), y los nombres de los ficheros y subcarpetas internos SHALL
sustituir los tokens de número por el `slug`, reconociendo las formas `episodio-NNN`, `eNNN`, `0NNN`
y `NNN` (sin ceros), acotadas por límites alfanuméricos. Las variantes que colapsan al mismo destino
(`0NNN`, `NNN`, `eNNN`) SHALL converger en el slug.

Para los perfiles que declaran `dir: "{slug}"` o no declaran `dir` —todos los actuales—, la regla
coincide con `<raíz de material>/<slug>`, así que el comportamiento no cambia.

#### Scenario: la carpeta y los tokens internos pasan a slug
- **GIVEN** una carpeta `0843/` con `e843-articulo.md` y `0843-notas.md` para un episodio con
  `slug: mi-episodio`
- **WHEN** se ejecuta `migrate-material`
- **THEN** la carpeta SHALL pasar a `mi-episodio/`
- **AND** los ficheros internos SHALL pasar a `mi-episodio-articulo.md` y
  `mi-episodio-notas.md`

#### Scenario: `episodio-NNN` conserva la palabra
- **GIVEN** un fichero `episodio-843-guion.md` para un episodio con `slug: mi-episodio`
- **WHEN** se ejecuta `migrate-material`
- **THEN** el fichero SHALL pasar a `episodio-mi-episodio-guion.md`

#### Scenario: perfil con plantilla numérica
- **GIVEN** un perfil con `episodes.scripts.dir: "episodio-{n:02d}"` y una carpeta `007/` para un
  episodio de número `7` con `slug: mi-episodio`
- **WHEN** se ejecuta `migrate-material`
- **THEN** la carpeta SHALL pasar a `episodio-07/`
