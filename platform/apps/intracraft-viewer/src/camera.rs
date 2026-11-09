//! The camera circles a point of the plant: drag with the left button to orbit, with the right or middle button to pan,
//! the wheel to come closer; F frames the whole plant. A click (a press and release without dragging) picks the object
//! under the pointer.

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::input::EguiWantsInput;

use crate::view::Objects;

/// Where the camera looks from: the point it circles, its turn and height angles (rad) and its distance (m).
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct Orbit {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
}

impl Orbit {
    /// A view of the box `(lo, hi)` from the south-east, from above.
    pub fn framing(lo: [f64; 3], hi: [f64; 3]) -> Orbit {
        let target = Vec3::new(((lo[0] + hi[0]) / 2.0) as f32, ((lo[1] + hi[1]) / 4.0) as f32, ((lo[2] + hi[2]) / 2.0) as f32);
        let span = ((hi[0] - lo[0]).max(hi[2] - lo[2]).max(hi[1] - lo[1]) as f32).max(3.0);
        Orbit { target, yaw: 0.75, pitch: 0.55, distance: span * 1.1 + 2.0 }
    }

    pub fn eye(&self) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        self.target + Vec3::new(cp * sy, sp, cp * cy) * self.distance
    }
}

/// How the whole plant is framed, for F.
#[derive(Resource, Clone, Copy)]
pub struct Home(pub Orbit);

/// The object picked, by its index in [`Objects`].
#[derive(Resource, Default)]
pub struct Picked(pub Option<usize>);

/// How far the pointer moved since the button went down, px: under a few it is a click.
#[derive(Resource, Default)]
pub struct Drag(f32);

pub fn spawn_camera(mut commands: Commands, home: Res<Home>) {
    commands.spawn((Camera3d::default(), Msaa::Sample4, Transform::from_translation(home.0.eye()).looking_at(home.0.target, Vec3::Y)));
    commands.insert_resource(home.0);
}

/// Orbits, pans and zooms the camera by the mouse; F goes home.
pub fn steer(
    mut orbit: ResMut<Orbit>,
    home: Res<Home>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    egui: Res<EguiWantsInput>,
    mut drag: ResMut<Drag>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
) {
    if keys.just_pressed(KeyCode::KeyF) && !egui.wants_keyboard_input() {
        *orbit = home.0;
    }
    let over_panel = egui.wants_pointer_input() || egui.is_pointer_over_area();
    if buttons.just_pressed(MouseButton::Left) || buttons.just_pressed(MouseButton::Right) {
        drag.0 = 0.0;
    }
    drag.0 += motion.delta.length();
    if !over_panel {
        if buttons.pressed(MouseButton::Left) {
            orbit.yaw -= motion.delta.x * 0.006;
            orbit.pitch = (orbit.pitch + motion.delta.y * 0.006).clamp(0.05, 1.5);
        } else if buttons.pressed(MouseButton::Right) || buttons.pressed(MouseButton::Middle) {
            let (sy, cy) = orbit.yaw.sin_cos();
            let (right, forward) = (Vec3::new(cy, 0.0, -sy), Vec3::new(-sy, 0.0, -cy));
            let k = orbit.distance * 0.0015;
            orbit.target += (-right * motion.delta.x + forward * motion.delta.y) * k;
        }
        let lines = match scroll.unit {
            MouseScrollUnit::Line => scroll.delta.y,
            MouseScrollUnit::Pixel => scroll.delta.y / 40.0,
        };
        orbit.distance = (orbit.distance * 0.88f32.powf(lines)).clamp(1.0, 500.0);
    }
    for mut t in &mut camera {
        *t = Transform::from_translation(orbit.eye()).looking_at(orbit.target, Vec3::Y);
    }
}

/// Picks the nearest object whose box the pointer's ray meets, on a click outside the panel.
pub fn pick(
    buttons: Res<ButtonInput<MouseButton>>,
    egui: Res<EguiWantsInput>,
    drag: Res<Drag>,
    objects: Res<Objects>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut picked: ResMut<Picked>,
) {
    if !buttons.just_released(MouseButton::Left) || drag.0 > 4.0 || egui.wants_pointer_input() || egui.is_pointer_over_area() {
        return;
    }
    let (Ok(window), Ok((camera, at))) = (window.single(), camera.single()) else { return };
    let Some(ray) = window.cursor_position().and_then(|p| camera.viewport_to_world(at, p).ok()) else { return };
    picked.0 = objects
        .0
        .iter()
        .enumerate()
        .filter_map(|(k, o)| {
            let (lo, hi) = o.bounds()?;
            hit_box(ray.origin, *ray.direction, lo.map(|v| v as f32), hi.map(|v| v as f32)).map(|d| (k, d))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(k, _)| k);
}

/// Where a ray from `origin` along `dir` first meets the box `(lo, hi)`, if it does: its distance.
pub fn hit_box(origin: Vec3, dir: Vec3, lo: [f32; 3], hi: [f32; 3]) -> Option<f32> {
    let (mut near, mut far) = (0.0f32, f32::INFINITY);
    for i in 0..3 {
        let (o, d) = (origin[i], dir[i]);
        if d.abs() < 1e-9 {
            if o < lo[i] || o > hi[i] {
                return None;
            }
            continue;
        }
        let (a, b) = ((lo[i] - o) / d, (hi[i] - o) / d);
        near = near.max(a.min(b));
        far = far.min(a.max(b));
    }
    (near <= far).then_some(near)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ray_meets_a_box_in_front_and_misses_one_beside() {
        let d = hit_box(Vec3::new(0.0, 0.5, -5.0), Vec3::Z, [-1.0, 0.0, -1.0], [1.0, 1.0, 1.0]);
        assert!((d.unwrap_or(0.0) - 4.0).abs() < 1e-6);
        assert_eq!(hit_box(Vec3::new(3.0, 0.5, -5.0), Vec3::Z, [-1.0, 0.0, -1.0], [1.0, 1.0, 1.0]), None);
    }

    #[test]
    fn the_home_view_sees_the_plant_from_above_the_south_east() {
        let o = Orbit::framing([0.0, 0.0, -2.0], [10.0, 2.0, 2.0]);
        let eye = o.eye();
        assert!(eye.y > 0.0 && eye.x > o.target.x && eye.z > o.target.z, "{eye:?}");
    }
}
