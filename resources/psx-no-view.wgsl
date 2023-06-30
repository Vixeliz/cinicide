struct CameraUniform {
    view_proj: mat4x4<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) color: vec4<f32>,
}


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) @interpolate(linear) tex_coord: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) vertex_color: vec4<f32>,
    @location(3) fog: f32,
}


@group(1) @binding(0)
var<uniform> camera: CameraUniform;


@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    var out: VertexOutput;
    // let in_clip = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    let in_clip = model_matrix * vec4<f32>(model.position, 1.0);
    let snap_scale = 100.0;
    var position = vec4(
        in_clip.x  / in_clip.w,
        in_clip.y  / in_clip.w,
        in_clip.z  / in_clip.w,
        in_clip.w
    );
    position = vec4(
        floor(in_clip.x * snap_scale) / snap_scale,
        floor(in_clip.y * snap_scale) / snap_scale,
        in_clip.z,
        in_clip.w
    );

    let fog_distance = vec2<f32>(25.0, 75.0);
    let depth_vert = camera.view_proj * vec4(position);
    let depth = abs(depth_vert.z / depth_vert.w);
    out.clip_position = position;
    out.tex_coord = model.tex_coords;
    out.fog = 1.0 - clamp((fog_distance.y - depth) / (fog_distance.y - fog_distance.x), 0.0, 1.0);
    out.color = instance.color;
    out.vertex_color = model.color;

    return out;
}

@group(0) @binding(0)
var t_color: texture_2d<f32>;

@group(0) @binding(1)
var s_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(t_color, s_sampler, in.tex_coord);
    if tex.a < 0.5 {
        discard;
    }
    let fir_col = in.vertex_color * tex;
    let fog_color = vec3<f32>(0.0, 0.0, 0.0);
    let col = vec4(mix(fir_col.rgb, fog_color, in.fog), 1.0);
    return col;
}
