// Vertex shader

@group(0) @binding(0)
var<uniform> camera: mat4x4<f32>;

@group(0) @binding(1)
var<uniform> model_transform: mat4x4<f32>;

@group(0) @binding(2)
var<storage, read> faces: array<u32>;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) primitive_id: u32,
}

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = camera * model_transform * vec4<f32>(model.position, 1.0);
    out.primitive_id = u32(floor(f32(model.vertex_index) / 4.0));
    return out;
}

// Fragment shader

@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    let err = vec4(0.5, 0.5, 0.5, 1.0);
    let red = vec4(1.0, 0.0, 0.0, 1.0);
    switch faces[in.primitive_id] {
        case 0u, 1u: {
            return red * 0.3;
        }
        case 2u, 3u: {
            return red * 0.5;
        }
        case 4u, 5u: {
            return red * 0.75;
        }
        default: {
            return err;
        }
    }
}

