# Spec Delta

## MODIFIED Requirements

### Requirement: `insert` desplaza números, fechas y material

`insert --before <num> --title "<título>"` SHALL colocar el episodio nuevo en el número y la fecha del que hoy ocupa `<num>`, SHALL avanzar `number + 1` y un *slot* de fecha a todos los de número `>= <num>`, y SHALL renombrar el material asociado de los desplazados. Una fecha aportada que no coincida con el slot SHALL ser un error. Un `<num>` inexistente SHALL ser un error.

El episodio nuevo SHALL derivar su `slug` del título (`slugify` a kebab-case),
sufijándolo `-2`, `-3`, … si colisiona con un `slug` ya presente en el tracker, salvo que
se aporte `--slug`, en cuyo caso SHALL usarse tal cual. Si el título no produce un `slug`
utilizable y no se aporta `--slug`, el episodio SHALL quedar sin `slug`.

#### Scenario: desplaza y renombra
- **GIVEN** un tracker con `#843` en `2026-10-26` y su material `0843/`
- **WHEN** se invoca `insert --before 843 --title "Urgente"`
- **THEN** el nuevo SHALL quedar `#843` en `2026-10-26`
- **AND** el antiguo SHALL pasar a `#844` con la fecha del slot siguiente
- **AND** su material SHALL renombrarse de `0843` a `0844`

#### Scenario: deriva el slug del título
- **GIVEN** un tracker sin ningún `slug` que colisione
- **WHEN** se invoca `insert --before 843 --title "Cómo montar un NAS"`
- **THEN** el episodio nuevo SHALL quedar con `slug: como-montar-un-nas`

#### Scenario: el slug derivado se sufija si colisiona
- **GIVEN** un tracker que ya tiene un episodio con `slug: urgente`
- **WHEN** se invoca `insert --before 843 --title "Urgente"`
- **THEN** el episodio nuevo SHALL quedar con `slug: urgente-2`

#### Scenario: `--slug` explícito manda
- **GIVEN** un tracker cualquiera
- **WHEN** se invoca `insert --before 843 --title "Urgente" --slug mi-slug`
- **THEN** el episodio nuevo SHALL quedar con `slug: mi-slug`

#### Scenario: título sin slug utilizable y sin `--slug`
- **GIVEN** un título que `slugify` reduce a vacío
- **WHEN** se invoca `insert --before 843 --title "¡¡¡"`
- **THEN** el episodio nuevo SHALL quedar sin `slug`
