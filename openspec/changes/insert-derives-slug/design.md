# Design

## Context

`insert` se portó 1:1 desde el `cmd_insert` del oráculo Python. Aquel no derivaba el
`slug` del episodio nuevo: lo dejaba `null` salvo `--slug` explícito. En ese momento no
importaba porque el material se numeraba por carpeta (`0843/`); con la migración a
material direccionado por **slug**, un episodio sin slug quedó huérfano: `where` no
puede resolver su ruta y el pipeline F1-F8 no puede crearla.

El requisito de paridad estricta ya se relajó al retirar el oráculo (change
`retire-python`): desde entonces el contrato vive en `openspec/specs/`. Este cambio
explota esa libertad de forma acotada a la derivación del slug.

## Goals / Non-Goals

**Goals**

- Que un `insert` sin `--slug` produzca un episodio **trabajable de inmediato** (con
  slug), sin `backfill-slugs` correctivo.
- Reutilizar la regla de unicidad ya vigente en `add`: una sola implementación.
- Cero cambio observable más allá del `slug` del episodio nuevo.

**Non-Goals**

- No se toca el desplazamiento de números/fechas/material.
- No se toca la forma del documento `--json` ni del `--dry-run`.
- No se unifica la derivación con `rename-material` ni con la migración a slug.

## Decisions

### D1 — Se deriva con la misma regla que `add`, sin inventar una nueva

`slug = slugify(title)` y, si colisiona con un slug existente, se sufija `-2`, `-3`, …
El algoritmo ya vive **duplicado** en `add::unique_slug` y `backfill_slugs::unique_slug`.
En vez de escribir una tercera copia se extrae a `model::unique_slug`, parametrizado por
un iterador de slugs usados, y los tres comandos lo consumen.

**Alternativa descartada:** copiar el bloque en `insert`. Es la tercera copia de un
helper de 15 líneas; el coste de mantenerlas sincronizadas supera el de extraerlo.

### D2 — La unicidad se comprueba contra los slugs del tracker, no contra el disco

Igual que `add`: la fuente de verdad de los slugs vivos es el tracker, no el árbol de
guiones (que puede tener material legado numerado o carpetas huérfanas). Se compara
contra `tracker.episodes[*].slug`.

### D3 — Título sin slug utilizable ⇒ episodio sin slug (paridad con `add`)

`slugify("¡¡¡")` es vacío. `add` deja `slug: None` en ese caso y `insert` hará lo mismo:
mejor un episodio sin slug (que `backfill-slugs` podrá tratar) que un slug inventado
vacío o inválido. `Slug::parse("")` fallaría, así que el caso se corta antes.

### D4 — `--slug` explícito manda sin tocar la unicidad

Como hoy en `add`: si el operador pasa `--slug`, se usa tal cual (`Slug::parse` valida el
formato kebab-case). No se le añade sufijo aunque colisione: es su elección explícita.

## Divergencia respecto del oráculo (registro)

| Aspecto | Oráculo Python (retirado) | `podtrack` (este change) |
|---|---|---|
| `slug` del episodio nuevo sin `--slug` | `null` | derivado del título, con unicidad |
| `--slug` explícito | se usa | se usa (sin cambio) |
| Desplazamiento de números/fechas/material | sí | sí (sin cambio) |

Los casos del guardián de paridad viva (`tests/parity_write_oracle.rs`) que invocan
`insert` se fijan con `--slug` explícito: así siguen ejerciendo el desplazamiento y el
material —lo que sí reproduce el oráculo— sin depender de la parte que ahora diverge.

## Migración / compatibilidad

- Trackers ya escritos por `insert` pueden tener episodios sin slug: los sigue cubriendo
  `backfill-slugs`, sin cambios.
- Consumidores de `--json` de `insert`: el campo `episode.slug` pasa de `null` a un
  valor. Es un endurecimiento (nuevo dato), no un cambio de forma.
