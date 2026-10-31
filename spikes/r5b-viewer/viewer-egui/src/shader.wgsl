// Spike R5b, the BIMCraft-style viewer: instanced meshes lit by one sun and an ambient term. No shadows.

struct Globals {
  view_proj: mat4x4<f32>,
  sun: vec4<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;

struct VertexIn {
  @location(0) pos: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) color: vec3<f32>,
  // The instance: its rotation and place as four columns, its scale along its own axes, and its tint.
  @location(3) m0: vec4<f32>,
  @location(4) m1: vec4<f32>,
  @location(5) m2: vec4<f32>,
  @location(6) m3: vec4<f32>,
  @location(7) scale: vec4<f32>,
  @location(8) tint: vec4<f32>,
};

struct VertexOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) normal: vec3<f32>,
  @location(1) color: vec3<f32>,
};

@vertex
fn vs_main(v: VertexIn) -> VertexOut {
  let model = mat4x4<f32>(v.m0, v.m1, v.m2, v.m3);
  let world = model * vec4<f32>(v.pos * v.scale.xyz, 1.0);
  var out: VertexOut;
  out.clip = globals.view_proj * world;
  // The model matrix only turns and moves, and the scale is along the box's own axes, so it keeps a box's normals.
  out.normal = (model * vec4<f32>(v.normal, 0.0)).xyz;
  out.color = v.color * v.tint.rgb;
  return out;
}

@fragment
fn fs_main(f: VertexOut) -> @location(0) vec4<f32> {
  let n = normalize(f.normal);
  let light = 0.35 + 0.75 * max(dot(n, globals.sun.xyz), 0.0);
  return vec4<f32>(f.color * light, 1.0);
}
