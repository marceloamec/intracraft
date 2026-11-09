//! A model as triangles, one primitive per material, and as a glTF 2.0 binary file (`.glb`): positions, normals, indices
//! and physically based materials, the parts of the format every 3D program reads. The file is the same for the same model,
//! byte for byte, so the files kept in the repository can be checked against their models.

use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::{Material, Model, Solid};

/// How many sides a cylinder is drawn with.
pub const CYLINDER_SIDES: usize = 24;

const GLB_MAGIC: u32 = 0x4654_6C67;
const GLB_VERSION: u32 = 2;
const CHUNK_JSON: u32 = 0x4E4F_534A;
const CHUNK_BIN: u32 = 0x004E_4942;
const ARRAY_BUFFER: u32 = 34962;
const ELEMENT_ARRAY_BUFFER: u32 = 34963;
const FLOAT: u32 = 5126;
const UNSIGNED_INT: u32 = 5125;

/// The triangles of one material.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Primitive {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

/// A model's triangles, by material.
pub fn primitives(model: &Model) -> BTreeMap<Material, Primitive> {
    let mut out: BTreeMap<Material, Primitive> = BTreeMap::new();
    for piece in &model.pieces {
        let p = out.entry(piece.material).or_default();
        match piece.solid {
            Solid::Box(size) => add_box(p, piece.centre, size),
            Solid::Cylinder { radius, height } => add_cylinder(p, piece.centre, radius, height),
        }
    }
    out
}

/// The model as a glTF 2.0 binary file.
pub fn write(model: &Model) -> Vec<u8> {
    let prims = primitives(model);
    let mut bin: Vec<u8> = Vec::new();
    let (mut views, mut accessors, mut materials, mut gltf_prims) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for (k, (material, p)) in prims.iter().enumerate() {
        let (lo, hi) = extent(&p.positions);
        let position = push_view(&mut bin, &mut views, floats(&p.positions), ARRAY_BUFFER);
        accessors.push(
            json!({"bufferView": position, "componentType": FLOAT, "count": p.positions.len(), "type": "VEC3", "min": lo, "max": hi}),
        );
        let normal = push_view(&mut bin, &mut views, floats(&p.normals), ARRAY_BUFFER);
        accessors.push(json!({"bufferView": normal, "componentType": FLOAT, "count": p.normals.len(), "type": "VEC3"}));
        let index_bytes: Vec<u8> = p.indices.iter().flat_map(|i| i.to_le_bytes()).collect();
        let index = push_view(&mut bin, &mut views, index_bytes, ELEMENT_ARRAY_BUFFER);
        accessors.push(json!({"bufferView": index, "componentType": UNSIGNED_INT, "count": p.indices.len(), "type": "SCALAR"}));
        gltf_prims.push(json!({"attributes": {"POSITION": 3 * k, "NORMAL": 3 * k + 1}, "indices": 3 * k + 2, "material": k}));
        materials.push(material_json(*material));
    }
    let doc = json!({
        "asset": {"version": "2.0", "generator": "IntraCraft intracraft-assets"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{"mesh": 0, "name": model.name}],
        "meshes": [{"name": model.name, "primitives": gltf_prims}],
        "materials": materials,
        "accessors": accessors,
        "bufferViews": views,
        "buffers": [{"byteLength": bin.len()}],
    });
    let mut text = doc.to_string().into_bytes();
    pad(&mut text, b' ');
    pad(&mut bin, 0);
    let total = 12 + 8 + text.len() + 8 + bin.len();
    let mut out = Vec::with_capacity(total);
    for word in [GLB_MAGIC, GLB_VERSION, total as u32, text.len() as u32, CHUNK_JSON] {
        out.extend_from_slice(&word.to_le_bytes());
    }
    out.extend_from_slice(&text);
    for word in [bin.len() as u32, CHUNK_BIN] {
        out.extend_from_slice(&word.to_le_bytes());
    }
    out.extend_from_slice(&bin);
    out
}

