//! The scene in Bevy: each object's parts as meshes in its paints, its moving parts under one entity the run moves, a
//! plate under it in the colour of its state; the hall's floor and light; and the pallets, each the pallet model of its
//! type with its load on it (the own glTF assets, ADR-0026).

use std::collections::HashMap;

use bevy::light::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use intracraft_assets::{LOAD_FILE_HEIGHT, LoadKind, load_file, pallet_file};
use intracraft_library::Library;
use intracraft_library::definition::Value;
use intracraft_library::layout::Layout;
use intracraft_library::params::instance_values;
use intracraft_scene::{Mover, Object, Paint, Part, Shape};

use crate::run::Picture;

/// The objects as drawn, for picking and names.
#[derive(Resource)]
pub struct Objects(pub Vec<Object>);

/// The last picture of the run.
#[derive(Resource, Default)]
pub struct Latest(pub Picture);

/// Whether objects show their state's colour.
#[derive(Resource)]
pub struct ShowStatus(pub bool);

/// The models of the pallets and loads, by file; and the load each pallet type carries (from the sources).
#[derive(Resource)]
pub struct PalletLook {
    models: HashMap<String, Handle<WorldAsset>>,
    loads: HashMap<String, Option<LoadKind>>,
    status: HashMap<&'static str, Handle<StandardMaterial>>,
}

#[derive(Component)]
pub struct Moving {
    id: String,
    mover: Mover,
}

#[derive(Component)]
pub struct StatusPlate(String);

#[derive(Component)]
pub struct PalletTag;

/// The pallets on show, by id.
#[derive(Resource, Default)]
pub struct OnShow(HashMap<u64, Entity>);

/// The states and their colours: working and detecting green, blocked red, stopped orange, waiting amber, a station
/// busy at its work blue; empty, idle and clear grey.
const STATES: [(&str, [f32; 3]); 13] = [
    ("working", [0.20, 0.75, 0.30]),
    ("detecting", [0.20, 0.90, 0.45]),
    ("blocked", [0.90, 0.20, 0.15]),
    ("stopped", [0.95, 0.55, 0.10]),
    ("waiting", [0.95, 0.75, 0.10]),
    ("busy", [0.20, 0.45, 0.95]),
    ("processing", [0.20, 0.45, 0.95]),
    ("lifting", [0.30, 0.60, 0.95]),
    ("lowering", [0.30, 0.60, 0.95]),
    ("turning", [0.55, 0.35, 0.90]),
    ("empty", [0.45, 0.47, 0.50]),
    ("idle", [0.45, 0.47, 0.50]),
    ("clear", [0.45, 0.47, 0.50]),
];

