# Tasks

## 0. Prerrequisito

- [x] 0.1 `src/commands/neighborhood.rs`: selector **puro** de la ventana
      (`select_neighborhood(&[Episode], &Episode, before, after) -> Neighborhood`, con `before`/`episode`/`after`), sin render y sin tocar ningún otro módulo. Test unitario en el propio fichero (`#[cfg(test)]`). La referencia se identifica **por identidad** (`std::ptr::eq`), no por número: con números repetidos marcar por número rompía el contrato (§ auditoría 2026-10-28).
- [x] 0.2 `cargo check` limpio.
- [x] 0.3 **Requiere la fase A de `plans/PLAN-009.md` cerrada** (con la suite en 340/0): esta feature
      no arranca con `close-the-port` a medias.

## 1. RED (pruebas que fallan primero)

- [x] 1.1 `tests/neighborhood.rs` (fichero nuevo): la ventana centra el episodio y marca la
      referencia con `►`.
- [x] 1.2 Resolución por slug del episodio de referencia (la cabecera declara su número).
- [x] 1.3 Con huecos, los vecinos son los **existentes más cercanos** y la cabecera declara los
      recuentos reales (`antes: 2, después: 2`); ninguna fila para un número inexistente.
- [x] 1.4 En el extremo de la parrilla (episodio con un solo anterior) no falla: muestra lo que hay.
- [x] 1.5 `--before 0` deja la ventana a un solo lado.
- [x] 1.6 Un episodio sin número NO aparece en la ventana aunque su fecha caiga en medio.
- [x] 1.7 La vecindad es **numérica, no de parrilla**: con las fechas de `678` y `679` invertidas,
      `679` sigue saliendo como siguiente.
- [x] 1.8 Los no publicados llevan `*` y la leyenda.
- [x] 1.9 `target` inexistente → el mismo error que `get`, código distinto de cero.
- [x] 1.10 `target` sin número (resuelto por slug) → error explícito, código distinto de cero, sin
      fallback a vecindad por fecha.
- [x] 1.11 No escribe nada: contenido y `mtime` del tracker intactos y sin `.bak`.
- [x] 1.12 `tests/json_output.rs`: `episode` + `before` + `after` en orden ascendente, con el
      inferior inmediato al final de `before` y el superior inmediato al principio de `after`; y el
      caso sin vecinos (listas vacías, claves presentes).
- [x] 1.13 `cargo test`: los nuevos fallan (el subcomando no existe) y los **340 existentes siguen
      verdes sin editarlos**.
- [x] 1.14 (auditoría) Tres tests más con **números repetidos**: marcador `►` una sola vez, `--after`
      completo pese a los duplicados, y tabla ≡ `--json`.

## 2. GREEN (implementación mínima)

- [x] 2.1 `src/commands/neighborhood.rs`: render de texto (cabecera + tabla).
- [x] 2.2 `src/commands/mod.rs`: `show_episodes` con marcador **opcional** de la referencia
      (`►` en la indentación, columnas alineadas); los puntos de llamada de `list`, `list-range` y
      `status` pasan el valor «sin marcador» **sin cambiar sus bytes**.
- [x] 2.3 `src/error.rs`: variante del episodio sin número para la vecindad (código de salida 1).
- [x] 2.4 `src/json.rs`: `neighborhood_document` (`episode`, `before`, `after`).
- [x] 2.5 `src/commands/mod.rs` y `src/main.rs`: módulo, subcomando, flags `--before`/`--after`
      (4/4 por defecto) y despacho.
- [x] 2.6 `cargo test` en verde: 340 previos + los nuevos.

## 3. REFACTOR

- [x] 3.1 `cargo fmt --all` y `cargo clippy --all-targets -- -D warnings` limpios.
- [x] 3.2 Sin `unwrap()`, `expect()` ni `panic!()` en producción.
- [x] 3.3 Ningún test toca `/data/notas` ni `/data/podcasts` (fixtures temporales).
- [x] 3.4 Pasada manual de **solo lectura** sobre el tracker real: un episodio publicado, uno con
      número provisional y uno en el extremo de la parrilla. — hecha sobre el perfil real (publicado, provisional y extremo), solo lectura.
- [x] 3.5 `docs/DEFINITION.md` §6: fila de `neighborhood` en la tabla de subcomandos (§6.1 sin
      cambios) y `README.md`: ejemplo de uso.

## 4. Verificación y cierre

- [x] 4.1 Los 340 tests previos pasan **sin editar un solo fichero de test existente** (guarda de
      que el marcador no contamina `list`/`list-range`/`status`). — 340 previos intactos; 363 en total.
- [x] 4.2 `just spec-validate episode-neighborhood` en estricto.
- [x] 4.3 `just check-all` en verde.
- [x] 4.4 Auditoría con el subagente `rust-reviewer` sobre el diff completo. — una pasada; H1 (marcador por número con duplicados) reparado y re-verificado.
- [ ] 4.5 PR `feature/episode-neighborhood` → `development`; release **minor** (`0.6.0`).
- [x] 4.6 `just spec-archive episode-neighborhood`. — archivado como openspec/changes/archive/2026-10-28-episode-neighborhood/; deltas fundidos (+1 en tracker-read, ~1 en json-output); openspec validate --specs --strict → 13/0.
- [ ] 4.7 `plans/PLAN-009.md` → `plans/PLAN-009-COMPLETED.md`.
- [x] 4.8 Tras archivar, comprobar que el `Purpose` de `openspec/specs/tracker-read/spec.md`
      enumera ya `neighborhood`: el `## Purpose` del delta es **descriptivo** (documenta por qué se
      toca la capacidad) y **no** sobrescribe el `Purpose` del spec fundido —eso solo ocurre en las
      capacidades **nuevas**—. Si no queda actualizado, editarlo a mano en el archivo. — el Purpose del delta se ignoró al fundir (aviso esperado); actualizado a mano en openspec/specs/tracker-read/spec.md para enumerar neighborhood.
