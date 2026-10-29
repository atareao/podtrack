# Proposal

## Why

`podtrack` tiene 20 subcomandos y **siete** de ellos (`get`, `where`, `open`, `neighborhood`,
`update`, `publish`, `move`) identifican el episodio con un `target` que puede ser un **número** o
un **slug** kebab-case. Hoy ninguna de las dos cosas se autocompleta: hay que teclear el slug entero
—`la-memoria-de-valet`, `bitwarden-vs-keeper`— o recordar el número, y tampoco hay completado de
subcomandos ni de flags.

La fricción es diaria y los errores de tecleo solo se descubren al ejecutar
(`EpisodeNotFound`, exit `1`).

## What Changes

1. Subcomando nuevo **`podtrack completions <shell>`**, con `<shell>` ∈ {`bash`, `zsh`, `fish`}, que
   imprime en `stdout` el **script de registro** del completado. No depende de un show resuelto ni
   de un tracker existente: funciona fuera de todo show y no escribe ningún fichero.
2. **Completado estático** de subcomandos, flags y los valores que `clap` ya enumera (`--status`,
   `--expect`), que sale de la propia definición del `Command`.
3. **Completado dinámico** del `target` de `get`, `where`, `open`, `neighborhood`, `update`,
   `publish` y `move`: el **número** y el **slug** de cada episodio del show resuelto.
4. **Completado dinámico** del valor de `--show`: los **slugs de perfil** de los directorios de la
   cascada del resolver.
5. El completado **nunca** altera el comportamiento del binario: sin la variable `COMPLETE` en el
   entorno, la salida es byte a byte la de siempre.

## Capabilities

### New Capabilities

- `shell-completions`: el subcomando que emite el script de registro, el contrato del completado
  estático y dinámico, y la garantía de no interferencia con el binario.

### Modified Capabilities

Ninguna.

## Impact

- `Cargo.toml` / `Cargo.lock`: dependencia nueva `clap_complete` con el feature `unstable-dynamic`
  (arrastra `is_executable`, `shlex` y `clap_lex`).
- `src/main.rs`, `src/lib.rs`, `src/commands/mod.rs`, `src/profile.rs`.
- `src/completions.rs` y `src/commands/completions.rs` (**nuevos**).
- `tests/completions.rs` (**nuevo**) y `tests/common/mod.rs` (guarda `COMPLETE`).
- `docs/DEFINITION.md` §6 y `README.md`.
- **Sin impacto** en `tests/golden/` (solo lectura), en la salida de ningún comando existente ni en
  las capacidades ya fundidas.
