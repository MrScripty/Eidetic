#import bevy_pbr::forward_io::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material_color: vec4<f32>;

@fragment
fn fragment(_mesh: VertexOutput) -> @location(0) vec4<f32> {
    return material_color;
}
