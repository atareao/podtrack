//! Subcomando `completions`: emite el script de registro del autocompletado.

use crate::commands::Rendered;
use crate::completions::Shell;

/// Parámetros de `completions`.
pub struct CompletionsParams {
    /// Shell para el que generar el script.
    pub shell: Shell,
}

/// Emite en `stdout` el script de registro del completado para el shell pedido.
///
/// No opera sobre la parrilla: no necesita ni perfil ni tracker, así que se despacha antes de
/// resolver el show, y no escribe ningún fichero. El flag global `--json` no aplica —la salida es
/// un script, no un documento— y se ignora.
///
/// # Errors
///
/// Falla solo si el script no se puede componer en memoria.
pub fn run(params: &CompletionsParams) -> anyhow::Result<Rendered> {
    let mut buf = Vec::new();
    params.shell.write_registration(&mut buf)?;
    Ok(Rendered::stdout(String::from_utf8(buf)?))
}
