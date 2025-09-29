use nannou::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct State {
    pub u: Vec2,
    pub du: Vec2,
}

// Struct for both u and du in RK4
impl State {
    pub const ZERO: Self = Self { u: Vec2::ZERO, du: Vec2::ZERO };

    pub fn new(u: Vec2, du: Vec2) -> Self {
        State { u, du }
    }

    pub fn scale(&self, factor: f32) -> State {
        State {
             u: self.u  * factor,
            du: self.du * factor,
        }
    }

    pub fn combine(&self, other: &State, factor: f32) -> State {
        State {
             u: self.u  + other.u  * factor,
            du: self.du + other.du * factor,
        }
    }
    
    pub fn add(&self, other: &State) -> State {
        State {
            u: self.u  + other.u,
            du: self.du + other.du,
        }
    }

    /// Calculate gravitational acceleration on self due to another state with given mass. Only makes sense when u: position, du: velocity
    fn grav_acc(&self, other: &Box<dyn Body>) -> Vec2 {
        let r = other.position() - self.u;
        let distance_sq = r.length_squared();
        
        let force_magnitude = other.mass() / (distance_sq + 5.0);
        r.normalize() * force_magnitude
    }
}



pub trait Body {
    fn position(&self) -> Vec2;
    fn position_mut(&mut self) -> &mut Vec2;
    fn velocity(&self) -> Vec2;
    fn velocity_mut(&mut self) -> &mut Vec2;
    fn mass(&self) -> f32;
    fn color(&self) -> Hsv;
    
    // To state
    fn get_state(&self) -> State {
        State::new(self.position(), self.velocity())
    }
    
    // From state
    fn set_state(&mut self, state: State) {
        *self.position_mut() = state.u;
        *self.velocity_mut() = state.du;
    }
    
    fn update_euler(&mut self, dt: f32) {
        let velocity = self.velocity();
        *self.position_mut() += velocity * dt;
    }

    fn grav_acc(&self, other: &dyn Body) -> Vec2 {
        let r = other.position() - self.position();
        let distance_sq = r.length_squared();
        
        let force_magnitude = other.mass() / distance_sq;
        r.normalize() * force_magnitude
    }

    fn draw(&self, draw: &Draw) {
        draw.ellipse()
            .xy(self.position())
            .radius(self.mass().sqrt() * 0.5)
            .color(self.color());
    }
}


pub struct Attractor {
    pub position: Vec2,
    pub velocity: Vec2,
    pub mass: f32,
    pub color: Hsv,
}

impl Attractor {
    pub fn new(position: Vec2, velocity: Vec2, mass: f32, hue: f32) -> Self {
        Attractor {
            position,
            velocity,
            mass,
            color: Hsv::new(hue, 1.0, 1.0),
        }
    }
}

impl Body for Attractor {
    fn position(&self) -> Vec2 { self.position }
    fn position_mut(&mut self) -> &mut Vec2 { &mut self.position }
    fn velocity(&self) -> Vec2 { self.velocity }
    fn velocity_mut(&mut self) -> &mut Vec2 { &mut self.velocity }
    fn mass(&self) -> f32 { self.mass }
    fn color(&self) -> Hsv { self.color }
}


pub struct Dust {
    pub position: Vec2,
    pub velocity: Vec2,
    pub color: Hsv,
}

impl Dust {
    pub fn new(position: Vec2, velocity: Vec2, hue: f32) -> Self {
        Dust {
            position,
            velocity,
            color: Hsv::new(hue, 1.0, 1.0),
        }
    }
}

impl Body for Dust {
    fn position(&self) -> Vec2 { self.position }
    fn position_mut(&mut self) -> &mut Vec2 { &mut self.position }
    fn velocity(&self) -> Vec2 { self.velocity }
    fn velocity_mut(&mut self) -> &mut Vec2 { &mut self.velocity }
    fn mass(&self) -> f32 { panic!("Asked for the mass of a dust") }
    fn color(&self) -> Hsv { self.color }

    fn draw(&self, draw: &Draw) {
        draw.ellipse()
            .xy(self.position())
            .radius(2.0)
            .color(self.color());
    }
}











/// RK4 Integrator for gravitational systems
pub struct RK4Integrator;

impl RK4Integrator {
    /// Evaluate derivative at given state and time
    fn evaluate(
        bodies: &[Box<dyn Body>],
        body_index: usize,
        state: State,
        derivative: State,
        dt: f32,
    ) -> State { // Output a derivative state
        // Create temporary state
        let temp_state = state.combine(&State::new(derivative.u, derivative.du), dt);
        
        // Calculate acceleration due to all other bodies
        let mut acceleration = Vec2::ZERO;
        
        for (i, other_body) in bodies.iter().enumerate() {
            if i != body_index {
                acceleration += temp_state.grav_acc(other_body);
            }
        }
        
        State::new(temp_state.du, acceleration)
    }