/// The JSON chunk of a `.glb` file, read back: its header checked, the document parsed.
pub fn read_json(glb: &[u8]) -> Result<Value, String> {
    let word = |at: usize| -> Result<u32, String> {
        glb.get(at..at + 4).and_then(|b| b.try_into().ok()).map(u32::from_le_bytes).ok_or_else(|| "the file is cut short".to_string())
    };
    if word(0)? != GLB_MAGIC || word(4)? != GLB_VERSION {
        return Err("not a glTF 2.0 binary file".into());
    }
    if word(8)? as usize != glb.len() {
        return Err("its length is not the one its header gives".into());
    }
    let length = word(12)? as usize;
    if word(16)? != CHUNK_JSON {
        return Err("its first chunk is not JSON".into());
    }
    let text = glb.get(20..20 + length).ok_or("the JSON chunk is cut short")?;
    serde_json::from_slice(text).map_err(|e| e.to_string())
}

fn material_json(m: Material) -> Value {
    let mut out = json!({
        "name": m.name(),
        "pbrMetallicRoughness": {"baseColorFactor": m.color(), "metallicFactor": m.metallic(), "roughnessFactor": m.roughness()},
    });
    if m.blends() {
        out["alphaMode"] = json!("BLEND");
        out["doubleSided"] = json!(true);
    }
    out
}

fn push_view(bin: &mut Vec<u8>, views: &mut Vec<Value>, bytes: Vec<u8>, target: u32) -> usize {
    pad(bin, 0);
    views.push(json!({"buffer": 0, "byteOffset": bin.len(), "byteLength": bytes.len(), "target": target}));
    bin.extend_from_slice(&bytes);
    views.len() - 1
}

fn floats(v: &[[f32; 3]]) -> Vec<u8> {
    v.iter().flatten().flat_map(|f| f.to_le_bytes()).collect()
}

fn pad(v: &mut Vec<u8>, with: u8) {
    while !v.len().is_multiple_of(4) {
        v.push(with);
    }
}

fn extent(points: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    points.iter().fold(([f32::MAX; 3], [f32::MIN; 3]), |(lo, hi), p| {
        (std::array::from_fn(|i| lo[i].min(p[i])), std::array::from_fn(|i| hi[i].max(p[i])))
    })
}

/// A box with flat faces: four corners and two triangles a face, wound outward.
fn add_box(p: &mut Primitive, c: [f64; 3], size: [f64; 3]) {
    let h = size.map(|s| s / 2.0);
    // Each face: its normal axis and sign; the corners go round counter-clockwise seen from outside.
    for (axis, sign) in [(0, 1.0), (0, -1.0), (1, 1.0), (1, -1.0), (2, 1.0), (2, -1.0)] {
        let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
        let corner = |a: f64, b: f64| {
            let mut q = c;
            q[axis] += sign * h[axis];
            q[u] += a * h[u];
            q[v] += b * h[v];
            q.map(|x| x as f32)
        };
        let mut normal = [0.0f32; 3];
        normal[axis] = sign as f32;
        let quad = if sign > 0.0 {
            [corner(-1.0, -1.0), corner(1.0, -1.0), corner(1.0, 1.0), corner(-1.0, 1.0)]
        } else {
            [corner(-1.0, -1.0), corner(-1.0, 1.0), corner(1.0, 1.0), corner(1.0, -1.0)]
        };
        add_quad(p, quad, [normal; 4]);
    }
}

