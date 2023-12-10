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
    let scaleX = 3.5 / 800.0; // Adjust these values to scale the set
    let scaleY = 2.0 / 600.0; // Adjust these values to scale the set
    let xOffset = -0.75; // Shifts the set left or right
    let yOffset = 0.0; // Shifts the set up or down
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
    let color = vec4<f32>(f32(i) / 500.0, 0.0, 0.0, 1.0);
    return color;
}
