# brand/

Identidad visual de IntraCraft. Estado: **logo elegido (A, rail), aplicado en la aplicación** (ADR-0030, 10-11-2026).

- `guidelines.md`: esencia, tono, reglas de uso.
- `tokens/`: paleta (`colors.json`) y variables CSS (`brand.css`).
- `logo/variants/`: las tres propuestas (A rail, **elegida**; B flow; C roller).
- `export/`: el logo A en PNG (64, 256, 512 y 1024 px) e `intracraft.ico` (icono del programa en Windows). Se generan
  desde el dibujo del logo en `platform/apps/intracraft/src/brand.rs` con
  `cargo run -p intracraft -- --write-brand` (dentro de `platform/`); una prueba avisa si quedan desactualizados.
  No se editan a mano.
- `visuals/`, `scripts/`: pendientes.

En la aplicación, la marca aparece en la página de inicio (logo, nombre y lema), en el icono de la ventana y en el
del `.exe`. Los colores de la interfaz siguen siendo los de BIMCraft (decisión de la responsable).
