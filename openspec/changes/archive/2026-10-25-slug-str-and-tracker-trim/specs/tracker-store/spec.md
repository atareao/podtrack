# Spec Delta

## ADDED Requirements

### Requirement: La lectura del slug en el modelo es canónica

El modelo del episodio SHALL exponer su `slug` de forma **canónica**: el valor **recortado de
espacios envolventes y no vacío**, o la ausencia (`None`) cuando el episodio no tiene slug, lo
tiene vacío o solo tiene espacios. Todos los consumidores —el texto de `status`, `get` y
`where`, y los documentos `--json`— SHALL leer el slug por esta vía canónica, de modo que un
mismo episodio SHALL presentar **el mismo** valor de slug en cualquier salida.

Un `slug` válido no vacío se expone **sin transformarlo** (salvo el recorte de espacios
envolventes, que en un slug válido es la identidad). Esta lectura canónica es **defensiva**: el
cargador ya garantiza que un episodio cargado tiene un slug kebab-case o ninguno, así que para
datos cargados NO SHALL cambiar la salida observable del CLI. El requisito fija el contrato del
modelo para que ningún camino lo eluda.

#### Scenario: un slug cargado se expone tal cual
- **GIVEN** un episodio cargado con `slug: la-memoria-de-valet`
- **WHEN** se lee su slug por la vía canónica
- **THEN** SHALL devolver `la-memoria-de-valet`

#### Scenario: un slug con espacios envolventes se expone recortado
- **GIVEN** un `Episode` construido en memoria cuyo slug es `" la-memoria-de-valet "` (caja blanca: el cargador no produce este caso)
- **WHEN** se lee su slug por la vía canónica
- **THEN** SHALL devolver `la-memoria-de-valet`, sin los espacios

#### Scenario: un slug ausente, vacío o de solo espacios se lee como ausencia
- **GIVEN** un `Episode` sin slug, o con un slug vacío o de solo espacios
- **WHEN** se lee su slug por la vía canónica
- **THEN** SHALL devolver la ausencia (`None`)
- **AND** las salidas que dependen del slug SHALL comportarse como «sin slug» (`—` en el texto, `null` en el JSON)
