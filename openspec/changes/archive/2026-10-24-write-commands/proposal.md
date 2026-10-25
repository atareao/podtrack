# Proposal

## Why

La fase 1 (`bootstrap-core`) portó la **mitad de lectura** del contrato: `podtrack` ya lee la parrilla con paridad byte a byte y sabe escribir el YAML canónico (con backup y atomicidad), pero **no hay ni un comando que lo use para escribir**. El tracker real sigue gobernado por `podcast-tracker.py`.

Esta fase cierra la otra mitad: los **cinco comandos que mutan la parrilla** —`add`, `update`, `publish`, `insert` y `move`—, que son los que los agentes invocan en cada episodio. Aquí está la semántica más delicada del dominio: el **número provisional** (un hueco de planificación, sobrescribible) frente al **número público autoritativo** (se estampa al publicar y nunca se reutiliza), y el único comando que toca el **material en disco** (`insert` desplaza números y renombra las carpetas/ficheros afectados).

Se sigue empezando por lo verificable: estas cinco rutas se pueden portar y comparar **byte a byte** contra el oráculo sobre fixtures temporales, sin tocar `/data/notas`.

## What Changes

1. **Comandos de escritura con paridad de contrato:** `add` (número opcional, hueco provisional, slug derivado), `update` (cambio de estado), `publish` (estampa `max(publicados)+1`), `insert` (inserta en el hueco de otro, desplazando números, fechas y material) y `move` (cambia la fecha de emisión). Mismos iconos, texto, y *exit codes* que el oráculo.

2. **Semántica del número, estructural:** el número **provisional** de un no publicado se sobrescribe al publicar y se marca con `*` en lectura; el número **público autoritativo** se estampa en `publish` (`max(publicados)+1`) y nunca se reutiliza; publicar **anula** el provisional obsoleto que ocupara ese número en otro episodio.

3. **`--dry-run` para `publish`, `insert` y `move`:** previsualiza sin escribir **nada** —ni el tracker ni el disco (material)—.

4. **Renombrado del material por `insert`:** al desplazar `#N → #N+1`, renombra las carpetas `0NNN` y los ficheros `0NNN`/`eNNN` del árbol de guiones. Es la misma maquinaria que la fase 3 (`migration-surgery`) reutilizará para `rename-material`, así que se factoriza en un módulo propio.

5. **`--json` extendido a las escrituras:** cada comando de escritura emite un documento versionado con la acción, el episodio y los cambios aplicados; la salida por defecto (texto) no cambia.

6. **Suite de paridad de escritura:** port 1:1 de los escenarios del oráculo que cubren estas rutas, sobre fixtures temporales, incluido el *golden file* del tracker resultante.

## Capabilities

### New Capabilities
- `tracker-write`: la superficie de escritura de la parrilla —`add`, `update`, `publish`, `insert`, `move`—, con la semántica provisional/autoritativo del número, `--dry-run` y los *exit codes*.
- `material-renumber`: el renombrado de los tokens de número del material de guion (`0NNN`, `eNNN`) cuando un episodio cambia de número.

### Modified Capabilities
- `json-output`: se añade el contrato de los documentos de los comandos de **escritura** (aditivo; la salida por defecto no cambia).

## Impact

- **`src/commands/`** — comandos nuevos: `add`, `update`, `publish`, `insert`, `move`.
- **`src/tracker.rs`** — primeros usos reales de `save_tracker` (backup + atómico ya implementados en la fase 1).
- **`src/material.rs`** (nuevo) — renombrado de material (compartido con la fase 3).
- **`src/json.rs`** — documentos de escritura.
- **`tests/`** — suite de paridad de escritura con fixtures temporales.
- **No** se toca `/data/notas`, ni los agentes, ni los `.py` de la skill: el Python sigue siendo el que corre en producción.
