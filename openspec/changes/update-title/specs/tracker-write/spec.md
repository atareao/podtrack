# Spec Delta

## MODIFIED Requirements

### Requirement: `update` cambia el estado de un episodio

`update` SHALL resolver el episodio por número o por `slug` y SHALL cambiar su `status` y/o su `title`, según los flags `--status` y `--title`. SHALL exigir **al menos uno** de los dos: una invocación sin ninguno SHALL ser un error de uso. Un estado inválido, un **título vacío** o un objetivo inexistente SHALL ser un error con código distinto de cero, sin escribir. Al retitular, el `slug` NO SHALL cambiar: es la identidad estable y la ruta del material no se altera.

#### Scenario: cambio de estado
- **GIVEN** un episodio `scripted`
- **WHEN** se invoca `update <slug> --status recorded`
- **THEN** su `status` SHALL pasar a `recorded` y SHALL persistirse

#### Scenario: objetivo inexistente no escribe
- **GIVEN** un identificador que no corresponde a ningún episodio
- **WHEN** se invoca `update no-existe --status recorded`
- **THEN** SHALL salir con un código distinto de cero
- **AND** el tracker SHALL permanecer inalterado

#### Scenario: cambio de título conserva el slug
- **GIVEN** un episodio con `slug: la-memoria-de-valet`
- **WHEN** se invoca `update <slug> --title "¿Cómo se le da memoria a una IA?"`
- **THEN** su `title` SHALL cambiar al nuevo
- **AND** su `slug` SHALL seguir siendo `la-memoria-de-valet`

#### Scenario: estado y título a la vez
- **GIVEN** un episodio `scripted`
- **WHEN** se invoca `update <slug> --status recorded --title "Otro título"`
- **THEN** SHALL cambiar tanto el `status` como el `title`

#### Scenario: sin `--status` ni `--title` es error de uso
- **GIVEN** un episodio existente
- **WHEN** se invoca `update <slug>` sin ningún flag
- **THEN** SHALL salir con un código distinto de cero
- **AND** NO SHALL modificar el tracker

#### Scenario: título vacío no escribe
- **GIVEN** un episodio existente
- **WHEN** se invoca `update <slug> --title "   "`
- **THEN** SHALL salir con un código distinto de cero
- **AND** el tracker SHALL permanecer inalterado
