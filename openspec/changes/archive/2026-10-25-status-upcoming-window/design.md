# Design

## Context

`status` nació como espejo del `cmd_status` del oráculo Python, con paridad byte a byte.
Su bloque final («Últimos 8 episodios») era una ventana por orden de parrilla
**invertida** (`ordered.iter().rev().take(8)`): los 8 episodios más recientes de la
parrilla, con publicados y no publicados mezclados y sin ninguna discriminación por
estado. Reproducía el guion original, pero no el uso real: `status` es el panel desde el
que se decide qué grabar y qué publicar.

El requisito de paridad estricta ya se relajó al retirar el oráculo (change
`retire-python`): a partir de ahí el contrato vive en `openspec/specs/`, no en el `.py`.
Este cambio explota esa libertad por primera vez, de forma acotada a la ventana final.

## Goals / Non-Goals

**Goals**

- Que la ventana responda de un vistazo a «¿qué está publicado y qué toca publicar?».
- Reutilizar los helpers existentes (`last_published`, el orden de `sort_key`) sin
  introducir una noción nueva de orden.
- Mantener el coste visual: 8 filas, ni una más.

**Non-Goals**

- No se toca el resumen superior (temporada, totales, «▶ Siguiente», bloque
  «Siguiente a publicar»).
- No se toca el documento `--json` (ya expone `last_published` y `next_to_publish`).
- No se introduce un flag `--window`/`--last`: el 8 (= 1 + 7) es fijo, es el contrato.

## Decisions

### D1 — La ventana es «último publicado + 7 próximos», no los 8 últimos de parrilla

El operador necesita dos anclas: el último hito publicado (contexto temporal) y la
cola inmediata a publicar (la acción). Los 8 últimos por parrilla invertida daban lo
contrario: en una parrilla con muchos no publicados futuros, la ventana se llenaba de
futuros lejanos y desplazaba fuera de cuadro al último publicado.

**Alternativas descartadas:**

- *Solo los 7 próximos:* pierde la referencia «dónde me quedé».
- *Solo el último publicado:* pierde la cola a publicar.
- *Configurable por perfil:* sobre-ingeniería; el número 8 ya es un hábito de lectura.

### D2 — El orden conserva la semántica de fecha, no la numérica

El publicado se elige con `last_published` (máximo por `date.to_iso()`, empate → primero
encontrado) y los próximos con el orden de `sort_key()` ya usado por `next_to_publish`.
No se reordena por número: en una parrilla con huecos provisionales el número miente, la
fecha no.

### D3 — Degradación explícita y simétrica

Sin publicado → cola de próximos. Sin próximos → solo el publicado. Sin ninguno →
`No hay episodios.` (ya la emite `show_episodes` con la lista vacía). Se evita cualquier
caso en el que la ventana quede muda sin explicar por qué.

### D4 — Solo texto; `--json` intacto

El documento JSON ya separa `last_published` y `next_to_publish`. Añadir una ventana
duplicada sería redundante y rompería consumidores sin necesidad. Si en el futuro se
quiere, se hará como campo **aditivo** en su propio change.

## Divergencia respecto del oráculo (registro)

| Aspecto | Oráculo Python (retirado) | `podtrack` (este change) |
|---|---|---|
| Selección | 8 últimos por orden de parrilla invertida | último publicado + 7 próximos |
| Mezcla de estados | sí | no (publicado primero, luego no publicados) |
| Marca provisional | sí (sobre los no publicados presentes) | sí, pero solo sobre la cola de próximos |
| Cabecera | `Últimos 8 episodios:` | refleja el nuevo contenido |

## Migración / compatibilidad

Cambio de **presentación de texto**, sin efectos sobre el tracker ni el material. Quien
parseara la salida de `status` para leer la ventana debería migrar a `--json`; no hay
consumidor programático conocido (los agentes usan `status` como panel humano y `get`/
`list` para datos).
