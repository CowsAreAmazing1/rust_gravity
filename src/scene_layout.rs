use nannou::{geom::Range, prelude::*};

use crate::Dust;



// Main trait implemented by all scene builder elements. Allows filling with Dust particles.
pub trait FillWithDust {
    fn build(&self, num: u32, target_vec: &mut Vec<Dust>);
}


pub struct Setup {
    elements: Vec<Box<dyn FillWithDust>>,
}

impl Setup {
    // Dust scene setup builder
    pub fn new() -> Self {
        Setup {
            elements: vec![],
        }
    }

    // Add an element to be filled with dust to the scene
    pub fn add<T: FillWithDust + 'static>(&mut self, element: T) -> &mut Self {
        self.elements.push(Box::new(element));
        self
    }

    // Build the scene
    pub fn build(&self, total_num_particles: u32, target: &mut Vec<Dust>) {
        let num = total_num_particles / self.elements.len() as u32;
        for element in &self.elements {
            element.build(num, target);
        }
    }
}




enum CreationOperation {
    CenterOffset(Vec2),
    VelocityOffset(Vec2),
    // RadialVelocity(f32),
    VelocityScale(f32),
    Orbit(Vec2, f32, bool), // (orbit center, mass, clockwise?)
}



// =========== DISC ===========
pub struct Disc {
    inner_radius: f32,
    outer_radius: f32,
    start_angle: f32,
    end_angle: f32,
    ops: Vec<CreationOperation>,
}

impl Disc {
    pub fn new() -> Self {
        Disc {
            inner_radius: 0.0,
            outer_radius: 1.0,
            start_angle: 0.0,
            end_angle: TAU,
            ops: vec![],
        }
    }

    pub fn start_angle(mut self, angle: f32) -> Self {
        self.start_angle = angle;
        self
    }
    pub fn end_angle(mut self, angle: f32) -> Self {
        self.end_angle = angle;
        self
    }
    pub fn inner_radius(mut self, radius: f32) -> Self {
        self.inner_radius = radius;
        self
    }
    pub fn outer_radius(mut self, radius: f32) -> Self {
        self.outer_radius = radius;
        self
    }
    pub fn center_position(mut self, center: Vec2) -> Self {
        self.ops.push(CreationOperation::CenterOffset(center));
        self
    }
    pub fn center_velocity(mut self, velocity: Vec2) -> Self {
        self.ops.push(CreationOperation::VelocityOffset(velocity));
        self
    }
    pub fn speed_scale(mut self, scale: f32) -> Self {
        self.ops.push(CreationOperation::VelocityScale(scale));
        self
    }
    pub fn orbit(mut self, center: Vec2, mass: f32, clockwise: bool) -> Self {
        self.ops.push(CreationOperation::Orbit(center, mass, clockwise));
        self
    }
}

impl FillWithDust for Disc {
    fn build(&self, num: u32, target: &mut Vec<Dust>) {
        let Self {
            inner_radius: inner,
            outer_radius: outer,
            start_angle: start,
            end_angle: end,
            ops,
        } = self;

        let phi = (1.0 + 5_f64.sqrt()) / 2.0;
        let i_2 = (*inner) as f64 * (*inner) as f64;
        let o_2 = (*outer) as f64 * (*outer) as f64;
        let delta_radius = o_2 - i_2;
        let delta_angle = *end as f64 - *start as f64;

        for i in 0..num {
            let r_2 = i_2 + delta_radius * (i as f64 / num as f64);
            let r = r_2.sqrt();
            
            let angle = *start as f64 + delta_angle * (i as f64 / phi).fract();
            
            let mut pos = vec2((r * angle.cos()) as f32, (r * angle.sin()) as f32);
            let mut vel = Vec2::ZERO;

            for op in ops {
                match op {
                    CreationOperation::CenterOffset(v) => { pos += *v; },
                    CreationOperation::VelocityOffset(v) => { vel += *v; },
                    CreationOperation::VelocityScale(s) => { vel *= *s; },
                    CreationOperation::Orbit(center, mass, clockwise) => {
                        let rel_pos = *center - pos;
                        let dist = rel_pos.length();
                        let angle = rel_pos.angle();
                        let speed = (mass / dist).sqrt();
                        let sign = if *clockwise { -1.0 } else { 1.0 };
                        vel += sign * speed * vec2(angle.sin() as f32, -angle.cos() as f32)
                    }
                }
            }
            target.push(Dust::new(pos, vel));
        };
    }
}



