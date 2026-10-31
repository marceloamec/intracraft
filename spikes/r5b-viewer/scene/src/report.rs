//! The plain-language report of spike R5b: both viewers' results side by side and what they say, in Spanish for the
//! owner (identifiers stay English). The decision rule is ADR-0019's: if the BIMCraft-style view holds 60 frames per
//! second, the product stays without a game engine; otherwise the Bevy viewer is adopted.

use crate::bench::BenchResult;

/// Frames per second the BIMCraft-style view must hold on average.
pub const TARGET_FPS: f64 = 60.0;
/// And the slowest 5 % of its frames.
pub const TARGET_LOW_FPS: f64 = 45.0;

/// What the measurements say.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// The BIMCraft-style view is fast enough: no game engine.
    OwnView,
    /// Only Bevy is fast enough: adopt the Bevy viewer.
    Bevy,
    /// Neither is: the scene or the viewers must be looked at again.
    Neither,
    /// A result is missing or was drawn without a real graphics card.
    Invalid,
}

pub fn verdict(egui: Option<&BenchResult>, bevy: Option<&BenchResult>) -> Verdict {
    let (Some(e), Some(b)) = (egui, bevy) else { return Verdict::Invalid };
    if is_software(&e.adapter) || is_software(&b.adapter) || e.frames.frames == 0 || b.frames.frames == 0 {
        return Verdict::Invalid;
    }
    let fast = |r: &BenchResult| r.frames.mean_fps >= TARGET_FPS && r.frames.low_5pct_fps >= TARGET_LOW_FPS;
    if fast(e) {
        Verdict::OwnView
    } else if fast(b) {
        Verdict::Bevy
    } else {
        Verdict::Neither
    }
}

/// Whether the adapter is the processor pretending to be a card (Windows' WARP, Mesa's llvmpipe or lavapipe).
pub fn is_software(adapter: &str) -> bool {
    let a = adapter.to_lowercase();
    ["basic render", "warp", "llvmpipe", "lavapipe", "softpipe", "swiftshader"].iter().any(|s| a.contains(s))
}

/// The report, as text.
pub fn render(egui: Option<&BenchResult>, bevy: Option<&BenchResult>) -> String {
    let mut out = String::new();
    out.push_str("PRUEBA R5b - VISOR 3D DE LA SIMULACION\n");
    out.push_str("=======================================\n\n");
    out.push_str(
        "Escena: 20 transportadores de rodillos, 2000 rodillos girando y 200 pallets con carga en movimiento.\n",
    );
    out.push_str("Cada visor dibujo la misma escena durante el mismo tiempo, con la camara dando una vuelta.\n\n");
    for (name, r) in [("Vista propia tipo BIMCraft (egui + wgpu)", egui), ("Visor Bevy", bevy)] {
        out.push_str(&format!("{name}\n"));
        match r {
            None => out.push_str("  No hay resultado (el visor no termino o no se abrio).\n\n"),
            Some(r) => {
                out.push_str(&format!("  Tarjeta grafica:              {} ({})\n", r.adapter, r.backend));
                out.push_str(&format!("  Imagenes por segundo (media): {:.0}\n", r.frames.mean_fps));
                out.push_str(&format!(
                    "  El 5 % mas lento, al menos:   {:.0} imagenes por segundo\n",
                    r.frames.low_5pct_fps
                ));
                out.push_str(&format!("  Imagen mas lenta:             {:.0} ms\n", r.frames.max_frame_ms));
                out.push_str(&format!("  Arranque hasta ver la escena: {:.1} s\n", r.startup_ms / 1000.0));
                out.push_str(&format!(
                    "  Modelo del pallet:            {} ({} triangulos, leido en {:.0} ms)\n",
                    if r.pallet_model == "boxes" {
                        "cajas (sin modelo del catalogo)"
                    } else {
                        "modelo GLB del catalogo"
                    },
                    r.pallet_triangles,
                    r.glb_load_ms
                ));
                out.push('\n');
            }
        }
    }
    out.push_str("RESULTADO\n---------\n");
    out.push_str(match verdict(egui, bevy) {
        Verdict::OwnView => {
            "La vista propia tipo BIMCraft llega a 60 imagenes por segundo: no hace falta un motor de juegos.\n"
        }
        Verdict::Bevy => "Solo Bevy llega a 60 imagenes por segundo: se recomienda adoptar el visor Bevy.\n",
        Verdict::Neither => {
            "Ninguno de los dos llega a 60 imagenes por segundo: hay que revisar la escena o los visores.\n"
        }
        Verdict::Invalid => {
            "La medicion no vale: falta un resultado o se dibujo sin tarjeta grafica real. Repetir en el portatil.\n"
        }
    });
    out.push_str("\nCopie este texto en el chat para escribir la decision (ADR-0022).\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench::FrameStats;

    fn result(adapter: &str, mean: f64, low: f64) -> BenchResult {
        BenchResult {
            adapter: adapter.into(),
            frames: FrameStats { frames: 1000, mean_fps: mean, low_5pct_fps: low, ..FrameStats::default() },
            ..BenchResult::default()
        }
    }

    #[test]
    fn a_fast_own_view_needs_no_engine() {
        let (e, b) = (result("Intel Iris Xe", 75.0, 55.0), result("Intel Iris Xe", 140.0, 110.0));
        assert_eq!(verdict(Some(&e), Some(&b)), Verdict::OwnView);
    }

    #[test]
    fn a_slow_own_view_picks_bevy() {
        let (e, b) = (result("Intel Iris Xe", 70.0, 30.0), result("Intel Iris Xe", 90.0, 60.0));
        assert_eq!(verdict(Some(&e), Some(&b)), Verdict::Bevy);
    }

    #[test]
    fn both_slow_is_neither() {
        let (e, b) = (result("Intel UHD", 30.0, 20.0), result("Intel UHD", 40.0, 25.0));
        assert_eq!(verdict(Some(&e), Some(&b)), Verdict::Neither);
    }

    #[test]
    fn a_software_adapter_or_a_missing_result_is_invalid() {
        let real = result("NVIDIA GeForce RTX 3050", 200.0, 150.0);
        assert_eq!(
            verdict(Some(&result("Microsoft Basic Render Driver", 200.0, 150.0)), Some(&real)),
            Verdict::Invalid
        );
        assert_eq!(verdict(Some(&real), None), Verdict::Invalid);
        assert_eq!(verdict(Some(&real), Some(&result("llvmpipe (LLVM 18)", 9.0, 5.0))), Verdict::Invalid);
    }

    #[test]
    fn the_report_names_both_viewers_and_the_result() {
        let r = result("Intel Iris Xe", 75.0, 55.0);
        let text = render(Some(&r), None);
        assert!(text.contains("Vista propia") && text.contains("Visor Bevy") && text.contains("No hay resultado"));
        assert!(text.contains("La medicion no vale"));
    }
}