/// A cylinder standing along y: its side with smooth normals, and a cap at each end.
fn add_cylinder(p: &mut Primitive, c: [f64; 3], radius: f64, height: f64) {
    let ring = |i: usize| {
        let a = std::f64::consts::TAU * i as f64 / CYLINDER_SIDES as f64;
        (a.cos(), a.sin())
    };
    let (y0, y1) = (c[1] - height / 2.0, c[1] + height / 2.0);
    let at = |(x, z): (f64, f64), y: f64| [(c[0] + radius * x) as f32, y as f32, (c[2] + radius * z) as f32];
    for i in 0..CYLINDER_SIDES {
        let (a, b) = (ring(i), ring(i + 1));
        let (na, nb) = ([a.0 as f32, 0.0, a.1 as f32], [b.0 as f32, 0.0, b.1 as f32]);
        // Seen from outside, going round toward −z first keeps the side counter-clockwise.
        add_quad(p, [at(a, y0), at(a, y1), at(b, y1), at(b, y0)], [na, na, nb, nb]);
    }
    for (y, up) in [(y1, 1.0f32), (y0, -1.0f32)] {
        let base = p.positions.len() as u32;
        p.positions.push([c[0] as f32, y as f32, c[2] as f32]);
        p.normals.push([0.0, up, 0.0]);
        for i in 0..=CYLINDER_SIDES {
            p.positions.push(at(ring(i), y));
            p.normals.push([0.0, up, 0.0]);
        }
        for i in 0..CYLINDER_SIDES as u32 {
            let (a, b) = (base + 1 + i, base + 2 + i);
            if up > 0.0 {
                p.indices.extend_from_slice(&[base, b, a]);
            } else {
                p.indices.extend_from_slice(&[base, a, b]);
            }
        }
    }
}

fn add_quad(p: &mut Primitive, corners: [[f32; 3]; 4], normals: [[f32; 3]; 4]) {
    let base = p.positions.len() as u32;
    p.positions.extend_from_slice(&corners);
    p.normals.extend_from_slice(&normals);
    p.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Piece;

    fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
    }

    fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
    }

    /// Every triangle turns its front (counter-clockwise) the way its normals point: outward.
    fn wound_outward(p: &Primitive) {
        for t in p.indices.chunks(3) {
            let [a, b, c] = [t[0], t[1], t[2]].map(|i| i as usize);
            let face = cross(sub(p.positions[b], p.positions[a]), sub(p.positions[c], p.positions[a]));
            let n = p.normals[a];
            assert!(face[0] * n[0] + face[1] * n[1] + face[2] * n[2] > 0.0, "a triangle faces inward: {t:?}");
        }
    }

    #[test]
    fn boxes_and_cylinders_are_wound_outward() {
        let m = Model {
            name: "test".into(),
            pieces: vec![
                Piece::cuboid(Material::Carton, [1.0, 2.0, 3.0], [0.4, 0.3, 0.2]),
                Piece::cylinder(Material::DrumBlue, [0.0, 0.5, 0.0], 0.3, 1.0),
            ],
        };
        let prims = primitives(&m);
        assert_eq!(prims[&Material::Carton].indices.len(), 36);
        assert_eq!(prims[&Material::DrumBlue].indices.len(), 3 * (2 * CYLINDER_SIDES + 2 * CYLINDER_SIDES));
        prims.values().for_each(wound_outward);
    }

    #[test]
    fn a_glb_file_reads_back_with_one_primitive_per_material_and_its_size() {
        let m = Model {
            name: "two".into(),
            pieces: vec![
                Piece::cuboid(Material::WoodLight, [0.0, 0.5, 0.0], [1.0, 1.0, 1.0]),
                Piece::cuboid(Material::Film, [0.0, 0.5, 0.0], [2.0, 1.0, 1.0]),
            ],
        };
        let glb = write(&m);
        assert_eq!(glb.len() % 4, 0);
        let doc = read_json(&glb).unwrap();
        assert_eq!(doc["meshes"][0]["primitives"].as_array().unwrap().len(), 2);
        assert_eq!(doc["materials"][1]["alphaMode"], "BLEND");
        assert_eq!(doc["accessors"][3]["max"][0], 1.0);
        assert_eq!(write(&m), glb, "the same model gives the same file");
        assert!(read_json(&glb[..glb.len() - 4]).is_err());
    }
}
