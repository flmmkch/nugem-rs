// Vertex shader: sprite position and texture


struct VertexInput {
    @location(0) position: vec2<u32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// global surface size storage variable
@group(0) @binding(0)
var<storage, read> surface_size: vec2<u32>;

@vertex
fn vs_main(
    input: VertexInput,
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    var position_2d: vec2<f32>;
    let surface_size_f32 = vec2<f32>(f32(surface_size.x), f32(surface_size.y)); 
    position_2d.x = 2.0 * f32(input.position.x) / surface_size_f32.x - 1.0;
    position_2d.y = 2.0 * f32(input.position.y) / surface_size_f32.y - 1.0;
    out.clip_position = vec4<f32>(position_2d, 0.0, 1.0);
    out.tex_coords = input.tex_coords;
    return out;
}

// Fragment shader: texture

@group(1) @binding(0)
var sprite_texture: texture_2d<f32>;
@group(1) @binding(1)
var sprite_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(sprite_texture, sprite_sampler, in.tex_coords);
}
