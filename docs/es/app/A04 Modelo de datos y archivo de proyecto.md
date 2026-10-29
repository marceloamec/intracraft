---
title: A04 Modelo de datos y archivo de proyecto
lang: es
tags: [app, data-model, file-format]
---

🌐 [[A04 Data Model and Project File|EN]] · **ES** · [[A04 Datenmodell und Projektdatei|DE]] · [[Inicio (ES)|Índice]]

# A04 Modelo de datos y archivo de proyecto

## 1. Entidades

| Entidad   | Contenido                                                                                            | Estado              |
| --------- | ---------------------------------------------------------------------------------------------------- | ------------------- |
| Proyecto  | `schemaVersion`, nombre, semilla, layout (más adelante: librerías usadas, unidades de visualización) | Esquema creado      |
| Layout    | Instancias, conexiones (más adelante: controladores, escenarios)                                     | Esquema creado      |
| Instancia | `id`, `definitionId` + `definitionVersion`, nombre, posición (x, y, z), giro, parámetros             | Esquema creado      |
| Puerto    | Posición local, dirección, altura de superficie, ancho útil, tolerancias                             | En las definiciones |
| Conexión  | Del puerto de salida de A al de entrada de B, con `gap` (0,02 m por defecto)                         | Esquema creado      |
| Señal     | Booleana con nombre, origen y destino                                                                | Prevista            |
| Pallet    | Tipo, masa, carga, estado y ubicación (objeto + coordenada)                                          | Tipos creados       |
| Escenario | Layout + generadores + semilla + duración + métricas esperadas                                       | Solo analíticos     |

Código: `packages/core-model/src/schema.ts` (Zod). Los tipos se deducen de los esquemas (`Project`, `Layout`, `Instance`, `Connection`).

## 2. Puertos y conexiones

- Cada puerto declara **altura de superficie**, **ancho útil** y **tolerancias** de alineación (por defecto ±10 mm de altura y ±20 mm laterales).
- El editor encaja puertos cercanos y avisa si altura, ancho u orientación no coinciden.
- Los puertos de las giratorias viven en el marco de la plataforma y solo cuentan como conectados cuando el ángulo coincide con una estación ([[T13 Giratorias y mesas de transferencia]]).
- El **hueco** entre transportadores es una propiedad de la conexión (20 mm por defecto). En 3D es un hueco real que el pallet debe salvar.

## 3. El archivo de proyecto (`.intracraft.json`)

JSON legible y versionable con git. **Todos los valores guardados están en SI** (m, kg, s, rad); el editor convierte a mm y grados solo para mostrarlos.

```json
{
  "schemaVersion": 1,
  "name": "Source to Sink",
  "seed": 1,
  "layout": {
    "instances": [
      {
        "id": "conv1",
        "definitionId": "conveyor.roller",
        "definitionVersion": "0.1.0",
        "name": "Mesa de rodillos 1",
        "position": { "x": 0, "y": 0, "z": 0 },
        "yaw": 0,
        "params": { "length": 3.0, "nominalSpeed": 0.25, "drive": "zoned", "accumulationMode": "zpa" }
      }
    ],
    "connections": [
      { "from": { "instanceId": "src1", "portId": "out" }, "to": { "instanceId": "conv1", "portId": "A" }, "gap": 0.02 }
    ]
  }
}
```

Solo hace falta guardar los parámetros que difieren de los valores por defecto de la definición; los que faltan toman el valor por defecto de la versión de definición fijada.

## 4. Versiones y migraciones

- `schemaVersion` es un literal en el esquema: un archivo con otra versión se **rechaza**, nunca se lee en silencio.
- Cada cambio de esquema añade una función de migración `v(n) → v(n+1)`; al abrir se aplican en orden y después se valida.
- Las instancias fijan `definitionVersion` (versionado semántico). Actualizar la definición de un componente es una migración explícita, así un layout antiguo sigue comportándose igual.

## 5. Qué no se guarda en el proyecto

Los resultados de simulación (estadísticas, registros de eventos, resultados de referencia) **nunca** se guardan dentro del archivo de proyecto. Se exportan aparte (CSV/JSON) o se guardan en `test_data/` para las pruebas de regresión.

## Relacionado

[[A05 Librería de componentes]] · [[A13 Convenciones]] · [[A14 Referencia de API]]
