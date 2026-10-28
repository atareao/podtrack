# Changelog
## [0.6.1] - 2026-10-28

### Bug Fixes

- *(test)* Canoniza el orden del plan de renombrado en el comparador del golden
## [0.6.0] - 2026-10-28

### Documentation

- Documenta `neighborhood` en el SRS y el README
- *(spec)* Aprueba y archiva el change episode-neighborhood

### Features

- *(neighborhood)* Añade la vecindad numérica de un episodio

### Miscellaneous Tasks

- Release v0.6.0
## [0.5.2] - 2026-10-26

### Documentation

- *(git-flow)* Retira una referencia muerta a AGENTS.md
- Declara el estado real (el port terminó)
- *(spec)* Aprueba y archiva el change close-the-port
- *(plan)* Cierra el plan del change close-the-port

### Miscellaneous Tasks

- *(justfile)* Retira la convención FEATURE-NNN sin uso
- Release v0.5.2

### Testing

- *(parity)* Congela la paridad del oráculo retirado en un golden
- *(guard)* Impide que el arnés retirado vuelva a la suite
## [0.5.1] - 2026-10-25

### Bug Fixes

- *(material)* Resuelve la carpeta de material con una única regla
- *(material)* Cierra los hallazgos de la auditoría del change

### Documentation

- Sincronizar SRS y Git Flow con el estado real
- Cierra PLAN-006 (migración del material legacy a slug)
- *(spec)* Aprueba el change material-layout-alignment
- *(definicion)* Documenta la regla única de la carpeta de material
- *(spec)* Archiva material-layout-alignment
- *(plan)* Cierra el plan del change material-layout-alignment

### Miscellaneous Tasks

- Release v0.5.1
## [0.5.0] - 2026-10-25

### Bug Fixes

- *(profile)* Read episodes.scripts laxly and harden open

### Documentation

- Add open-presentation change proposal and PLAN-005
- Clarify material root and document the open subcommand
- Archive open-presentation and close PLAN-005

### Features

- *(open)* Open an episode's Typst presentation with typshow

### Miscellaneous Tasks

- Release v0.5.0

### Testing

- *(open)* Add failing RED tests for open-presentation
- *(open)* Cover launcher-missing, default dir, json and dir-by-number
## [0.4.2] - 2026-10-25

### Bug Fixes

- *(profile)* Distingue perfil ilegible, mal formado y tracker irresoluble

### Miscellaneous Tasks

- Release v0.4.2
## [0.4.1] - 2026-10-25

### Bug Fixes

- *(profile)* Recorta el tracker del perfil y unifica la lectura del slug

### Miscellaneous Tasks

- Release v0.4.1
## [0.4.0] - 2026-10-25

### Bug Fixes

- *(profile)* Resuelve el tracker del perfil a una ruta absoluta

### Features

- *(get)* Muestra el slug y la ruta del material del episodio

### Miscellaneous Tasks

- Release v0.4.0
## [0.3.0] - 2026-10-25

### Documentation

- *(openspec)* Archiva el change update-title

### Features

- *(update)* `--title` cambia el título del episodio (#13)

### Miscellaneous Tasks

- Release v0.3.0
## [0.2.1] - 2026-10-25

### Bug Fixes

- *(insert)* Deriva el slug del título cuando falta `--slug` (#11)

### Documentation

- *(openspec)* Archiva el change insert-derives-slug

### Miscellaneous Tasks

- Release v0.2.1
## [0.2.0] - 2026-10-25

### Documentation

- *(openspec)* Archiva el change status-upcoming-window

### Features

- *(status)* Ventana de último publicado + 7 próximos (#9)

### Miscellaneous Tasks

- Release v0.2.0
## [0.1.0] - 2026-10-25

### Documentation

- Add README and MIT license
- Add bootstrap-core OpenSpec change proposal
- Reframe the Python as temporary parity scaffolding and add the retire-python phase
- Add the srt-stage phase and widen the roadmap to five phases (Python cero)

### Features

- *(bootstrap-core)* Infra tipada + superficie de lectura con paridad byte a byte (#1)
- *(write-commands)* Superficie de escritura del tracker con paridad byte a byte (#2)
- *(migration-surgery)* Cirugía de parrilla y material con paridad byte a byte (#3)
- *(srt-stage)* Clasificador de etapas del SRT con paridad byte a byte (#4)
- *(retire-python)* Retira el tooling Python; el binario podtrack lo sustituye (#5)

### Miscellaneous Tasks

- Bootstrap project base
- Prepare release tooling
- Add CI and release pipelines
- Initialize OpenSpec (SDD)
- Release v0.1.0
- Metadatos de publicación y habilita el crate en crates.io (#7)
- Release v0.1.0
