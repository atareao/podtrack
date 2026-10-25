# Design

## Context

Ver `proposal.md` — *Why*. Estado actual relevante:

- **Extracción del slug duplicada y asimétrica.** Cinco-ish sitios extraen el slug. Con
  `trim`: `status.rs::next_to_publish_block`, `where_cmd.rs::run`, `get.rs::run` y el
  `material` de `json.rs::get_document`. **Sin** `trim`: `json.rs::status_document` (el
  `next_to_publish` y el `last_published`), `json.rs::where_document` y `json.rs::
  episode_fields`. El helper futuro `Episode::slug_str()` está **prescrito** en el change
  archivado `get-shows-slug-and-path` (design.md, D8).
- **Invariante de carga.** `src/tracker.rs:95` ya normaliza el slug al cargar
  (`Some(s) if s.trim().is_empty() => None`, `Some(s) => Some(Slug::parse(s.trim())?)`) y
  `Slug::parse` exige kebab-case. Por tanto, un episodio **cargado** nunca tiene un slug con
  espacios ni vacío.
- **`tracker:` sin recortar en la resolución.** `src/profile.rs::ShowResolver::load` compara
  `raw.trim()` en la guarda (línea ~193) pero resuelve con `Path::new(&raw)` sin recortar
  (línea ~200).

## Goals / Non-Goals

**Goals**

- Unificar la lectura del slug en el modelo con un accesor canónico (`Episode::slug_str()`:
  recortado y no vacío, o `None`) y hacer que **todos** los consumidores lo usen.
- Resolver el `tracker:` del perfil a partir del valor **recortado**, cerrando la asimetría
  entre la guarda y la construcción de la ruta.
- Mantener la paridad observable: ninguna salida del CLI cambia salvo el caso del `tracker:`
  entrecomillado con espacios.

**Non-Goals**

- **No** se normaliza el slug al construirlo fuera de la carga (`add`, `insert`,
  `backfill-slugs`, `renumber`): esos caminos ya pasan por `Slug::parse`, que rechaza espacios.
  El accesor es defensivo, no un punto de normalización de entrada.
- **No** se toca el writer, el parseo del tracker, los códigos de error ni el esquema del JSON.
- **No** se documenta la semántica de la ruta relativa en `shows/README.md` (fuera del repo).

## Decisions

### D1 — El accesor canónico vive en el modelo y devuelve `&str` recortado o `None`

`Episode::slug_str(&self) -> Option<&str>` devuelve el slug **recortado y no vacío**, o
`None`. Es la firma prescrita en D8 del change archivado. Se implementa como
`self.slug.as_ref().map(Slug::as_str).map(str::trim).filter(|s| !s.is_empty())`, sin asignar
memoria (vista prestada). Los consumidores que hoy usan `unwrap_or_default()` (texto) siguen
haciéndolo sobre `&str` (`""` cuando no hay slug); los documentos JSON usan el `&str` directo,
serializable por `serde_json`.

**Alternativa descartada:** devolver `String` (asigna en todos los sitios y rompe la vista
prestada sin ganancia). **Alternativa descartada:** normalizar el slug en el modelo al cargar y
eliminar el `trim` de los consumidores —tentador, pero excede el alcance y no unifica la
lectura.

### D2 — La mitad «slug» es un refactor **no observable**; se registra como tal

Con el invariante de carga, la salida observable del CLI **no cambia** para ningún dato
cargado: el slug ya llega recortado y kebab-case. El valor de este arreglo es de **contrato del
modelo** (DRY + defensa), no de comportamiento de cara al usuario. Se documenta
explícitamente para no disfrazar un refactor de bug fix. La duplicación de la extracción
desaparece: un único sitio define «qué es el slug de un episodio».

### D3 — La mitad «`tracker:`» **sí** es observable y se cierra en `load`

`ShowResolver::load` resuelve con `raw.trim()` en lugar de `&raw`. Un `tracker: " /a/b "`
entrecomillado (YAML preserva los espacios) pasa la guarda de ausencia —`raw.trim()` no está
vacío— y hoy produce `Path::new(" /a/b ")`, una ruta con espacios que no existe. Con el
recorte, resuelve a `/a/b`. La guarda y la construcción quedan así **coherentes**: ambas miran
el mismo valor recortado.

### D4 — Qué capability recibe cada mitad

- **`show-profile`** (ADDED): «El valor de `tracker:` se interpreta sin espacios envolventes».
  Es **comportamiento observable** y testeable por CLI (`tracker: " /ruta "`).
- **`tracker-store`** (ADDED): «La lectura del slug en el modelo es canónica». Es un contrato
  **de modelo**; sus escenarios son de caja blanca (`Episode` en memoria con slug con espacios,
  solo construible dentro de `src/model.rs`). No cambia la salida observable.
- **`tracker-read` y `json-output` NO reciben delta**: ningún `Given/When/Then` de esos specs
  cambia su resultado verificable (el slug cargado ya era canónico). Añadir requisitos ahí
  sería inventar contrato no observable, y el esquema lo desaconseja.

## Risks / Trade-offs

- *La mitad «slug» no aporta valor observable* → asumido y registrado (D2). Su valor es la
  unificación y la defensa futura; se prueba con tests unitarios de modelo y de paridad
  texto/JSON.
- *Un `tracker:` con espacios envolventes era un caso patológico* → precisamente por eso hoy
  «funciona por casualidad del dato» (ningún perfil real lo declara). El test lo fija con una
  fixture entrecomillada, sin depender del dato.
- *Coste de la vista prestada en JSON* → nulo: `Option<&str>` es serializable y `json!` lo
  acepta sin asignación adicional.

## Open Questions

Ninguna. El alcance está cerrado por el usuario a las dos asimetrías del `trim`.
