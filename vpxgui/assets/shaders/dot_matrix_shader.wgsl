// Port of the LED Dot Matrix Effect for Three.js by @felixturner
// https://www.airtightinteractive.com/demos/js/ledeffect/

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

// TODO why are these resolution and dimension uniforms not passed correctly?
// eg 1280, 320
@group(2) @binding(0) var<uniform> resolution2: vec2<f32>;
// eg 128, 32
@group(2) @binding(1) var<uniform> dimension2: vec2<f32>;
@group(2) @binding(2) var color_texture: texture_2d<f32>;
@group(2) @binding(3) var color_sampler: sampler;

// For the definition of VertexOutput,
// see https://github.com/bevyengine/bevy/blob/main/crates/bevy_sprite/src/mesh2d/mesh2d_vertex_output.wgsl

@fragment
fn fragment(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    let size = 4.0;
    let blur = 10.0;
    let resolution = vec2(1280.0, 320.0);
    let dimension = vec2(128.0, 32.0);
    let count = dimension;
    let spacing = vec2(resolution/count);
    let p = floor(in.uv*count)/count;
    let color: vec4<f32> = textureSample(color_texture, color_sampler, p);
    let pos: vec2<f32> = (resolution * in.uv) % vec2(spacing) - vec2(spacing/2.0);
    let dist_squared: f32 = dot(pos, pos);
    let dmd_color = mix(color, vec4(0.0), smoothstep(size, size + blur, dist_squared));
    return dmd_color;
}
