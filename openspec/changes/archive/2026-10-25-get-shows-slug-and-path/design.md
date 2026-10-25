# Design

## Context

`get` nació como espejo del `cmd_get` del oráculo Python, con paridad byte a byte: un
bloque de cinco líneas con el número, el título, el estado, la fecha y la temporada. El
`slug` y la ruta del material nunca formaron parte de él porque el oráculo los servía por
otro lado (`where`). Con el oráculo ya retirado (change `retire-python`), el contrato vive
en `openspec/specs/` y este cambio lo extiende. Ver `proposal.md` para la motivación.

Los precedentes de formato ya existen en el repo: `status` imprime `slug:` y `ruta:` en su
bloque «Siguiente a publicar» (`next_to_publish_block`), y su documento `--json` calcula
`material` como `ctx.show.scripts_root().join(slug)`. `where` confirma que esa ruta es la
carpeta de material, no el YAML del tracker.

## Goals / Non-Goals

**Goals**

- Que `get` sea autosuficiente: identidad (`slug`) y localización (ruta del material) en la
  misma salida que el resto del detalle.
- Reutilizar el cálculo de ruta ya probado (`scripts_root().join(slug)`) sin introducir una
  noción nueva de «ruta de material».
- Cero cambios en las cinco líneas actuales, para no romper los tests de paridad.

**Non-Goals**

- No se comprueba la existencia del directorio: `get` describe el episodio; si el material
  existe o no es cosa de `where`.
- No se toca el resto de comandos de lectura ni la escritura.
- No cambia el JSON **observable** de `list`, `list-range`, `add`, `update`, `publish`,
  `insert` y `move` (verificado byte a byte); `material` sigue siendo exclusivo de `get`.
- No se sube `schema_version`.

## Decisions

### D1 — Etiquetas capitalizadas (`Slug:`, `Ruta:`), coherentes con el bloque

El bloque de `get` usa etiquetas capitalizadas y alineadas (`Título:`, `Estado:`, `Fecha:`,
`Temporada:`). Añadir `slug:`/`ruta:` en minúscula —como hace `status`— rompería esa
coherencia visual dentro del mismo bloque.

**Alternativa descartada:** *`slug:`/`ruta:` en minúscula, copiando `status`.* Se descarta
porque `status` es un bloque de acción indentado y `get` es una ficha de campos: la
convención la fija el bloque que se está ampliando, no el otro comando.

### D2 — Alineación a 11 columnas con `format!("  {:<10} {}", …)`

`Título:` ocupa 7 caracteres y su valor arranca en la columna 11 (dos de sangría + etiqueta
de 10 + un espacio). `Slug:` (5) y `Ruta:` (5) deben empezar en la misma columna, así que se
emiten con `format!("  {:<10} {}", "Slug:", …)`, que replica exactamente ese ancho sin tocar
las líneas existentes.

**Alternativa descartada:** *Alinear con espacios literales contados a mano.* Se descarta
por frágil ante futuras etiquetas de distinta longitud.

### D3 — Las líneas nuevas van al final, antes del separador de cierre

Las cinco líneas existentes permanecen byte a byte en su sitio. Las dos nuevas se añaden
después de `Temporada:` y antes del separador `═`, de modo que cualquier test de paridad
que compruebe línea a línea el bloque actual siga verde y las líneas nuevas sean puramente
aditivas.

### D4 — Sin slug: `—` en texto y `null` en JSON

Un episodio sin `slug` (existe uno real en el fixture: el `#202` de `MINI_YAML_V2`) no
tiene ruta de material que mostrar. Se sigue el precedente de `status`: `—` en texto —nunca
se inventa ruta— y `null` en JSON, exactamente como `slug` ya vale `null` cuando falta.
Nunca se emite una ruta vacía ni una cadena con `—` dentro del JSON.

### D5 — `material` se añade en `get_document`, **no** en el helper compartido

`episode_fields(episode) -> serde_json::Map<String, Value>` es el helper compartido por
`list`, `list-range`, `add`, `update`, `publish`, `insert` y `move`; meter `material` ahí
cambiaría la carga útil de siete comandos y la de sus tests. `get_document` es el único
consumidor que debe cambiar: parte del mapa de `episode_fields(...)` y le **inyecta**
`material`. El JSON observable de los otros siete comandos no cambia (verificado byte a
byte).

**Alternativa descartada:** *Añadir `material` al helper compartido.* Se descarta por ser un
cambio de alcance desproporcionado y potencialmente rompedor para consumidores que no lo
pidieron.

### D6 — `schema_version` se queda en 1

La adición es **aditiva**: una clave nueva dentro de `episode`. Un consumidor que ignore
`material` sigue funcionando, y no existe todavía ninguno que lo espere. Subir la versión
por una clave aditiva obligaría a los consumidores a ramificar sin motivo. La regla del
repo (`json-output`) ya fija que `schema_version` discrimina la *forma*, y la forma aquí no
cambia.

