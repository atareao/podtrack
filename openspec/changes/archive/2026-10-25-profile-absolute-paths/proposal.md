# Proposal

## Why

El requisito de `get` dice que la ruta del material es **absoluta** «cuando el `tracker:` del
perfil lo sea». Esa coletilla no es un matiz: es un agujero. `ResolvedShow::scripts_root()` es el
**directorio padre del `tracker:` tal cual lo declara el perfil**, y `profile.rs` no normaliza
nada: si un perfil declara `tracker: guiones/podcast-tracker.yaml` (relativo) —o si
`PODTRACK_SHOWS_DIR` es relativo y el perfil se localiza por la cascada—, la resolución produce
un `tracker` relativo, `scripts_root()` cae al `PathBuf::from(".")` de emergencia, y la `Ruta:`
que imprimen `status`, `get` y `where` sale **relativa o contaminada con `.`**. Lo mismo vale
para un `tracker:` absoluto pero sucio (`/a/./b/../c.yaml`).

Hoy no se nota porque el único perfil real con tracker (`atareao-con-linux`) lo declara
**absoluto y limpio**; el otro (`la-era-de-las-distros`) declara `tracker: none`, que ni
siquiera llega a la rama que normaliza. Pero
«funciona por casualidad del dato» no es un contrato: es deuda registrada en el change
`get-shows-slug-and-path` (design.md, «Supuestos y deuda registrada»), que dejó el `SHALL` de
`get` conscientemente cualificado para no prometer algo que el código no cumplía. Este change
paga esa deuda: hace que la ruta del tracker sea **siempre absoluta y léxicamente limpia**, de
modo que el «absoluta» deje de depender de cómo esté escrito el perfil.

## What Changes

- `ResolvedShow.tracker` SHALL ser **siempre una ruta absoluta y normalizada léxicamente**,
  venga del perfil como venga.
- Si `tracker:` es **absoluto**, SHALL usarse tal cual, normalizado léxicamente (sin componentes
  `.` ni `..`).
- Si `tracker:` es **relativo**, SHALL resolverse contra el **directorio del fichero de perfil**
  (el directorio que lo declara); si ese directorio fuera a su vez relativo, SHALL resolverse
  contra el directorio de trabajo actual (`std::env::current_dir()`). El resultado SHALL ser
  absoluto.
- La normalización SHALL ser **puramente léxica**: NO SHALL consultar el sistema de ficheros. No
  SHALL exigir que la ruta exista, ni resolver enlaces simbólicos, ni crear nada.
- Consecuencia directa: `scripts_root()` SHALL ser absoluto y NO SHALL caer al `"."` de
  emergencia para un `tracker:` resuelto; las rutas de material de `status`, `get` y `where`
  SHALL ser absolutas **sin condiciones**.
- El `SHALL` de `get` en `tracker-read` SHALL perder la cualificación «cuando el `tracker:` del
  perfil lo sea»: la ruta es absoluta, punto.
- `srt-stage` NO SHALL cambiar: resuelve el perfil sin exigir `tracker:` y no usa rutas de
  material.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `show-profile`: se **añade** un requisito que fija que la ruta del tracker resuelta por el
  perfil es absoluta y léxicamente normalizada, con la regla de base para los perfiles que
  declaran una ruta relativa.
- `tracker-read`: el requisito «`get` muestra el detalle de un episodio» **pierde la
  cualificación** de la ruta absoluta y pasa a exigirla sin condiciones; el requisito «`where`
  imprime la ruta del material sin crear nada» fija que esa ruta es absoluta.

## Impact

- `src/profile.rs`: `ResolvedShow::scripts_root()` (se retira el fallback `"."`) y la
  construcción de `ResolvedShow.tracker` en `ShowResolver::load` (normalización a absoluto). Un
  helper léxico `absolutize`-style, sin E/S, con tests unitarios propios.
- `openspec/specs/show-profile/spec.md` y `openspec/specs/tracker-read/spec.md`: deltas.
- `tests/parity_profile.rs`: escenarios de resolución (absoluto idéntico, relativo contra el
  directorio del perfil, `..` normalizado, sin exigir existencia).
- `tests/parity_read.rs`: la ruta que imprimen `get`/`where`/`status` es absoluta aunque el
  perfil declare un `tracker:` relativo.
- `docs/DEFINITION.md`: nota en la resolución del perfil (la ruta del tracker se normaliza a
  absoluta).
- **Sin impacto** en el parseo del tracker, en el writer, en la escritura ni en el JSON (salvo
  que la ruta del tracker aparezca en un mensaje de error, que pasaría a ser absoluta).
- **Fuera del repo (documentación, no código):** el esquema del perfil
  (`~/.agents/skills/podcast-srt/shows/README.md`) describe `tracker` como `path | none`; se
  anotará allí la semántica de la ruta relativa. No entra en este PR.
- **Sin divergencia** respecto del oráculo Python: el oráculo ya no existe (`retire-python`) y
  ningún perfil real ejercita el camino relativo (el único con tracker lo declara absoluto y
  limpio), de modo que la paridad observable se mantiene: la normalización de un absoluto
  limpio es la identidad.

## Criterio de cierre

Para un perfil con `tracker:` relativo, `podtrack where <episodio>`, `podtrack get <episodio>`
y el bloque «Siguiente a publicar» de `podtrack status` imprimen una ruta **absoluta**; y
`scripts_root()` no devuelve nunca `"."` para un `tracker:` resuelto.
