# Design

## Context

`update` nació como «cambia el estado» y su contrato está congelado en `tracker-write`.
El título es identidad **visible** (lo que se lista y se publica), mientras que el `slug`
es identidad **estable** (a dónde apunta el material). Retitular un episodio es legítimo y
hoy no tiene vía. Este cambio añade esa vía sin tocar nada más.

## Goals / Non-Goals

**Goals**

- Poder corregir el `title` de un episodio ya existente sin editarlo a mano.
- Mantener intacto el comportamiento (texto, exit codes, tracker) de `update --status`.
- Combinar estado y título en una invocación.

**Non-Goals**

- No se toca el `slug` (ni se re-deriva). Es identidad estable por diseño.
- No se renombran ficheros de material.
- No se añade `--dry-run` a `update` (fuera de alcance).

## Decisions

### D1 — Al menos uno de `--status` / `--title`

El CLI exige que esté presente uno de los dos (`required_unless_present` mutuo). `update`
sin ninguno es un error de uso, como lo era sin `--status`.

### D2 — El `slug` no cambia

Retitular cambia solo el `title`. El `slug` sigue siendo la identidad estable y la ruta
del material (`where`) no se altera. Si se quiere cambiar de slug, es otra cosa (y hoy no
existe vía, deliberadamente).

### D3 — Título vacío es error, no borrado

`--title ""` no borra el título: es un error. Un episodio sin título no tiene sentido en la
parrilla.

### D4 — Formato de salida

La línea de estado se conserva **byte a byte**. Cuando cambia el título se añade una línea
`Título: "viejo" → "nuevo"`. En `--json`, `changes` es una lista de `{field, from, to}`,
con `field: "status"` y/o `field: "title"` según lo que cambie.

## Divergencia respecto del oráculo (registro)

| Aspecto | Oráculo Python (retirado) | `podtrack` (este change) |
|---|---|---|
| `update --status` | sí | sí (sin cambio) |
| `update --title` | no existe | sí (capacidad nueva) |

Al ser capacidad nueva que no altera el camino existente, los casos del guardián de paridad
de `update` siguen valiendo tal cual.
