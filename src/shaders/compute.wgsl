struct Particle {
    pos: vec2<f32>,
    vel: vec2<f32>,
};
@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;

struct Attractor {
    pos: vec2<f32>,
    mass: f32,
    _padding: f32,
}
@group(1) @binding(0) var<storage, read> attractors: array<Attractor>;

struct DispatchParams {
    offset: u32,
    dt: f32,
    // min_speed: f32,
    // max_speed: f32,
    // frame: u32,
};
@group(2) @binding(0) var<uniform> params: DispatchParams;


@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let num_particles = arrayLength(&particles);
    let i = id.x + params.offset;
    if (i >= num_particles) {
        return;
    }

    var num_attractors = arrayLength(&attractors);
    var p = particles[i];

    var dt = params.dt;
    
    for (var a = 0u; a < num_attractors; a++) {
        var attractor = attractors[a];
        var rel_pos = attractor.pos - p.pos;
        var distance = length(rel_pos);

        var acc = attractor.mass * rel_pos / (distance * distance * distance);
        p.vel = p.vel + acc * dt;
    }

    p.pos = p.pos + p.vel * dt;
    
    particles[i] = p;
}

fn improved_hash(seed: u32) -> u32 {
    var h = seed;
    h ^= h >> 16u;
    h *= 0x85ebca6bu;
    h ^= h >> 13u;
    h *= 0xc2b2ae35u;
    h ^= h >> 16u;
    return h;
}

fn random(seed: u32) -> f32 {
    let hashed = improved_hash(seed);
    return f32(hashed & 0x00FFFFFFu) / f32(0x01000000u); // Normalize to [0.0, 1.0)
}