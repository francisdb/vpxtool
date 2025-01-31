#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var<uniform> color_start: vec4<f32>;
@group(2) @binding(1) var<uniform> color_end: vec4<f32>;

// For the definition of VertexOutput,
// see https://github.com/bevyengine/bevy/blob/main/crates/bevy_sprite/src/mesh2d/mesh2d_vertex_output.wgsl

@fragment
fn fragment(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    let t = in.uv.y;
    return mix(color_start, color_end, t);
}

