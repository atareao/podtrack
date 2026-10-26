# PodTrack — Instrucciones para agentes

> Las directivas globales de este entorno (OpenSpec SDD + TDD, tags de estado, tabla de subagentes) se inyectan ya en cada sesión desde `~/.config/opencode/AGENTS.md`. No se repiten aquí: lo de abajo es solo lo específico de este repositorio.

## Qué es PodTrack

El **rastreador de la parrilla y del ciclo de vida de los episodios** de un podcast. Sustituyó al script Python `podcast-tracker.py` que vivía en `~/.agents/skills/podcast-srt/`, conservando su contrato de línea de comandos **byte a byte** y añadiendo, de forma opt-in, una salida `--json` para que los agentes dejen de *parsear texto bonito*.

No es un cliente de podcasts: no se suscribe a feeds, no descarga ni reproduce audio. Eso es otro proyecto (`podcli`).

- **Binario único** (`podtrack`), sin runtime de Python.
- **Multi-show**: el perfil del show (`shows/<slug>.yaml`) es el que declara la ruta del tracker, los días de publicación, las temporadas y el layout del material. `--show <slug>` viaja en toda invocación.
- **El número de episodio no es la clave.** La identidad estable es el `slug` (kebab-case, inmutable). El número se asigna **al publicar** con `publish` y queda congelado; mientras tanto puede llevar un número *provisional* (informativo, marcado con `*`).

## Fuente de verdad: `docs/DEFINITION.md`

El SRS del proyecto. Manda sobre este fichero cuando haya conflicto. Y por encima de él, para el **comportamiento**, mandan el binario y su suite de tests:

| Fuente | Para qué |
|---|---|
| `openspec/specs/` (capacidades ya fundidas: `tracker-read`, `tracker-write`, `srt-stage`, `python-retirement`, …) | Las capacidades del dominio, ya especificadas y verificadas. |
| `tests/golden/` | **Golden congelado del oráculo retirado.** Evidencia histórica de su comportamiento; **no se regenera** y ningún test puede escribirlo. |
| `~/.agents/openspec/specs/{episode-identity,episode-material-layout,episode-scheduling}` | Las capacidades ya especificadas del dominio. |
| `~/.agents/skills/podcast-srt/shows/README.md` | Esquema canónico del perfil de show. |
| `~/.agents/skills/podcast-srt/SKILL.md` | Documentación de los subcomandos, ya sobre el binario. |

**Regla de oro:** ante cualquier duda de comportamiento, la respuesta la dan el binario y su suite, no la intuición. El golden de `tests/golden/` es la única evidencia del comportamiento del oráculo retirado — **se lee, nunca se reescribe**.

**El Python fue *andaime*, no cimiento.** Existió para garantizar la paridad mientras se construía el reemplazo y **se derribó** al terminar: `podcast-tracker.py`, su suite y `srt-stage.py` se borraron y los agentes pasaron a invocar el binario. El invariante vivo del proyecto es **Python cero**, vigilado por `tests/no_python.rs`.

## Protocolo SDD (OpenSpec)

En este repo **no se escribe código ni tests sin un change proposal aprobado** en `openspec/changes/`. El flujo completo vive en el `.justfile`:

```bash
just spec-list                 # changes activos
just spec-new bootstrap-core   # crea openspec/changes/<name>/
just spec-validate <name>      # valida en estricto
just spec-status <name>        # artefactos que faltan
just spec-archive <name>       # funde el delta en openspec/specs/
```

## Trabajo en fases

Las cinco fases del port **están archivadas** en `openspec/changes/archive/`, y sus deltas fundidos en `openspec/specs/`:

| Change archivado | Alcance |
|---|---|
| `2026-10-24-bootstrap-core` | **tracker** — crate, modelo tipado, resolución de `--show`, writer YAML canónico y comandos de **solo lectura** (`status`, `next`, `list`, `where`, `get`) — paridad + `--json` |
| `2026-10-24-write-commands` | **tracker** — `add`, `update`, `publish`, `insert`, `move` |
| `2026-10-24-migration-surgery` | **tracker** — `renumber`, `rename-material`, `backfill-slugs`, `migrate-material`, `rebase-references` |
| `2026-10-25-srt-stage` | **clasificador SRT** — `srt-stage.py` portado al binario (etapas `raw`/`final`; lo usan 4 agentes) |
| `2026-10-25-retire-python` | **derribar el andaime** — conmutó agentes y `SKILL.md` al binario y **borró** los tres `.py` de la skill |

Los agentes del pipeline (`~/.config/opencode/agents/`) **ya invocan `podtrack`**. El éxito del proyecto —que no quede ningún `.py` en la skill del podcast— **está cumplido**: Python cero, vigilado por `tests/no_python.rs`.

## Git Flow y convenciones

Ver [`GIT_FLOW.md`](./GIT_FLOW.md). Nada directo en `main` ni en `development`; conventional commits con gitmoji; release automatizada.

## Comandos

```bash
just check        # fmt + clippy + test (paridad exacta con el CI)
just check-all    # + rustdoc
just dev          # cargo run -- <args>
just fmt-fix      # aplica formato
```