/// Builds the scene: floor, light, every object's parts and plate.
pub fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    objects: Res<Objects>,
) {
    let (lo, hi) = intracraft_scene::bounds(&objects.0).unwrap_or(([-5.0, 0.0, -5.0], [5.0, 2.0, 5.0]));
    let centre = Vec3::new(((lo[0] + hi[0]) / 2.0) as f32, 0.0, ((lo[2] + hi[2]) / 2.0) as f32);
    let size = ((hi[0] - lo[0]).max(hi[2] - lo[2]) as f32 + 20.0).max(30.0);
    let floor = materials.add(StandardMaterial { base_color: Color::srgb(0.62, 0.63, 0.62), perceptual_roughness: 0.9, ..default() });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(size, 0.02, size))),
        MeshMaterial3d(floor),
        Transform::from_translation(centre - Vec3::Y * 0.011),
    ));
    commands.spawn((
        DirectionalLight { illuminance: 9_000.0, shadow_maps_enabled: true, ..default() },
        Transform::from_translation(centre + Vec3::new(18.0, 30.0, 12.0)).looking_at(centre, Vec3::Y),
        CascadeShadowConfigBuilder { maximum_distance: size * 1.5, first_cascade_far_bound: 8.0, ..default() }.build(),
    ));

    let mut paints: HashMap<Paint, Handle<StandardMaterial>> = HashMap::new();
    let mut shapes: HashMap<String, Handle<Mesh>> = HashMap::new();
    let mut paint = |p: Paint, materials: &mut Assets<StandardMaterial>| {
        paints
            .entry(p)
            .or_insert_with(|| {
                let [r, g, b] = p.color();
                materials.add(StandardMaterial {
                    base_color: Color::srgb_u8(r, g, b),
                    metallic: p.metallic(),
                    perceptual_roughness: p.roughness(),
                    emissive: if p == Paint::Beam { LinearRgba::rgb(0.4, 2.0, 0.8) } else { LinearRgba::BLACK },
                    ..default()
                })
            })
            .clone()
    };
    for o in &objects.0 {
        let mut root = commands.spawn((Transform::default(), Visibility::default()));
        root.with_children(|c| {
            for part in &o.fixed {
                c.spawn(part_bundle(part, &mut shapes, &mut meshes, paint(part.paint, &mut materials)));
            }
            if !o.moving.is_empty() {
                c.spawn((Moving { id: o.id.clone(), mover: o.mover }, Transform::default(), Visibility::default())).with_children(|m| {
                    for part in &o.moving {
                        m.spawn(part_bundle(part, &mut shapes, &mut meshes, paint(part.paint, &mut materials)));
                    }
                });
            }
        });
        if let Some((a, b)) = o.bounds() {
            let (w, d) = ((b[0] - a[0]) as f32 + 0.1, (b[2] - a[2]) as f32 + 0.1);
            let at = Vec3::new(((a[0] + b[0]) / 2.0) as f32, (a[1] as f32).max(0.0) + 0.003, ((a[2] + b[2]) / 2.0) as f32);
            commands.spawn((
                StatusPlate(o.id.clone()),
                Mesh3d(meshes.add(Cuboid::new(w, 0.004, d))),
                Transform::from_translation(at),
                Visibility::Hidden,
            ));
        }
    }
}

/// The pallet and load models, loaded once, and the colours of the states.
pub fn load_looks(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    layout: Res<LayoutRes>,
) {
    let library = &layout.library;
    let mut models = HashMap::new();
    for t in &library.reference.pallet_types {
        let mut file = |name: String| {
            let handle = asset_server.load(GltfAssetLabel::Scene(0).from_asset(format!("{}/{name}", intracraft_assets::FOLDER)));
            models.insert(name, handle);
        };
        file(pallet_file(&t.id));
        for kind in LoadKind::ALL {
            file(load_file(&t.id, kind));
        }
    }
    let status = STATES
        .iter()
        .map(|(name, [r, g, b])| {
            let m = StandardMaterial { base_color: Color::srgb(*r, *g, *b), unlit: true, ..default() };
            (*name, materials.add(m))
        })
        .collect();
    commands.insert_resource(PalletLook { models, loads: loads_by_type(&layout.layout, library), status });
}

/// The layout and library the window shows.
#[derive(Resource)]
pub struct LayoutRes {
    pub layout: Layout,
    pub library: Library,
}

/// The load each pallet type carries: its first source's choice.
pub fn loads_by_type(layout: &Layout, library: &Library) -> HashMap<String, Option<LoadKind>> {
    let mut out = HashMap::new();
    for i in layout.instances.iter().filter(|i| i.definition_id == "flow.source") {
        let Some(d) = library.get(&i.definition_id) else { continue };
        let v = instance_values(d, &i.params);
        let text = |k: &str| v.get(k).and_then(Value::text).unwrap_or_default().to_string();
        out.entry(text("palletType")).or_insert_with(|| LoadKind::of_source(&text("loadKind"), &text("loadAsset")));
    }
    out
}

/// Turns turntable platforms and lifts decks to the run's angles and heights.
pub fn move_platforms(latest: Res<Latest>, mut movers: Query<(&Moving, &mut Transform)>) {
    for (m, mut t) in &mut movers {
        *t = match m.mover {
            Mover::Turn { pivot } => {
                let angle = latest.0.angles.get(&m.id).copied().unwrap_or(0.0) as f32;
                let pivot = Vec3::new(pivot[0] as f32, pivot[1] as f32, pivot[2] as f32);
                let turn = Quat::from_rotation_y(angle);
                Transform::from_translation(pivot - turn * pivot).with_rotation(turn)
            }
            Mover::Lift { rest } => {
                let height = latest.0.heights.get(&m.id).copied().unwrap_or(rest);
                Transform::from_translation(Vec3::Y * (height - rest) as f32)
            }
            Mover::None => Transform::default(),
        };
    }
}