    fn evaluate_dust(
        bodies: &[Box<dyn Body>],
        state: State,
        derivative: State,
        dt: f32,
    ) -> State { // Output a derivative state
        // Create temporary state
        let temp_state = state.combine(&State::new(derivative.u, derivative.du), dt);
        
        // Calculate acceleration due to all other bodies
        let mut acceleration = Vec2::ZERO;
        
        for other_body in bodies.iter() {
            acceleration += temp_state.grav_acc(other_body);
        }
        
        State::new(temp_state.du, acceleration)
    }
    
    pub fn integrate(
        bodies: &[Box<dyn Body>],
        body_index: usize,
        initial_state: State,
        dt: f32,
    ) -> State {
        // RK4 coefficients
        let k1 = Self::evaluate(bodies, body_index, initial_state, State::ZERO, 0.0);
        let k2 = Self::evaluate(bodies, body_index, initial_state, k1, dt * 0.5);
        let k3 = Self::evaluate(bodies, body_index, initial_state, k2, dt * 0.5);
        let k4 = Self::evaluate(bodies, body_index, initial_state, k3, dt);
        
        let final_state = k1
            .add(&k2.scale(2.0))
            .add(&k3.scale(2.0))
            .add(&k4)
            .scale(1.0 / 6.0);
        
        // Apply to state
        initial_state.combine(&final_state, dt)
    }

    pub fn integrate_dust(
        bodies: &[Box<dyn Body>],
        initial_state: State,
        dt: f32,
    ) -> State {
        // RK4 coefficients
        let k1 = Self::evaluate_dust(bodies, initial_state, State::ZERO, 0.0);
        let k2 = Self::evaluate_dust(bodies, initial_state, k1,          dt * 0.5);
        let k3 = Self::evaluate_dust(bodies, initial_state, k2,          dt * 0.5);
        let k4 = Self::evaluate_dust(bodies, initial_state, k3,          dt);
        
        let final_state = 
            k1
            .add(&k2.scale(2.0))
            .add(&k3.scale(2.0))
            .add(&k4)
            .scale(1.0 / 6.0);
        
        // Apply to state
        initial_state.combine(&final_state, dt)
    }
}









/// Physics system that manages bodies and integration
pub struct System {
    pub bodies: Vec<Box<dyn Body>>,
    pub dust: Vec<Dust>,
    pub use_rk4: bool, // True: RK4, False: Euler
}

impl System {
    pub fn new() -> Self {
        System {
            bodies: Vec::new(),
            dust: Vec::new(),
            use_rk4: true, // RK4 by default
        }
    }
    
    pub fn add_attractor<T: Body + 'static>(&mut self, body: T) {
        self.bodies.push(Box::new(body));
    }

    pub fn add_dust(&mut self, dust: Dust) {
        self.dust.push(dust);
    }
    
    pub fn set_integration_method(&mut self, use_rk4: bool) {
        self.use_rk4 = use_rk4;
    }
    
    // Main update
    pub fn update(&mut self, dt: f32) {
        if self.use_rk4 {
            self.update_rk4(dt);
        } else {
            self.update_euler(dt);
        }
    }
    
    fn update_rk4(&mut self, dt: f32) {
        // Initial system state as States
        let mut attractor_states: Vec<State> = self.bodies.iter()
            .map(|body| body.get_state())
            .collect();

        let mut dust_states: Vec<State> = self.dust.iter()
            .map(|body| body.get_state())
            .collect();
        


        // Substeps
        let sub_dt = dt / 5.0;
        for _ in 0..5 {
            attractor_states = attractor_states.iter().enumerate()
                .map(|(i, &state)| {
                    RK4Integrator::integrate(&self.bodies, i, state, sub_dt)
                })
                .collect();
    
            dust_states = dust_states.iter()
                .map(|&state| {
                    RK4Integrator::integrate_dust(&self.bodies, state, sub_dt)
                })
                .collect();
        }

        // Apply new states
        for (body, new_state) in self.bodies.iter_mut().zip(attractor_states.iter()) {
            body.set_state(*new_state);
        }

        for (body, new_state) in self.dust.iter_mut().zip(dust_states.iter()) {
            body.set_state(*new_state);
        }
    }
    
    /// Update using simple Euler integration (for comparison)
    fn update_euler(&mut self, dt: f32) {
        // Calculate forces
        let mut accelerations = vec![Vec2::ZERO; self.bodies.len()];
        
        for i in 0..self.bodies.len() {
            for j in 0..self.bodies.len() {
                if i != j {
                    let force = self.bodies[i].grav_acc(self.bodies[j].as_ref());
                    accelerations[i] += force / self.bodies[i].mass();
                }
            }
        }
        
        // Apply accelerations and update positions
        for (body, acceleration) in self.bodies.iter_mut().zip(accelerations.iter()) {
            let velocity = body.velocity();
            *body.velocity_mut() += *acceleration * dt;
            *body.position_mut() += velocity * dt;
        }
    }
    
    pub fn get_bodies(&self) -> &[Box<dyn Body>] {
        &self.bodies
    }
    pub fn get_dusts(&self) -> &[Dust] {
        &self.dust
    }


    pub fn draw(&self, draw: &Draw) {
        for body in self.get_bodies() {
            body.draw(&draw);
        }
        for dust in self.get_dusts() {
            dust.draw(&draw);
        }
    }
}