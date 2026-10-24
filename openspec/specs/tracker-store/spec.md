# tracker-store Specification

## Purpose
Definir el modelo tipado del episodio y las reglas de lectura y escritura del YAML de estado (la parrilla). Es la base sobre la que se apoyan tanto los comandos de lectura de esta fase como los de escritura de las fases siguientes.

## Requirements

### Requirement: El episodio es una entidad tipada

Un episodio SHALL modelarse con los campos `number` (entero opcional), `slug` (opcional), `season`, `date`, `title` y `status`. `status` SHALL ser uno de `pending`, `scripted`, `recorded` o `published` —los mismos cuatro del esquema canónico del perfil (`shows/README.md`) y de la implementación de referencia—. Un `status` fuera de ese conjunto SHALL ser un error de lectura, no un valor tolerado.

#### Scenario: un estado desconocido no se acepta
- **GIVEN** una entrada del tracker con `status: cancelled`
- **WHEN** se lee el tracker
- **THEN** SHALL fallar indicando que el estado no es válido

#### Scenario: `number` puede faltar
- **GIVEN** una entrada con `number: null`
- **WHEN** se lee el tracker
- **THEN** el episodio SHALL cargarse con su número ausente
- **AND** NO SHALL ser un error

### Requirement: El `slug` es opcional y kebab-case

Cuando un episodio declara `slug`, SHALL ser una cadena en kebab-case. Un `slug` mal formado SHALL ser un error de lectura.

#### Scenario: un slug válido se carga tal cual
- **GIVEN** una entrada con `slug: la-memoria-de-valet`
- **WHEN** se lee el tracker
- **THEN** el episodio SHALL exponer ese slug sin transformarlo

### Requirement: La parrilla se ordena por fecha, no por número

La colección de episodios SHALL ordenarse por `date` ascendente. `number` SHALL usarse únicamente como desempate entre fechas iguales. Los episodios sin fecha (`TBD` o vacía) SHALL colocarse al final del orden.

#### Scenario: el número no ordena
- **GIVEN** un episodio con `date: 2026-10-26` y `number: 900` y otro con `date: 2026-10-29` y `number: 843`
- **WHEN** se listan ordenados
- **THEN** el de `2026-10-26` SHALL aparecer primero, pese a tener número mayor

#### Scenario: sin fecha va al final
- **GIVEN** un episodio con `date: TBD`
- **WHEN** se listan ordenados
- **THEN** SHALL aparecer después de todos los que sí tienen fecha

### Requirement: La escritura reproduce un formato canónico

Al guardar, el binario SHALL reescribir el tracker entero con un formato fijo: la cabecera de comentarios literal del show, el orden de campos de nivel superior (`season`, `season_start`, `episode_duration_minutes`, `publish_days`, `episodes`) y, por episodio, el orden `number`, `slug` (solo si existe), `season`, `date`, `title` entrecomillado y `status`. NO SHALL hacer *round-trip* de YAML genérico ni perder la cabecera de comentarios.

#### Scenario: la cabecera de comentarios sobrevive
- **GIVEN** un tracker con su cabecera de comentarios
- **WHEN** se guarda tras una operación
- **THEN** el fichero resultante SHALL conservar la cabecera de comentarios
- **AND** SHALL escribir los episodios en el orden de campos canónico

#### Scenario: `slug` solo se escribe cuando existe
- **GIVEN** un episodio sin `slug`
- **WHEN** se guarda el tracker
- **THEN** su entrada NO SHALL incluir la línea `slug:`

### Requirement: Toda escritura hace backup y es atómica

Antes de sobrescribir el tracker, el binario SHALL copiar el fichero previo a `<tracker>.yaml.bak`. La escritura SHALL ser atómica (escribir a un temporal y renombrarlo sobre el destino), de modo que un fallo a mitad NO SHALL dejar el tracker a medias.

#### Scenario: se conserva el estado anterior
- **GIVEN** un tracker existente
- **WHEN** se realiza una escritura
- **THEN** SHALL existir `<tracker>.yaml.bak` con el contenido previo