/// Shows each object's state as the colour of its plate.
pub fn paint_states(
    latest: Res<Latest>,
    show: Res<ShowStatus>,
    look: Option<Res<PalletLook>>,
    mut commands: Commands,
    mut plates: Query<(Entity, &StatusPlate, &mut Visibility)>,
) {
    let Some(look) = look else { return };
    for (e, plate, mut visible) in &mut plates {
        let material = latest.0.status.get(plate.0.as_str()).and_then(|s| look.status.get(s));
        match (show.0, material) {
            (true, Some(m)) => {
                commands.entity(e).insert(MeshMaterial3d(m.clone()));
                *visible = Visibility::Visible;
            }
            _ => *visible = Visibility::Hidden,
        }
    }
}

/// Puts each pallet of the run where it is: new ones are made from their models, gone ones removed.
pub fn place_pallets(
    mut commands: Commands,
    latest: Res<Latest>,
    look: Option<Res<PalletLook>>,
    mut on_show: ResMut<OnShow>,
    mut placed: Query<&mut Transform, With<PalletTag>>,
) {
    let Some(look) = look else { return };
    let mut seen = Vec::with_capacity(latest.0.pallets.len());
    for p in &latest.0.pallets {
        seen.push(p.id);
        let pose = Transform::from_translation(p.centre.as_vec3()).with_rotation(p.rotation.as_quat());
        if let Some(t) = on_show.0.get(&p.id).and_then(|&e| placed.get_mut(e).ok()) {
            *t.into_inner() = pose;
            continue;
        }
        let pallet = look.models.get(&pallet_file(&p.type_id)).or_else(|| look.models.get(&pallet_file("eur"))).cloned();
        let load = look.loads.get(&p.type_id).copied().flatten().filter(|_| p.load_height > 0.0);
        let load = load.and_then(|k| look.models.get(&load_file(&p.type_id, k))).cloned();
        let mut e = commands.spawn((PalletTag, pose, Visibility::default()));
        e.with_children(|c| {
            if let Some(model) = pallet {
                c.spawn(WorldAssetRoot(model));
            }
            if let Some(model) = load {
                let stretch = (p.load_height / LOAD_FILE_HEIGHT) as f32;
                c.spawn((WorldAssetRoot(model), Transform::from_xyz(0.0, p.height as f32, 0.0).with_scale(Vec3::new(1.0, stretch, 1.0))));
            }
        });
        on_show.0.insert(p.id, e.id());
    }
    on_show.0.retain(|id, e| {
        let keep = seen.contains(id);
        if !keep {
            commands.entity(*e).despawn();
        }
        keep
    });
}

fn part_bundle(
    part: &Part,
    shapes: &mut HashMap<String, Handle<Mesh>>,
    meshes: &mut Assets<Mesh>,
    material: Handle<StandardMaterial>,
) -> (Mesh3d, MeshMaterial3d<StandardMaterial>, Transform) {
    let key = format!("{:?}", part.shape);
    let mesh = shapes
        .entry(key)
        .or_insert_with(|| {
            meshes.add(match part.shape {
                Shape::Box(s) => Mesh::from(Cuboid::new(s[0] as f32, s[1] as f32, s[2] as f32)),
                Shape::Roller { radius, length } => Cylinder::new(radius as f32, length as f32).mesh().resolution(20).build(),
                Shape::Post { radius, height } => Cylinder::new(radius as f32, height as f32).mesh().resolution(16).build(),
            })
        })
        .clone();
    // Bevy's cylinders stand along +y; a roller lies along its own z.
    let lay = if matches!(part.shape, Shape::Roller { .. }) { Quat::from_rotation_x(std::f32::consts::FRAC_PI_2) } else { Quat::IDENTITY };
    let turn = Quat::from_rotation_y(part.yaw as f32) * Quat::from_rotation_z(-part.tilt as f32) * lay;
    let at = Vec3::new(part.centre[0] as f32, part.centre[1] as f32, part.centre[2] as f32);
    (Mesh3d(mesh), MeshMaterial3d(material), Transform::from_translation(at).with_rotation(turn))
}
