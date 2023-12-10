[[block]]
struct Params {
    zoom: f32;
    offsetX: f32;
    offsetY: f32;
    padding: f32; // Added padding for alignment
};

[[group(0), binding(0)]] var<uniform> params: Params;


[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] vertex_index : u32) -> [[builtin(position)]] vec4<f32> {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), // bottom left
        vec2<f32>( 1.0, -1.0), // bottom right
        vec2<f32>(-1.0,  1.0), // top left
        vec2<f32>(-1.0,  1.0), // top left
        vec2<f32>( 1.0, -1.0), // bottom right
        vec2<f32>( 1.0,  1.0)  // top right
    );
    let position = positions[vertex_index];
    return vec4<f32>(position, 0.0, 1.0);
}

[[stage(fragment)]]
fn fs_main([[builtin(position)]] frag_coord: vec4<f32>) -> [[location(0)]] vec4<f32> {
    let scaleX = params.zoom / 1600.0; 
    let scaleY = params.zoom / 1200.0;
    let xOffset = params.offsetX; 
    let yOffset = params.offsetY;
    let c = vec2<f32>(
        frag_coord.x * scaleX - 2.5 + xOffset, 
        frag_coord.y * scaleY - 1.0 + yOffset
    );

    var z = vec2<f32>(0.0, 0.0);
    var i = 0;
    for (; i < 1000; i = i + 1) {
        if (dot(z, z) > 4.0) {
            break;
        }
        z = vec2<f32>(
            z.x * z.x - z.y * z.y + c.x,
            2.0 * z.x * z.y + c.y
        );
    }

    if (i == 1000) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0); // Black color for points inside the Mandelbrot set
    }

    let normalized = f32(i) / 1000.0;
    let color = vec4<f32>(
        0.3 + 0.7 * cos(3.0 + normalized * 12.56),
        0.3 + 0.7 * sin(4.0 + normalized * 12.56),
        0.3 + 0.7 * cos(5.0 + normalized * 12.56),
        1.0
    );
    return color;
}