// ========== QUAD ===========
pub struct Quad {
    rect: Rect,
    ops: Vec<CreationOperation>,
}

impl Quad {
    pub fn new() -> Self {
        Quad {
            rect: Rect::from_wh(Vec2::ONE),
            ops: vec![],
        }
    }

    pub fn square(mut self, size: f32) -> Self {
        self.rect = Rect::from_w_h(size, size);
        self
    }
    pub fn width(mut self, width: f32) -> Self {
        self.rect.x = Range::new(-0.5*width, 0.5*width);
        self
    }
    pub fn height(mut self, height: f32) -> Self {
        self.rect.y = Range::new(-0.5*height, 0.5*height);
        self
    }
    pub fn center_position(mut self, center: Vec2) -> Self {
        self.ops.push(CreationOperation::CenterOffset(center));
        self
    }
    pub fn center_velocity(mut self, velocity: Vec2) -> Self {
        self.ops.push(CreationOperation::VelocityOffset(velocity));
        self
    }
    pub fn speed_scale(mut self, scale: f32) -> Self {
        self.ops.push(CreationOperation::VelocityScale(scale));
        self
    }
    pub fn orbit(mut self, center: Vec2, mass: f32, clockwise: bool) -> Self {
        self.ops.push(CreationOperation::Orbit(center, mass, clockwise));
        self
    }
}

impl FillWithDust for Quad {
    fn build(&self, num: u32, target: &mut Vec<Dust>) {
        let Self { rect, ops } = self;
        let (w, h) = rect.w_h();

        // Initial random distribution
        let dx = 1.0 / num as f64;
        let inv_phi = 2.0 / (1.0 + 5_f64.sqrt());
        let points = (0..num).map(|n| {
            // voronator::delaunator::Point {
            //     x: random_range(-0.5*w, 0.5*w) as f64,
            //     y: random_range(-0.5*h, 0.5*h) as f64,
            // }
            voronator::delaunator::Point {
                x: w as f64 * (n as f64 * dx - 0.5),
                y: h as f64 * ((n as f64 * inv_phi).fract() - 0.5),
            }
        }).collect::<Vec<voronator::delaunator::Point>>();

        // Single pass of Lloyd's algorithm to improve distribution
        let points = voronator::CentroidDiagram::<voronator::delaunator::Point>::new(&points).unwrap().centers;

        for point in points {
            let mut pos = vec2(point.x as f32, point.y as f32);
            let mut vel = Vec2::ZERO;

            for op in ops {
                match op {
                    CreationOperation::CenterOffset(v) => { pos += *v; },
                    CreationOperation::VelocityOffset(v) => { vel += *v; },
                    CreationOperation::VelocityScale(s) => { vel *= *s; },
                    CreationOperation::Orbit(center, mass, clockwise) => {
                        let rel_pos = *center - pos;
                        let dist = rel_pos.length();
                        let angle = rel_pos.angle();
                        let speed = (mass / dist).sqrt();
                        let sign = if *clockwise { -1.0 } else { 1.0 };
                        vel += sign * speed * vec2(angle.sin() as f32, -angle.cos() as f32)
                    }
                }
            }
            target.push(Dust::new(pos, vel));
        };
    }
}






















    // LineBuilder
    // pub fn center_vec(mut self, center: Vec2) -> Self {
    //     self.center = center;
    //     self
    // }

    // pub fn center_xy(mut self, x: f32, y: f32) -> Self {
    //     self.center = vec2(x, y);
    //     self
    // }

    // pub fn velocity_vec(mut self, velocity: Vec2) -> Self {
    //     self.velocity += velocity;
    //     self
    // }

    // pub fn velocity_xy(mut self, x: f32, y: f32) -> Self {
    //     self.velocity += vec2(x, y);
    //     self
    // }

    // pub fn horizontal(mut self) -> Self {
    //     self.horizontal = true;
    //     self
    // }

    // pub fn vertical(mut self) -> Self {
    //     self.horizontal = false;
    //     self
    // }

    // pub fn build(self) -> Vec<Particle> {
    //     (0..self.num_particles).map(|i| {
    //         let t = i as f64 / (self.num_particles - 1) as f64;

    //         let pos = if self.horizontal {
    //             [(self.center.x as f64 - self.length as f64 * 0.5 + t * self.length as f64) as f32, self.center.y]
    //         } else {
    //             [self.center.x, (self.center.y as f64 - self.length as f64 * 0.5 + t * self.length as f64) as f32]
    //         };

    //         let vel = self.velocity.into();

    //         Particle { pos, vel }
    //     }).collect::<Vec<_>>()
    // }







    // RingBuilder
    // pub fn center_vec(mut self, center: Vec2) -> Self{
    //     self.center = center;
    //     self
    // }

    // pub fn center_xy(mut self, x: f32, y: f32) -> Self{
    //     self.center = vec2(x,y);
    //     self
    // }

    // pub fn velocity_vec(mut self, velocity: Vec2) -> Self{
    //     self.velocity += velocity;
    //     self
    // }

    // pub fn velocity_xy(mut self, x: f32, y: f32) -> Self{
    //     self.velocity += vec2(x,y);
    //     self
    // }

    // pub fn orbit(mut self, orbit: bool) -> Self {
    //     self.orbit = orbit;
    //     self
    // }

    // pub fn velocity_scale(mut self, scale: f32) -> Self {
    //     self.velocity_scale = scale;
    //     self
    // }

    // pub fn build(self) -> Vec<Particle> {
    //     (0..self.num_particles).map(|i| {
    //         let angle = f64::TAU() / self.num_particles as f64 * i as f64;
    //         let pos = [(self.radius as f64 * angle.cos() + self.center.x as f64) as f32, (self.radius as f64 * angle.sin() + self.center.y as f64) as f32];
    //         if self.orbit {
    //             let speed = (SOLAR_MASS as f64 * G as f64 / self.radius as f64).sqrt() * self.velocity_scale as f64;
    //             Particle {
    //                 pos,
    //                 vel: [(self.velocity.x as f64 + speed * angle.sin()) as f32, (self.velocity.y as f64 - speed * angle.cos()) as f32],
    //             }
    //         } else {
    //             Particle {
    //                 pos,
    //                 vel: [self.velocity.x, self.velocity.y],
    //             }
    //         }
    //     }).collect::<Vec<_>>()
    // }