### D7 — Etiqueta `Ruta:` para el valor absoluto

`Ruta:` describe la carpeta de material del episodio (absoluta, calculada desde
`scripts.root`), no una ruta relativa ni un fichero. Se elige el nombre corto porque el
bloque ya está contextualizado por el episodio y su número.

### D8 — El helper compartido devuelve el **mapa**, no un `Value` (deuda registrada)

El objeto `episode` lo comparten `list`, `list-range`, `add`, `update`, `publish`, `insert`
y `move`; `get` le añade `material`. Para evitar un `if let Value::Object(fields)` que
descartaría la clave en silencio si el helper dejara de devolver un objeto, el helper pasa
a ser `fn episode_fields(episode) -> serde_json::Map<String, Value>` y cada documento lo
envuelve con `Value::Object(...)`. La extracción de `get` inserta `material` en el mapa
directamente: sin `if let`, sin descarte posible.

Como `serde_json` se compila sin `preserve_order`, el mapa es un `BTreeMap` y el JSON
emitido es **idéntico** al de antes (mismas claves, mismo orden alfabético). Blindado por
el test `el_material_es_exclusivo_de_get`.

**Deuda técnica conocida (no se paga en este change).** La extracción del `slug` («tiene
slug, sin espacios alrededor, o nada») está duplicada en cinco sitios —`status.rs::
next_to_publish_block`, `where_cmd.rs`, `json.rs::status_document`, `get.rs` y
`get_document`— y además con una **asimetría**: `status_document` usa `s.as_str()` sin
`trim`, mientras `next_to_publish_block`, `where_cmd` y `get`/`get_document` sí `trim()`.
Aquí `get` se dejó deliberadamente consistente con `next_to_publish_block` (el precedente
que cita D4), no con `status_document`. El helper futuro propuesto es
`Episode::slug_str() -> Option<&str>` (slug recortado o `None`), que unificaría las cinco
extracciones y eliminaría la asimetría del `trim`. No se extrae ahora porque tocar
`status.rs` y `where_cmd.rs` arriesga la paridad y este change está acotado a `get`.

## Supuestos y deuda registrada

- **La ruta del material es absoluta *bajo el supuesto de que `tracker:` es absoluto en el
  perfil`.** `ResolvedShow::scripts_root()` es el padre del `tracker:` del perfil y, si el
  `tracker:` fuera relativo (o no hubiera perfil), cae a `PathBuf::from(".")`. En ese caso
  la `Ruta:` de `get` sería relativa, incumpliendo el «absoluta» del requisito. Es
  **preexistente y compartido** con `status` (bloque «Siguiente a publicar») y `where`, y
  todos los perfiles reales declaran un `tracker:` absoluto. No se normaliza aquí: hacerlo
  tocaría `profile.rs` y la resolución de show, fuera del alcance de `get`. La normalización
  del `tracker:` a absoluto en la resolución de show sería un **change de seguimiento**
  (`profile-absolute-paths`); hasta entonces, el supuesto documentado es que el perfil
  declara `tracker:` absoluto.
- **Duplicación de la extracción de `slug` en cinco sitios** y asimetría del `trim` (ver
  D8).

## Risks / Trade-offs

- *La ruta puede no existir y `get` no lo dice* → deliberado: `where` es el comando que
  informa de la existencia; `get` no debe duplicar esa semántica.
- *Consumidores que parseen la salida de `get` por líneas* → mitigado: las líneas nuevas van
  al final y las cinco existentes quedan idénticas; los consumidores programáticos tienen
  `--json`.
- *Divergencia de estilo entre el `slug:` de `status` y el `Slug:` de `get`* → asumida y
  documentada en D1: son dos bloques con convenciones distintas.

## Divergencia respecto del oráculo (registro)

| Aspecto | Oráculo Python (retirado) | `podtrack` (este change) |
|---|---|---|
| Líneas del bloque | 5 (número, título, estado, fecha, temporada) | 5 + `Slug:` + `Ruta:` |
| Ruta del material | `get` no la muestra; la da `where` | `get` la muestra, absoluta y sin comprobar |
| Existencia del directorio | — | deliberadamente no comprobada |
| JSON de `get` | sin `material` | `material` aditivo (ruta absoluta o `null`) |

Esta extensión no reproduce ninguna línea del oráculo retirado: es capacidad nueva nuestra.

> Aviso: la ruta se anuncia «absoluta» **suponiendo que `tracker:` es absoluto en el perfil**
> (ver «Supuestos y deuda registrada»); con un `tracker:` relativo —preexistente, compartido
> con `status`/`where`— saldría relativa.

## Migración / compatibilidad

Cambio de presentación de texto **aditivo** y de JSON **aditivo**, sin efectos sobre el
tracker ni el material. Ningún consumidor existente queda roto; quien parseara el bloque de
`get` por número de líneas debería migrar a `--json`.
