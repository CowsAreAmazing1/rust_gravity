struct Particle {
    pos: vec2<f32>,
    vel: vec2<f32>,
};
@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;

@group(1) @binding(0) var<storage, read_write> colors: array<f32>;

struct Attractor {
    pos: vec2<f32>,
    mass: f32,
    _padding: f32,
}
@group(2) @binding(0) var<storage, read> attractors: array<Attractor>;

struct DispatchParams {
    offset: u32,
    dt: f32,
    // min_speed: f32,
    // max_speed: f32,
    // frame: u32,
};
@group(3) @binding(0) var<uniform> params: DispatchParams;

fn calculate_acceleration(pos: vec2<f32>) -> vec2<f32> {
    let num_attractors = arrayLength(&attractors);
    var acc = vec2<f32>(0.0, 0.0);

    for (var a = 0u; a < num_attractors; a++) {
        let attractor = attractors[a];
        let rel_pos = attractor.pos - pos;
        let distance_sq = dot(rel_pos, rel_pos);

        // Add softening parameter to avoid singularities
        let softening = 1e-6;
        let distance = sqrt(distance_sq + softening);
        let distance_cubed = distance * distance * distance;

        acc += attractor.mass * rel_pos / distance_cubed;
    }

    return acc;
}

fn rk4_step(pos: vec2<f32>, vel: vec2<f32>, dt: f32) -> Particle {
    // k1 = f(t, y)
    let k1_vel = vel;
    let k1_acc = calculate_acceleration(pos);

    // k2 = f(t + dt/2, y + k1*dt/2)
    let pos_k2 = pos + k1_vel * (dt * 0.5);
    let vel_k2 = vel + k1_acc * (dt * 0.5);
    let k2_vel = vel_k2;
    let k2_acc = calculate_acceleration(pos_k2);

    // k3 = f(t + dt/2, y + k2*dt/2)
    let pos_k3 = pos + k2_vel * (dt * 0.5);
    let vel_k3 = vel + k2_acc * (dt * 0.5);
    let k3_vel = vel_k3;
    let k3_acc = calculate_acceleration(pos_k3);

    // k4 = f(t + dt, y + k3*dt)
    let pos_k4 = pos + k3_vel * dt;
    let vel_k4 = vel + k3_acc * dt;
    let k4_vel = vel_k4;
    let k4_acc = calculate_acceleration(pos_k4);

    let final_pos = pos + (dt / 6.0) * (k1_vel + 2.0 * k2_vel + 2.0 * k3_vel + k4_vel);
    let final_vel = vel + (dt / 6.0) * (k1_acc + 2.0 * k2_acc + 2.0 * k3_acc + k4_acc);

    var result: Particle;
    result.pos = final_pos;
    result.vel = final_vel;
    return result;
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let num_particles = arrayLength(&particles);
    let i = id.x + params.offset;
    if i >= num_particles {
        return;
    }

    let p = particles[i];
    let dt = params.dt;

    let integrated = rk4_step(p.pos, p.vel, dt);

    particles[i] = integrated;

    // let color_value = length(calculate_acceleration(p.pos));
    let color_value = length(p.vel);
    colors[i] = color_value;
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
