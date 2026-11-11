//! Meshes the viewer draws: a unit box, a roller and, from a catalogue GLB, the pallet. What it takes to read a GLB
//! here (the `gltf` crate, node transforms, a colour per material) is part of what spike R5b measures.

use std::path::Path;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 3],
}

#[derive(Clone, Debug, Default)]
pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl MeshData {
    pub fn triangles(&self) -> usize {
        self.indices.len() / 3
    }

    fn quad(&mut self, corners: [Vec3; 4], normal: Vec3, color: [f32; 3]) {
        let base = self.vertices.len() as u32;
        for c in corners {
            self.vertices.push(Vertex { pos: c.to_array(), normal: normal.to_array(), color });
        }
        self.indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

/// A box of side 1 centred on the origin, white; instances scale and tint it.
pub fn unit_box() -> MeshData {
    let mut m = MeshData::default();
    let h = 0.5;
    for axis in 0..3 {
        for sign in [-1.0_f32, 1.0] {
            let mut n = Vec3::ZERO;
            n[axis] = sign;
            let (u, v) = {
                let mut u = Vec3::ZERO;
                let mut v = Vec3::ZERO;
                u[(axis + 1) % 3] = 1.0;
                v[(axis + 2) % 3] = 1.0;
                (u, v)
            };
            let c = n * h;
            let (u, v) = if sign > 0.0 { (u * h, v * h) } else { (v * h, u * h) };
            m.quad([c - u - v, c + u - v, c + u + v, c - u + v], n, [1.0; 3]);
        }
    }
    m
}

/// A roller: a cylinder of `radius` along +Z, `length` long and centred, with `sides` facets.
pub fn roller(radius: f32, length: f32, sides: u32) -> MeshData {
    let mut m = MeshData::default();
    let color = [0.78, 0.8, 0.82];
    let half = length / 2.0;
    let at = |i: u32| {
        let a = std::f32::consts::TAU * i as f32 / sides as f32;
        Vec3::new(a.cos(), a.sin(), 0.0)
    };
    for i in 0..sides {
        let (p, q) = (at(i), at(i + 1));
        let base = m.vertices.len() as u32;
        for (dir, z) in [(p, -half), (q, -half), (q, half), (p, half)] {
            m.vertices.push(Vertex { pos: (dir * radius + Vec3::Z * z).to_array(), normal: dir.to_array(), color });
        }
        m.indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        for (z, n) in [(half, 1.0_f32), (-half, -1.0)] {
            let base = m.vertices.len() as u32;
            let normal = [0.0, 0.0, n];
            for v in [Vec3::new(0.0, 0.0, z), p * radius + Vec3::Z * z, q * radius + Vec3::Z * z] {
                m.vertices.push(Vertex { pos: v.to_array(), normal, color });
            }
            if n > 0.0 {
                m.indices.extend([base, base + 1, base + 2]);
            } else {
                m.indices.extend([base, base + 2, base + 1]);
            }
        }
    }
    m
}

/// Every triangle of a GLB's first scene, in the scene's axes, each coloured by its material: the base colour
/// times the mean of its base colour picture (the viewer has no textures).
pub fn load_glb(path: &Path) -> Result<MeshData, String> {
    let (doc, buffers, images) = gltf::import(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let scene = doc.default_scene().or_else(|| doc.scenes().next()).ok_or("the file has no scene")?;
    let mut out = MeshData::default();
    let mut stack: Vec<(gltf::Node, Mat4)> = scene.nodes().map(|n| (n, Mat4::IDENTITY)).collect();
    while let Some((node, parent)) = stack.pop() {
        let world = parent * Mat4::from_cols_array_2d(&node.transform().matrix());
        if let Some(mesh) = node.mesh() {
            for prim in mesh.primitives() {
                if prim.mode() != gltf::mesh::Mode::Triangles {
                    continue;
                }
                let reader = prim.reader(|b| buffers.get(b.index()).map(|d| &d.0[..]));
                let Some(positions) = reader.read_positions() else { continue };
                let positions: Vec<Vec3> = positions.map(|p| world.transform_point3(Vec3::from(p))).collect();
                let normals: Option<Vec<Vec3>> = reader
                    .read_normals()
                    .map(|ns| ns.map(|n| world.transform_vector3(Vec3::from(n)).normalize_or_zero()).collect());
                let indices: Vec<u32> = match reader.read_indices() {
                    Some(i) => i.into_u32().collect(),
                    None => (0..positions.len() as u32).collect(),
                };
                let color = material_color(&prim.material(), &images);
                let base = out.vertices.len() as u32;
                for (i, p) in positions.iter().enumerate() {
                    let normal = normals.as_ref().and_then(|n| n.get(i).copied()).unwrap_or(Vec3::Y);
                    out.vertices.push(Vertex { pos: p.to_array(), normal: normal.to_array(), color });
                }
                out.indices.extend(indices.iter().filter(|&&i| (i as usize) < positions.len()).map(|i| base + i));
            }
        }
        for child in node.children() {
            stack.push((child, world));
        }
    }
    out.indices.truncate(out.indices.len() / 3 * 3);
    if out.indices.is_empty() {
        return Err("the file has no triangles".into());
    }
    Ok(out)
}

fn material_color(material: &gltf::Material, images: &[gltf::image::Data]) -> [f32; 3] {
    let pbr = material.pbr_metallic_roughness();
    let f = pbr.base_color_factor();
    let mean = pbr.base_color_texture().and_then(|t| images.get(t.texture().source().index())).map(mean_color);
    let m = mean.unwrap_or([1.0; 3]);
    [f[0] * m[0], f[1] * m[1], f[2] * m[2]]
}

/// The mean colour of a picture, linear 0–1.
fn mean_color(image: &gltf::image::Data) -> [f32; 3] {
    let channels = match image.format {
        gltf::image::Format::R8G8B8 => 3,
        gltf::image::Format::R8G8B8A8 => 4,
        _ => return [1.0; 3],
    };
    let mut sum = [0.0_f64; 3];
    let pixels = image.pixels.chunks_exact(channels);
    let n = pixels.len().max(1) as f64;
    for px in pixels {
        for c in 0..3 {
            sum[c] += srgb_to_linear(px[c]);
        }
    }
    [(sum[0] / n) as f32, (sum[1] / n) as f32, (sum[2] / n) as f32]
}

fn srgb_to_linear(c: u8) -> f64 {
    let c = c as f64 / 255.0;
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_unit_box_is_closed_and_faces_out() {
        let m = unit_box();
        assert_eq!(m.triangles(), 12);
        for t in m.indices.chunks_exact(3) {
            let [a, b, c] = [t[0], t[1], t[2]].map(|i| Vec3::from(m.vertices[i as usize].pos));
            let n = Vec3::from(m.vertices[t[0] as usize].normal);
            assert!((b - a).cross(c - a).dot(n) > 0.0, "a face winds inward");
            assert!(a.abs().max_element() <= 0.5 + 1e-6);
        }
    }

    #[test]
    fn the_roller_has_its_size() {
        let m = roller(0.0445, 1.0, 16);
        assert_eq!(m.triangles(), 16 * 4);
        let max_z = m.vertices.iter().map(|v| v.pos[2]).fold(f32::MIN, f32::max);
        assert!((max_z - 0.5).abs() < 1e-6);
    }

    #[test]
    fn a_missing_glb_is_an_error_not_a_crash() {
        assert!(load_glb(Path::new("no-such-file.glb")).is_err());
    }

    #[test]
    fn mean_color_reads_srgb() {
        let white =
            gltf::image::Data { pixels: vec![255; 12], format: gltf::image::Format::R8G8B8, width: 2, height: 2 };
        assert_eq!(mean_color(&white), [1.0; 3]);
    }
}

/// The catalogue pallet, when `R5B_GLB` names it (CI downloads it; it is not in git).
#[cfg(test)]
mod catalogue {
    use super::*;

    #[test]
    fn the_catalogue_pallet_reads_at_its_size() {
        let Some(path) = std::env::var_os("R5B_GLB").filter(|p| Path::new(p).exists()) else { return };
        let m = load_glb(Path::new(&path)).map_err(|e| e.to_string());
        let m = m.as_ref().map_err(|e| e.as_str());
        assert!(m.is_ok(), "{m:?}");
        let Ok(m) = m else { return };
        let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        for v in &m.vertices {
            lo = lo.min(Vec3::from(v.pos));
            hi = hi.max(Vec3::from(v.pos));
        }
        let size = hi - lo;
        // The catalogue says 1.204 x 0.209 x 0.8 m.
        assert!(
            (size.x - 1.204).abs() < 0.01 && (size.y - 0.209).abs() < 0.01 && (size.z - 0.8).abs() < 0.01,
            "{size}"
        );
        assert!(m.triangles() > 12);
    }
}
