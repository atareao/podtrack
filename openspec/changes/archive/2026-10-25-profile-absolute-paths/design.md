# Design

## D1 — Qué se normaliza: `ResolvedShow.tracker`, no `scripts_root()`

El punto único por el que entra la relatividad es `ResolvedShow.tracker`: `scripts_root()` es su
`parent()` y todos los comandos que imprimen material pasan por ahí. Normalizar en el
constructor (`ShowResolver::load`) resuelve el problema para **todos** los consumidores a la vez
(`status`, `get`, `where`, y cualquier futuro), en vez de parchear cada llamada. Por eso el
cambio vive en `profile.rs` y no en `get.rs`.

## D2 — El fallback `"."` de `scripts_root()` se retira

Hoy `scripts_root()` es `self.tracker.parent().map_or_else(|| PathBuf::from("."), …)`. Ese `"."`
es una mentira silenciosa: no hay error, no hay aviso, sale una ruta relativa. Con `tracker`
garantizado absoluto (D1), el `parent()` de un absoluto **siempre** existe salvo el caso límite
de la raíz `/`, donde el `parent()` es `None`. Se mantiene un `map_or_else` pero el fallback
pasa a ser la propia ruta absoluta cuando no tiene padre (evita el `"."` y mantiene la
invariante «absoluto»). Se documenta en el rustdoc.

## D3 — Base para un `tracker:` relativo: el directorio del perfil

`path` es el fichero de perfil que se acaba de leer en `load(&slug, path)`, así que su
`parent()` es la base natural: **un perfil declara rutas relativas a sí mismo** (semántica
convencional de un fichero de configuración). Alternativas descartadas:

- *Relativo al CWD*: depende de dónde se invoque el binario; el mismo perfil daría rutas
  distintas. Se descarta como base primaria.
- *Rechazar el `tracker:` relativo* (error tipado): haría el `SHALL` verdad por la vía de
  prohibir, pero pierde una capacidad legítima y rompe perfiles hasta hoy tolerados.
- *`canonicalize()`*: exige que la ruta exista y resuelve enlaces simbólicos, lo que cambiaría
  la ruta observada (rompería la paridad byte a byte de los perfiles absolutos que hoy pasan por
  un symlink). Se descarta: la resolución es **léxica**.

Si el directorio del perfil fuera a su vez relativo (`PODTRACK_SHOWS_DIR=./shows`), la base se
absolutiza contra `std::env::current_dir()` para que el resultado siga siendo absoluto. Es el
único punto donde el CWD entra, y solo como último recurso.

## D4 — Normalización léxica: `join` no basta

`base.join(rel)` produce una ruta absoluta pero **sin colapsar** `..`: `shows/../guiones` →
`/base/shows/../guiones`. El `parent()` de esa ruta daría `/base/shows/..`, y `get` imprimiría
esa fealdad. Se implementa un colapso léxico de componentes (`CurDir` y `ParentDir`) sin tocar
el sistema de ficheros: ~15 líneas, con tests unitarios propios. `ParentDir` sobre la raíz se
ignora (`/..` → `/`), como hace el propio `Path::canonicalize` conceptualmente y como espera
cualquier usuario.

## D5 — Por qué no se toca la paridad con el oráculo

El oráculo Python **ya no existe** (`retire-python`): la paridad que se sigue cuidando es la de
la salida observable, y el camino relativo **no está ejercitado por ningún perfil real**: el
único perfil vivo que declara `tracker` (`atareao-con-linux`) lo declara absoluto y limpio, y
el otro (`la-era-de-las-distros`) declara `tracker: none`, que toma la rama `ProfileNoTracker`
**antes** de llegar a la normalización. Para un absoluto limpio la normalización es la
identidad, así que **la salida del perfil real no cambia un byte**.
Lo único observable que puede cambiar es el mensaje de `TrackerMissing` para un perfil con
`tracker:` relativo que no existe: pasaría a nombrar la ruta absoluta. Ningún test lo cubre hoy,
y es una mejora (la ruta absoluta es la que permite encontrarlo).

## D6 — `srt-stage` fuera de alcance

`srt-stage` resuelve el perfil por `find_profile` **sin exigir `tracker:`** y no calcula rutas de
material. Ese camino no pasa por `ResolvedShow` (o pasa con un perfil que ni siquiera tiene
tracker), así que no lo toca este change. Se verifica explícitamente que sus tests siguen verdes.

## Riesgos / Trade-offs

- *Un perfil con `tracker:` relativo y un CWD distinto del esperado* → la resolución es estable
  mientras el perfil esté donde está; es la semántica declarada (base = directorio del perfil).
- *Un consumidor que compare la ruta del tracker textualmente* → para los perfiles reales no
  cambia nada; para un relativo, la ruta cambia de forma (de relativa a absoluta), que es
  justamente el objetivo.
- *Coste en `profile.rs`* (+ ~40 líneas incluido el colapso léxico y sus tests) → acotado y
  sin E/S.

## Deuda que NO paga este change

- **Extracción del `slug` duplicada en cinco sitios** y asimetría del `trim` (D8 del change
  `get-shows-slug-and-path`). Sigue pendiente; helper propuesto: `Episode::slug_str()`.
- El esquema del perfil (`shows/README.md`, fuera del repo) no documenta aún la semántica de la
  ruta relativa; se anotará fuera de este PR.
- **Asimetría del `trim` en `ShowResolver::load`** (hallazgo #3 del auditor, **preexistente**):
  la comparación `none`/vacío usa `raw.trim()`, pero la ruta se construye con `raw` sin
  recortar. Un `tracker: " /a/b "` conservaría los espacios. No es regresión de este change
  (el resultado sigue siendo absoluto) y no está en el `SHALL`; se deja como está para no
  meter un cambio de comportamiento no pedido.
- **Mapeo de `io::Error` a `InvalidProfile`** (hallazgo #4 del auditor): el único `io::Error`
  posible viene de `std::env::current_dir()`, así que conflarlo con «perfil inválido» mezcla un
  fallo de entorno con un error de contenido. Poco probable y aceptable; un error tipado propio
  sería más preciso si algún día importa.
