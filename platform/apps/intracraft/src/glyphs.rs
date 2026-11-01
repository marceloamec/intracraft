//! IntraCraft's ribbon icons: our own outlines with BIMCraft's single 1.5 pt pen, painted in the square and the colour the
//! ribbon gives (`bimcraft_ui_egui::ribbon_button`).

use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, vec2};

const PEN: f32 = 1.5;

/// What a glyph draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glyph {
    Rollers,
    Chains,
    Turntable,
    Transfer,
    Lift,
    Elevator,
    Gravity,
    Process,
    Source,
    Sink,
    Photocell,
    Scenery,
    Run,
    Pause,
    Step,
    Reset,
    Detach,
    /// A speed, as its factor.
    Speed(u32),
}

impl Glyph {
    /// The glyph of a library definition.
    pub fn of(definition: &str) -> Glyph {
        match definition {
            "conveyor.roller" => Glyph::Rollers,
            "conveyor.chain" => Glyph::Chains,
            "conveyor.roller_turntable" | "conveyor.chain_turntable" => Glyph::Turntable,
            "conveyor.transfer_rc" => Glyph::Transfer,
            "conveyor.lift_table_manual" | "conveyor.scissor_lift" => Glyph::Lift,
            "conveyor.elevator" => Glyph::Elevator,
            "conveyor.gravity" => Glyph::Gravity,
            "flow.process_station" => Glyph::Process,
            "flow.source" => Glyph::Source,
            "flow.sink" => Glyph::Sink,
            "sensor.photoeye" => Glyph::Photocell,
            _ => Glyph::Scenery,
        }
    }

