# Tasks

## 0. Prerrequisito

- [x] 0.1 `src/commands/get.rs`: localizar el punto del bloque de texto donde se cierran
      las cinco líneas actuales (tras `Temporada:`) y el punto donde `get_document` arma el
      objeto `episode`. Nada más.
- [x] 0.2 `cargo check` limpio.

## 1. RED (pruebas que fallan primero)

- [x] 1.1 `tests/parity_read.rs`: `get 200` contiene `Slug: primer-episodio` y la ruta
      absoluta `<root>/primer-episodio` (con `scripts_root` = raíz del fixture temporal).
- [x] 1.2 `tests/parity_read.rs`: `get 202` (episodio sin `slug` en `MINI_YAML_V2`)
      contiene `Slug: —` y `Ruta: —`.
- [x] 1.3 `tests/parity_read.rs`: la ruta se muestra aunque el directorio de material NO
      exista (no se crea el directorio en el fixture y aun así aparece la ruta).
- [x] 1.4 `tests/parity_read.rs`: las cinco líneas existentes del bloque siguen byte a byte
      idénticas (los casos de paridad actuales siguen verdes).
- [x] 1.5 `tests/json_output.rs`: `get 200 --json` incluye `episode.material` con la ruta
      absoluta `<root>/primer-episodio` y `schema_version` sigue siendo `1`.
- [x] 1.6 `tests/json_output.rs`: `get 202 --json` (sin `slug`) deja `episode.material` a
      `null` y `episode.slug` a `null`.
- [x] 1.7 `cargo test`: las pruebas nuevas fallan (hoy `get` no imprime `Slug:`/`Ruta:` ni
      el documento expone `material`).

## 2. GREEN (implementación mínima)

- [x] 2.1 `src/commands/get.rs`: añadir `Slug:` con el `slug` del episodio o `—` si no lo
      tiene, usando `format!("  {:<10} {}", "Slug:", …)`.
- [x] 2.2 `src/commands/get.rs`: añadir `Ruta:` con
      `ctx.show.scripts_root().join(&slug).display()` o `—` si no hay `slug`, SIN comprobar
      la existencia del directorio.
- [x] 2.3 Colocar las dos líneas nuevas tras `Temporada:` y antes del separador `═`, sin
      alterar las cinco existentes.
- [x] 2.4 `src/json.rs`: `get_document` inyecta `material` (ruta absoluta con
      `ctx.show.scripts_root().join(&slug).display().to_string()`, o `null` sin `slug`) en
      el objeto `episode`. **No** tocar `episode_json` ni `schema_version`.
- [x] 2.5 `cargo test` en verde.

## 3. REFACTOR

- [x] 3.1 `cargo fmt --all` y `cargo clippy --all-targets -- -D warnings` limpios.
- [x] 3.2 Sin `unwrap()`, `expect()` ni `panic!()` en producción.
- [x] 3.3 Ningún test toca `/data/notas` ni `/data/podcasts` (fixtures temporales).
- [x] 3.4 Pasada manual de solo lectura: `get <slug>` y `get <número sin slug>` sobre el
      tracker real muestran `Slug:`/`Ruta:` correctos y `—` cuando corresponde; `get --json`
      expone `material`.

## 4. Archivo

- [x] 4.1 `just spec-validate get-shows-slug-and-path` en estricto.
- [x] 4.2 `just check-all` en verde.
- [x] 4.3 `just spec-archive get-shows-slug-and-path`.

## 5. Cierre de hallazgos del revisor (post-review)

- [x] 5.1 Hallazgo #1: `episode_json` pasa a `episode_fields` (devuelve el `serde_json::Map`)
      y cada documento lo envuelve con `Value::Object(...)`; `get_document` inserta `material`
      en el mapa sin `if let` (sin descarte silencioso) y sin `unwrap`/`expect`/`panic!`.
- [x] 5.2 Hueco B.1: `tests/parity_read.rs` resuelve `get primer-episodio` (slug) y fija
      `Slug:`/`Ruta:` alineadas.
- [x] 5.3 Hueco B.2: `tests/json_output.rs` blinda que `episode` en `list --json` y en
      `add --json` NO lleva `material` (alcance D5).
- [x] 5.4 Hueco B.3: `tests/parity_read.rs` fija el **orden** de las líneas nuevas (tras
      `Temporada:` y antes del separador de cierre).
- [x] 5.5 Hueco B.4: `tests/json_output.rs` fija que `get sin-numero --json`
      (`number: null`, con slug) expone `material` que termina en `sin-numero`.
- [x] 5.6 Hallazgo #2: decisión **D8** en `design.md` (duplicación de la extracción de slug
      + asimetría del `trim`); código de `get` consistente con `next_to_publish_block`.
- [x] 5.7 Hallazgo #3: sección «Supuestos y deuda registrada» en `design.md` (ruta absoluta
      *bajo el supuesto* de `tracker:` absoluto) + aviso bajo la tabla de divergencia.
- [x] 5.8 Mutation check (método y alcance real): con `src/json.rs` revertido a HEAD (stash),
      los tests de JSON de `get` se ponen en rojo (falta `material`); el negativo de D5 es un
      **guardián de regresión** y solo se pone en rojo al **inyectar** `material` dentro de
      `episode_fields`. Árbol restaurado exactamente (mismo `sha256`, stash vacía).
