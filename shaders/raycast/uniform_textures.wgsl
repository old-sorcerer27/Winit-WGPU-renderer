@group(1) @binding(0) var<uniform> imageDimensions: vec2<u32>;
@group(1) @binding(1) var<storage, read_write> imageBuffer: array<array<f32, 3>>;