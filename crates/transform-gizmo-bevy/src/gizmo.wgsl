struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};


@vertex
fn vertex(vertex: VertexInput) -> VertexOutput {
    var position = vec4(vertex.position.x, -vertex.position.y, 0.5, 1.0);
    var color = vertex.color;

    return VertexOutput(position, color);
}

struct FragmentInput {
    @location(0) color: vec4<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
};

@fragment
fn fragment(in: FragmentInput) -> FragmentOutput {
    return FragmentOutput(in.color);
}
