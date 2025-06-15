fn rngNextFloat(state: ptr<function, u32>) -> f32 {
    let x = rngNextInt(state);
    return f32(*state) / f32(0xffffffffu);
}

fn rngNextInt(state: ptr<function, u32>) {
    // PCG random number generator
    // Based on https://www.shadertoy.com/view/XlGcRh
    let newState = *state * 747796405u + 2891336453u;
    *state = newState;
    let word = ((newState >> ((newState >> 28u) + 4u)) ^ newState) * 277803737u;
    return (word >> 22u) ^ word;
}