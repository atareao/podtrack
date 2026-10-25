# Tasks

## 0. Prerrequisito

- [x] 0.1 `src/commands/status.rs`: exponer la selección de la ventana en una función
      pura testeable —«último publicado + hasta 7 próximos»— sin tocar el resto del
      resumen.
- [x] 0.2 `cargo check` limpio.

## 1. RED (pruebas que fallan primero)

- [x] 1.1 `tests/parity_read.rs`: caso con dos publicados y varios próximos → la ventana
      abre con el publicado más reciente y sigue con los próximos en orden ascendente;
      el publicado antiguo no aparece.
- [x] 1.2 Caso sin publicados → la ventana muestra solo los próximos, todos con `*`.
- [x] 1.3 Caso sin próximos → la ventana muestra solo el último publicado, sin `*`.
- [x] 1.4 Caso con más de 7 próximos → la ventana corta en 1 publicado + 7 próximos.
- [x] 1.5 Caso sin episodios → `No hay episodios.`
- [x] 1.6 `cargo test`: las pruebas nuevas fallan (la conducta actual muestra los 8
      últimos por parrilla).

## 2. GREEN (implementación mínima)

- [x] 2.1 `src/commands/status.rs`: sustituir `ordered.iter().rev().take(8)` por la
      ventana último-publicado + próximos.
- [x] 2.2 Actualizar la cabecera del bloque a `Último publicado y próximos a publicar:`.
- [x] 2.3 Mantener la marca provisional y la leyenda solo sobre la cola de próximos.
- [x] 2.4 `cargo test` en verde.

## 3. REFACTOR

- [x] 3.1 `cargo fmt --all` y `cargo clippy --all-targets -- -D warnings` limpios.
- [x] 3.2 Sin `unwrap()`, `expect()` ni `panic!()` en producción.
- [x] 3.3 Ningún test toca `/data/notas` ni `/data/podcasts` (fixtures temporales).
- [x] 3.4 Pasada manual de solo lectura sobre el tracker real: la ventana muestra el
      último publicado y los 7 siguientes.

## 4. Archivo

- [ ] 4.1 `just spec-validate status-upcoming-window` en estricto.
- [ ] 4.2 `just check-all` en verde.
- [ ] 4.3 `just spec-archive status-upcoming-window`.
