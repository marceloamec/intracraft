# 13 Objetos obsoletos o prescindibles

Criterio: IntraCraft es un simulador de **pallets sobre rodillos y cadenas**, con un solo layout y dos modos. Lo que sigue es mi valoración [C] de la referencia comercial; marca tú lo que discrepes.

## No copiar (no aportan o chocan con el diseño)

| Objeto / función de la referencia               | Motivo                                                                                        | Alternativa en IntraCraft                        |
| ----------------------------------------------- | --------------------------------------------------------------------------------------------- | ------------------------------------------------ |
| Orden de conexión como prioridad oculta         | Cambia el resultado sin verse; el propio tutorial sugiere «borrar y reconectar» [PDF]         | Prioridad explícita y editable de sucesores      |
| Conector sin geometría                          | En pallets importa el punto exacto de entrega                                                 | Puertos con posición y orientación               |
| Interfaces obligatorias entre Frames            | Complejidad de jerarquía; el propio programa ya las hace opcionales a un nivel [PDF]          | Subesquemas con puertos expuestos                |
| Frame «Glass Box» vs «Model Frame»              | Dos formas de lo mismo; confunde                                                              | Un solo tipo de grupo con opción «expandir»      |
| Scripts de propósito general en cada objeto     | Rompe el determinismo y las pruebas, difícil de depurar para quien no programa                | Reglas Si/Entonces + plantillas de control       |
| Animación como entidad aparte del modelo        | El 3D del programa es cosmético; aquí el 3D es física                                         | Un solo layout con dos motores                   |
| Intercambio de gráfico por modelo 3D comercial  | Licencias de modelos 3D de fabricante                                                         | Geometría paramétrica propia                     |
| Objeto «Ciclo»                                  | Un bucle de MU no tiene sentido para un flujo lineal con topes                                | Un bucle real = conexiones circulares            |
| Estación genérica que «procesa tiempo»          | En manutención de pallets no hay procesamiento; sí transporte, acumulación y giro             | Objetos específicos del MVP                      |
| Reloj 3D (EventController visible en el modelo) | Es un objeto físico dentro del plano, sin sentido físico                                      | Barra de control en la UI                        |

## Existen, pero fuera del MVP (útiles más adelante)

| Objeto                                              | Cuándo tendría sentido                                       |
| --------------------------------------------------- | ------------------------------------------------------------ |
| Fallos y turnos                                     | Estudios de disponibilidad                                   |
| Transporter (AGV, carretilla) y Tracks              | Cuando entren vehículos                                      |
| Recursos humanos (operarios, rutas)                 | Estaciones manuales                                          |
| Gestor de experimentos y optimización               | Barridos paramétricos con CLI                                |
| Almacenes y robots (pick-and-place)                 | Si se amplía a almacén                                       |
| Elevadores, curvas, cajas, cubetas, energía, OPC UA | Según AGENTS.md: fuera hasta nuevo aviso                     |

## Fuera de alcance por naturaleza

- **Objetos de fluidos** (tuberías, depósitos, mezcladores) y de **cinta continua**: no son transporte de unidades.
- **Add-ins** de dominios específicos (carpintería, ensamblaje de coches…): no aplican.
- **Programación SimTalk completa**: IntraCraft no pretende un lenguaje de script.

## Lo que sí falta en la referencia y IntraCraft debe tener

| Necesidad                                                | Por qué                                                         |
| -------------------------------------------------------- | --------------------------------------------------------------- |
| Física de contacto real                                  | Rozamiento, topes, guías, alineación del pallet                 |
| Sensores con geometría real                              | Rayo, alcance, altura                                           |
| Determinismo garantizado con semilla                     | Comparabilidad entre ejecuciones y entre modos 2D/3D            |
| Pruebas analíticas de referencia                         | T1–T5 de AGENTS.md                                              |
| Acumulación sin presión por zonas                        | Rodillos de acumulación son el caso central                     |
| Mesas giratorias y de transferencia como objetos nativos | En la referencia hay que improvisarlas con estaciones           |
