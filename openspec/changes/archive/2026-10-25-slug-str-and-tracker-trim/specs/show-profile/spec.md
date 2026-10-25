# Spec Delta

## ADDED Requirements

### Requirement: El valor de `tracker:` se interpreta sin espacios envolventes

El valor de `tracker:` del perfil SHALL interpretarse **recortado de espacios envolventes**
antes de resolverlo a ruta absoluta. Un perfil que declare `tracker: " /a/b/c.yaml "`
(entrecomillado, de modo que YAML preserva los espacios) SHALL resolver el tracker a la **misma
ruta** que si declarase `tracker: /a/b/c.yaml`. Un `tracker:` que, una vez recortado, quede
vacío o valga `none` SHALL tratarse como ausente, coherente con la guarda de ausencia.

#### Scenario: un tracker entrecomillado con espacios resuelve a la ruta limpia
- **GIVEN** un perfil que declara `tracker: " /data/notas/Podcasts/guiones/podcast-tracker.yaml "`
- **WHEN** se resuelve el perfil
- **THEN** la ruta del tracker SHALL ser `/data/notas/Podcasts/guiones/podcast-tracker.yaml`
- **AND** NO SHALL conservar espacios envolventes
- **AND** `<scripts.root>` SHALL ser `/data/notas/Podcasts/guiones`

#### Scenario: un tracker de solo espacios es ausencia
- **GIVEN** un perfil que declara `tracker: "   "`
- **WHEN** se resuelve el perfil
- **THEN** SHALL tratarse como tracker ausente, con su error tipado
