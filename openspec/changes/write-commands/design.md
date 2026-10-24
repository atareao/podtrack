# Design

## Context

La fase 1 dejó listo todo lo que esta necesita: el **modelo tipado**, la **resolución de show en cascada**, la **lectura** por `serde` y el **escritor canónico a mano** (`save_tracker`, con backup `.bak` y escritura atómica). Lo que falta es *usarlo*: los comandos que mutan la parrilla, y el único que además toca el disco (`insert`).

Restricciones que dan forma al diseño:

- La salida de texto es **contrato con nueve agentes**: byte a byte idéntica, incluido el `✓ Tracker actualizado: <ruta>` que imprime el guardado.
- Los tests **nunca** tocan `/data/notas`.
- `--dry-run` debe ser **inofensivo de verdad**: ni el tracker ni una sola carpeta renombrada.

## Goals / Non-Goals

**Goals:**

- Paridad byte a byte de `add`/`update`/`publish`/`insert`/`move` (texto + *exit codes* + tracker resultante).
- Semántica del número provisional/autoritativo estructural y bien probada.
- Renombrado de material fiable, ordenado y reutilizable por la fase 3.
- `--dry-run` sin efectos secundarios.
- `--json` aditivo para las escrituras.

**Non-Goals:**

- `renumber`, `rename-material`, `backfill-slugs`, `migrate-material`, `rebase-references` (fase 3).
- Reapuntar los agentes al binario (fase 5).
- Interfaz interactiva, TUI o `--help` idéntico al de `argparse`.

## Decisions

### D1. El `--dry-run` clona el modelo, no el fichero

El oráculo usa `copy.deepcopy(data)` para previsualizar sobre una copia y luego **no** llamar a `save_tracker`. Se porta igual: se clona el `Tracker` en memoria (derive `Clone`) y se calcula todo sobre la copia; el `--dry-run` solo *imprime*. En `insert`, el renombrado de material se **planifica** (lista de pares origen→destino) y se imprime, pero no se aplica.

*Alternativa:* escribir a un temporal — innecesario, y rompería el invariante «ni un solo efecto».

### D2. El número: dos conceptos, no uno

- **Provisional** (`_next_free_number`, `max(number de todos) + 1`): hueco de planificación que `add` asigna sin número explícito. Es informativo y **se sobrescribe**.
- **Autoritativo** (`_next_public_number`, `max(publicados) + 1`): se estampa **solo** en `publish`, nunca se reutiliza, y publicar **anula** (`number: null`) el provisional obsoleto que otro episodio no publicado ocupara con ese número.

Ya existen en la fase 1; aquí se dota al dominio de la mutación correspondiente.

### D3. El renombrado de material vive en `material.rs`

`insert` renombra carpetas `0NNN` y ficheros `0NNN`/`eNNN` mediante la misma maquinaria que la fase 3 expondrá como `rename-material`. Se factoriza ya: `renamed_component` (reescritura de tokens acotada por límites alfanuméricos, sin tocar secuencias de dígitos más largas), `plan_number_renames` (pares ordenados **de más profundo a menos**, para aplicarlos en secuencia) y `apply_renames` (aborta si el destino ya existe). La fase 3 solo añadirá el subcomando y su detección de huérfanos.

### D4. Slots de fecha con los días del show (lunes/jueves), como el oráculo

`insert` avanza la fecha de cada desplazado un *slot* (`_next_slot`) y `move` exige que la fecha nueva sea lunes o jueves (`PUBLISH_DAYS`). Igual que el oráculo, **hardcodeado** (el perfil no se consulta en esta fase); se documenta como la divergencia consciente n.º 7 de la fase 1.

### D5. Toda escritura pasa por `save_tracker`

Ningún comando escribe el fichero a mano: todos mutan el modelo y llaman al escritor canónico de la fase 1 (backup `.bak` + temp+rename) y a su línea de confirmación. Así el formato y la atomicidad siguen viviendo en un único sitio.

### D6. `--json` de escritura: acción + episodio + cambios

Cada comando emite `{ schema_version, show, action, episode: {...}, changes: [...] }` (forma concreta por comando; p. ej. `publish` incluye `number` estampado y los `nulled`; `insert` incluye los desplazamientos y el material renombrado). Aditivo: sin `--json` la salida de texto no cambia.

## Risks / Trade-offs

- **Escritura sobre el tracker real.** A diferencia de la fase 1, estas rutas mutan datos. *Mitigación:* todos los tests y el `diff` de paridad corren sobre fixtures temporales; el binario no se reapunta en producción hasta la fase 5.
- **Cirugía de material.** `insert` renombra en disco; un error de planificación corrompería el árbol. *Mitigación:* planificar antes de aplicar, ordenar de más profundo a menos, abortar si el destino existe, y `--dry-run` para previsualizar.
- **Deriva de paridad.** El oráculo puede cambiar mientras se porta. *Mitigación:* suite + *golden file* del tracker resultante y `diff` en vivo contra el oráculo.
- **Alcance de `insert`.** Se entrega **completo** (tracker + material), no partido, para no dejar un comando a medias ni duplicar la maquinaria en la fase 3.