    pub fn paint(self, p: &Painter, r: Rect, ink: Color32) {
        let s = Stroke::new(PEN, ink);
        // The square's points on a 24 × 24 grid.
        let at = |x: f32, y: f32| r.min + vec2(x, y) * (r.width() / 24.0);
        let line = |a: Pos2, b: Pos2| p.line_segment([a, b], s);
        let frame = |x0: f32, y0: f32, x1: f32, y1: f32| {
            p.rect_stroke(Rect::from_two_pos(at(x0, y0), at(x1, y1)), 1.0, s, egui::StrokeKind::Middle)
        };
        let circle = |x: f32, y: f32, radius: f32| p.circle_stroke(at(x, y), radius * r.width() / 24.0, s);
        let poly = |pts: &[(f32, f32)]| {
            p.add(Shape::closed_line(pts.iter().map(|(x, y)| at(*x, *y)).collect(), s));
        };
        match self {
            Glyph::Rollers => {
                frame(2.0, 8.0, 22.0, 16.0);
                for x in [6.0, 10.0, 14.0, 18.0] {
                    circle(x, 12.0, 1.6);
                }
            }
            Glyph::Chains => {
                frame(2.0, 8.0, 22.0, 16.0);
                line(at(4.0, 10.5), at(20.0, 10.5));
                line(at(4.0, 13.5), at(20.0, 13.5));
            }
            Glyph::Turntable => {
                circle(12.0, 12.0, 9.0);
                frame(6.0, 8.0, 18.0, 16.0);
                line(at(12.0, 3.0), at(15.0, 4.5));
            }
            Glyph::Transfer => {
                frame(3.0, 3.0, 21.0, 21.0);
                line(at(12.0, 6.0), at(12.0, 18.0));
                line(at(6.0, 12.0), at(18.0, 12.0));
                poly(&[(18.0, 10.0), (20.5, 12.0), (18.0, 14.0)]);
            }
            Glyph::Lift => {
                frame(4.0, 4.0, 20.0, 8.0);
                line(at(6.0, 20.0), at(18.0, 8.0));
                line(at(6.0, 8.0), at(18.0, 20.0));
                line(at(3.0, 20.0), at(21.0, 20.0));
            }
            Glyph::Elevator => {
                line(at(5.0, 2.0), at(5.0, 22.0));
                line(at(19.0, 2.0), at(19.0, 22.0));
                frame(7.0, 10.0, 17.0, 14.0);
                poly(&[(12.0, 3.0), (9.5, 6.5), (14.5, 6.5)]);
            }
            Glyph::Gravity => {
                line(at(2.0, 8.0), at(22.0, 16.0));
                for x in [6.0, 11.0, 16.0] {
                    circle(x, 8.0 + (x - 2.0) * 0.4 + 2.0, 1.6);
                }
                line(at(21.0, 12.0), at(21.0, 18.0));
            }
            Glyph::Process => {
                frame(3.0, 15.0, 21.0, 19.0);
                line(at(6.0, 15.0), at(6.0, 4.0));
                line(at(18.0, 15.0), at(18.0, 4.0));
                line(at(6.0, 4.0), at(18.0, 4.0));
                frame(10.0, 6.0, 14.0, 10.0);
            }
            Glyph::Source | Glyph::Sink => {
                frame(3.0, 7.0, 13.0, 17.0);
                let (tail, tip) = if self == Glyph::Source { (13.0, 21.0) } else { (21.0, 13.5) };
                line(at(tail, 12.0), at(tip, 12.0));
                let back = if self == Glyph::Source { tip - 3.0 } else { tip + 3.0 };
                poly(&[(tip, 12.0), (back, 9.5), (back, 14.5)]);
            }
            Glyph::Photocell => {
                frame(2.0, 9.0, 6.0, 15.0);
                frame(18.0, 9.0, 22.0, 15.0);
                p.line_segment([at(6.0, 12.0), at(18.0, 12.0)], Stroke::new(PEN, ink.gamma_multiply(0.6)));
            }
            Glyph::Scenery => {
                poly(&[(4.0, 8.0), (12.0, 4.0), (20.0, 8.0), (20.0, 17.0), (12.0, 21.0), (4.0, 17.0)]);
                line(at(4.0, 8.0), at(12.0, 12.0));
                line(at(20.0, 8.0), at(12.0, 12.0));
                line(at(12.0, 12.0), at(12.0, 21.0));
            }
            Glyph::Run => poly(&[(7.0, 4.0), (19.0, 12.0), (7.0, 20.0)]),
            Glyph::Pause => {
                frame(6.0, 4.0, 10.0, 20.0);
                frame(14.0, 4.0, 18.0, 20.0);
            }
            Glyph::Step => {
                poly(&[(5.0, 5.0), (15.0, 12.0), (5.0, 19.0)]);
                line(at(18.0, 5.0), at(18.0, 19.0));
            }
            Glyph::Reset => {
                let c = at(12.0, 12.0);
                let radius = 7.5 * r.width() / 24.0;
                let arc: Vec<Pos2> = (0..=20).map(|k| 0.6 + k as f32 * 0.25).map(|a: f32| c + vec2(a.cos(), -a.sin()) * radius).collect();
                p.add(Shape::line(arc, s));
                let tip = c + vec2(0.6f32.cos(), -0.6f32.sin()) * radius;
                p.add(Shape::closed_line(vec![tip, tip + vec2(-3.5, -2.0), tip + vec2(0.5, -4.0)], s));
            }
            Glyph::Detach => {
                frame(3.0, 3.0, 11.0, 11.0);
                frame(13.0, 13.0, 21.0, 21.0);
                line(at(11.0, 13.0), at(13.0, 11.0));
            }
            // A dial whose needle leans further the faster: ×1 to the left, ×10 000 to the right.
            Glyph::Speed(n) => {
                let c = at(12.0, 16.0);
                let radius = 9.0 * r.width() / 24.0;
                let arc: Vec<Pos2> =
                    (0..=16).map(|k| std::f32::consts::PI * k as f32 / 16.0).map(|a| c + vec2(-a.cos(), -a.sin()) * radius).collect();
                p.add(Shape::line(arc, s));
                let share = (n.max(1) as f32).log10() / 4.0;
                let a = std::f32::consts::PI * (0.1 + 0.8 * share.min(1.0));
                line(c, c + vec2(-a.cos(), -a.sin()) * radius * 0.85);
                p.circle_filled(c, 1.5, ink);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_library_definition_has_a_glyph_of_its_own() {
        let lib = intracraft_library::Library::built_in().unwrap();
        let named = lib.list().iter().filter(|d| Glyph::of(&d.id) != Glyph::Scenery).count();
        assert_eq!(named, lib.list().len() - 1, "only the scenery object falls back");
    }
}
