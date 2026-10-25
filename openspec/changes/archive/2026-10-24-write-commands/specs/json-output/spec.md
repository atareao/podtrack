# Spec Delta

## ADDED Requirements

### Requirement: Los comandos de escritura emiten un documento `--json` con la acción y su resultado

Con `--json`, cada comando de escritura SHALL emitir un documento JSON con `schema_version`, `show`, la `action` ejecutada, el episodio afectado y los cambios aplicados. Sin `--json`, la salida de texto SHALL permanecer inalterada.

#### Scenario: `publish --json` expone el número estampado
- **GIVEN** un no publicado `#843`
- **WHEN** se invoca `podtrack publish <slug> --json`
- **THEN** el JSON SHALL incluir `action: "publish"` y el `number` estampado
- **AND** SHALL ser un único documento JSON válido

### Requirement: `--dry-run` con `--json` informa sin aplicar

Con `--dry-run`, el documento `--json` SHALL describir los cambios **previstos** y SHALL marcar que no se aplicaron, sin escribir el tracker.

#### Scenario: dry-run se refleja en el documento
- **GIVEN** una invocación con `--json --dry-run`
- **WHEN** el comando termina
- **THEN** el JSON SHALL indicar que es una previsualización
- **AND** el tracker SHALL permanecer inalterado
