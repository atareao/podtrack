# podtrack

<p align="center">
  <strong>Tracker de la parrilla y del ciclo de vida de los episodios de un podcast, en Rust.</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/github/actions/workflow/status/atareao/podtrack/ci.yml?branch=main&style=flat-square" alt="build"/>
  <img src="https://img.shields.io/github/last-commit/atareao/podtrack?style=flat-square" alt="last-commit"/>
  <img src="https://img.shields.io/github/license/atareao/podtrack?style=flat-square" alt="license"/>
</p>

---

## Descripción

`podtrack` gobierna **los metadatos y el layout** del material de un podcast: qué episodios existen, cuándo salen, en qué estado están y dónde vive su material de guion. Es un binario único, sin runtime de Python, pensado para que el resto del pipeline (agentes de guion, grabación y publicación) lo invoque muchas veces por episodio.

Nació como **port de paridad** del script `podcast-tracker.py` (Python), con su mismo contrato de línea de comandos —y además ofrece, de forma *opt-in*, una salida `--json` para consumidores programáticos. Su razón de ser era **desterrar Python**: el script se retiró y el binario es hoy lo único que corre.

### La idea en una frase

La identidad de un episodio es su **`slug`**, no su número. El número se asigna **al publicar** y queda congelado, así que insertar o reordenar episodios nunca obliga a renumerar ni renombrar el material derivado.

## Características

- **Multi-show**: cada podcast se describe con un perfil (`shows/<slug>.yaml`); `--show <slug>` viaja en toda invocación.
- **Identidad por slug**: reordenar la parrilla no rompe nada.
- **Número al publicar**: `publish` estampa `max(publicados) + 1` y lo congela.
- **Material por slug**: el material de guion vive en `<scripts.root>/<slug>/`, nunca indexado por número.
- **Relleno de huecos**: `insert`, `move`, `renumber` y `rename-material` reordenan sin cirugía manual.
- **Vecindad numérica**: `neighborhood` lista los episodios alrededor de uno, con la referencia marcada en su sitio.
- **`--json` opt-in**: estructura para los agentes, texto bonito para las personas.
- **Seguro por defecto**: backup `.bak` y escritura atómica en cada operación; `--dry-run` en todo lo que reordena.

## Instalación

```bash
cargo install --git https://github.com/atareao/podtrack
```

O desde el código:

```bash
git clone https://github.com/atareao/podtrack
cd podtrack
cargo build --release
```

## Uso

```bash
# Estado de la parrilla y siguiente a publicar
podtrack status --show atareao-con-linux

# Siguiente hueco y próximo número autoritativo
podtrack next --show atareao-con-linux

# Dónde vive el material de un episodio
podtrack where la-memoria-de-valet --show atareao-con-linux

# Los cuatro episodios anteriores y los cuatro siguientes a este número
podtrack neighborhood 843 --show atareao-con-linux

# Lo mismo, en JSON
podtrack status --json --show atareao-con-linux
```

## Desarrollo

```bash
just check       # fmt + clippy + test (lo mismo que el CI)
just check-all   # + rustdoc
just spec-list   # changes OpenSpec activos
```

El proyecto sigue **OpenSpec (SDD) + TDD**: no hay código sin un change proposal aprobado. Ver [`AGENTS.md`](./AGENTS.md) y [`docs/DEFINITION.md`](./docs/DEFINITION.md).

## Licencia

MIT © Lorenzo Carbonell