// pub fn compute_l1_point(_g: f64, M: f64, m: f64, R: f64) -> f64 {
//     // let omega2 = g * (M + m) / (R * R * R);
    
//     let f = |x: f64| -> f64 {
//         if x <= 0.0 || x >= R {
//             return f64::INFINITY;
//         }
//         -M / (x * x) + m / ((R - x).powi(2)) + M * m / (x * x)
//     };

//     // Try to find a valid bracket where f(x) crosses zero
//     let mut a = R * 0.001;
//     let mut b = R * 0.999;

//     let mut fa = f(a);
//     let mut fb = f(b);

//     // If initial guess doesn't cross zero, scan for a bracket
//     if fa * fb > 0.0 {
//         for i in 1..1000 {
//             let t = i as f64 / 1000.0;
//             let x1 = t * R;
//             let x2 = (t + 0.001) * R;
//             let f1 = f(x1);
//             let f2 = f(x2);
//             if f1.is_finite() && f2.is_finite() && f1 * f2 < 0.0 {
//                 a = x1;
//                 b = x2;
//                 fa = f1;
//                 fb = f2;
//                 break;
//             }
//         }
//         if fa * fb > 0.0 {
//             panic!("Could not find a valid bracket for root finding.");
//         }
//     }

//     // Brent-style bisection
//     for _ in 0..100 {
//         let m = 0.5 * (a + b);
//         let fm = f(m);

//         if fm.abs() < 1e-10 || (b - a).abs() < 1e-8 {
//             return m;
//         }

//         if fa * fm < 0.0 {
//             b = m;
//             fb = fm;
//         } else {
//             a = m;
//             fa = fm;
//         }
//     }

//     panic!("Root finding did not converge.");
// }


