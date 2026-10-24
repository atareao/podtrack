# Spec Delta

## Purpose

Definir el renombrado de los **tokens de número** del material de guion cuando un episodio cambia de número (`0NNN` → `0MMM`, `eNNN` → `eMMM`). Es la maquinaria que `insert` usa al desplazar la parrilla y que la fase `migration-surgery` reutilizará para `rename-material`.

## ADDED Requirements

### Requirement: Solo se reescriben los tokens de número reconocidos

El renombrado SHALL reconocer la forma rellena a cuatro dígitos (`0NNN`) y la forma con prefijo `e` (`eNNN`), acotadas por límites alfanuméricos, y NO SHALL tocar secuencias de dígitos más largas ni identificadores que las contengan.

#### Scenario: el token `0NNN` se reescribe
- **GIVEN** un material con la carpeta `0843` y el fichero `e843-articulo.md`
- **WHEN** se renombra el material de `843` a `844`
- **THEN** la carpeta SHALL pasar a `0844` y el fichero a `e844-articulo.md`

#### Scenario: no se toca una secuencia más larga
- **GIVEN** un nombre que contiene `08430`
- **WHEN** se renombra de `843` a `844`
- **THEN** ese nombre NO SHALL cambiar

### Requirement: El plan de renombrado se ordena de más profundo a menos

El renombrado SHALL planificarse como pares origen→destino ordenados **de más profundo a menos** (primero ficheros y subcarpetas, y la carpeta del episodio al final), de modo que se puedan aplicar en secuencia sin invalidar rutas intermedias.

#### Scenario: se renombran hijos antes que el padre
- **GIVEN** un árbol `0843/` con ficheros dentro
- **WHEN** se planifica el renombrado a `0844`
- **THEN** los ficheros SHALL aparecer antes que la carpeta en el plan

### Requirement: El renombrado aborta si un destino ya existe

Al aplicar el plan, si un destino ya existe, el binario SHALL abortar con un error y un código distinto de cero, informando de la ruta en conflicto, y NO SHALL dejar el árbol a medias.

#### Scenario: destino ocupado
- **GIVEN** un material donde `0844/` ya existe
- **WHEN** se intenta renombrar `0843/` a `0844/`
- **THEN** SHALL error con código distinto de cero y sin renombrar

### Requirement: La aplicación reporta cada renombrado

Al aplicar el plan (sin `--dry-run`), cada renombrado SHALL reportarse en `stdout` con la ruta relativa al árbol de guiones, origen y destino.

#### Scenario: se reporta cada par
- **GIVEN** un renombrado aplicado con éxito
- **WHEN** termina
- **THEN** cada par origen→destino SHALL aparecer en `stdout`
