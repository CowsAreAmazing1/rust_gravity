use core::f32;

use main_gravity::{Body, Attractor, Dust, System};
use nannou::{glam::{vec2, Vec2}, math::{Vec2Angle, Vec2Rotate}};



fn sun_planet_sys() -> System {
    let mut system = System::new();

    let mut sun = Attractor::new(Vec2::ZERO, Vec2::ZERO, 100.0, 0.0);
    let mut planet = Attractor::new(vec2(200.0, 0.0), Vec2::ZERO, 5.0, 100.0);
    sun.orbit_pair(&mut planet, false);

    system.add_attractor(sun);
    system.add_attractor(planet);

    system
}


fn main() {
    let mut data = Vec::new();
    for x in 0..50 {
        for i in 0..50 {
            let mut system = sun_planet_sys();
        
            let planet = system.get_body(1).unwrap();
            
            let interp = x as f32 / 50.0;
            let dust_pos = interp * planet.position();
            let dust_vel = i as f32 / 50.0;
            let dust = Dust::new(dust_pos, vec2(0.0, dust_vel));
            system.add_dust(dust);
        
            let mut ypos = f32::INFINITY;
            let rpos = {
                let mut rpos = Vec2::ZERO;
                
                while ypos > 0.0 {
                    system.update(0.1, 10, None, None);
                    
                    let angle = (system.get_body(1).unwrap().position() - system.center_of_mass()).angle();
                    rpos = (system.get_dust(0).unwrap().position() - system.center_of_mass()).rotate(-angle);
                    ypos = rpos.y;
                }
                rpos
            };
            data.push([rpos.x, dust_vel]);

        }
    }

    data.iter().for_each(|d| {
        print!("({}, {})", d[0],d[1]);
        print!(",")
    });
}