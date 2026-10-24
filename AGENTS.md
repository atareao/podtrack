# PodTrack — Instrucciones para agentes

> Las directivas globales de este entorno (OpenSpec SDD + TDD, tags de estado, tabla de subagentes) se inyectan ya en cada sesión desde `~/.config/opencode/AGENTS.md`. No se repiten aquí: lo de abajo es solo lo específico de este repositorio.

## Qué es PodTrack

El **rastreador de la parrilla y del ciclo de vida de los episodios** de un podcast. Sustituye al script Python `podcast-tracker.py` que hoy vive en `~/.agents/skills/podcast-srt/`, conservando su contrato de línea de comandos **byte a byte** y añadiendo, de forma opt-in, una salida `--json` para que los agentes dejen de *parsear texto bonito*.

No es un cliente de podcasts: no se suscribe a feeds, no descarga ni reproduce audio. Eso es otro proyecto (`podcli`).

- **Binario único** (`podtrack`), sin runtime de Python.
- **Multi-show**: el perfil del show (`shows/<slug>.yaml`) es el que declara la ruta del tracker, los días de publicación, las temporadas y el layout del material. `--show <slug>` viaja en toda invocación.
- **El número de episodio no es la clave.** La identidad estable es el `slug` (kebab-case, inmutable). El número se asigna **al publicar** con `publish` y queda congelado; mientras tanto puede llevar un número *provisional* (informativo, marcado con `*`).

## Fuente de verdad: `docs/DEFINITION.md`

El SRS del proyecto. Manda sobre este fichero cuando haya conflicto. Y por encima de él, para el **comportamiento**, manda la implementación de referencia:

| Fuente | Para qué |
|---|---|
| `~/.agents/skills/podcast-srt/podcast-tracker.py` | **Oráculo de comportamiento.** Salida de texto, exit codes, formato del YAML, invariantes. |
| `~/.agents/skills/podcast-srt/podcast-tracker.test.py` | **Suite de conformidad**: ~119 escenarios. Se portan 1:1 como tests de integración. |
| `~/.agents/skills/podcast-srt/SKILL.md` | Documentación de los subcomandos. |
| `~/.agents/openspec/specs/{episode-identity,episode-material-layout,episode-scheduling}` | Las capacidades ya especificadas del dominio. |
| `~/.agents/skills/podcast-srt/shows/*.yaml` + `shows/README.md` | Esquema y perfiles de show. |

**Regla de oro:** ante cualquier duda de comportamiento, la respuesta la da el Python, no la intuición. Este proyecto es un **port de paridad** primero y una mejora después.

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

| Change | Alcance |
|---|---|
| `bootstrap-core` | crate, modelo tipado, resolución de `--show`, writer YAML canónico y comandos de **solo lectura** (`status`, `next`, `list`, `where`, `get`) — paridad + `--json` |
| `write-commands` | `add`, `update`, `publish`, `insert`, `move` |
| `migration-surgery` | `renumber`, `rename-material`, `backfill-slugs`, `migrate-material`, `rebase-references` |

Los agentes del pipeline (`~/.config/opencode/agents/`) solo se reapuntan al binario **cuando las tres fases estén verdes**. Hasta entonces, el Python sigue siendo el que corre.

## Git Flow y convenciones

Ver [`GIT_FLOW.md`](./GIT_FLOW.md). Nada directo en `main` ni en `development`; conventional commits con gitmoji; release automatizada.

## Comandos

```bash
just check        # fmt + clippy + test (paridad exacta con el CI)
just check-all    # + rustdoc
just dev          # cargo run -- <args>
just fmt-fix      # aplica formato
```
