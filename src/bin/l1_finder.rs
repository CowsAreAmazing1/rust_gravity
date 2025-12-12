use std::f32::consts::PI;

use main_gravity::prelude::*;
use nannou::math::{Vec2Angle, Vec2Rotate};
use plotters::prelude::*;

fn system_setup(pos_mul: f32, vel_mul: f32) -> System{
    let mut system = sun_planet_binary_ccw(100.0, 5.0);
    let planet = system.get_body(1).unwrap();
    
    let dust = Dust::new(pos_mul * planet.position(), vel_mul * planet.velocity());
    system.add_dust(dust);

    return system;
}

fn simulate_dust(pos_mul: f32, vel_mul: f32) -> Vec<(f32, f32)> {
    let mut system = system_setup(pos_mul, vel_mul);

    let mut dust_path = Vec::new();

    system.update_until(|sys| {
        let planet = sys.get_body(1).unwrap();
        let com = sys.center_of_mass();
        let sun_space = planet.position() - com;
        let angle = sun_space.angle();

        let dust_pos = sys.get_dust(0).unwrap().position();
        let dust_pos_rot =   (dust_pos - com).rotate(-angle);

        dust_path.push(dust_pos_rot);

        return dust_pos_rot.x < 0.0;
        // return false
    }, 0.1, 5, 100_000, None, None);

    return dust_path.iter().map(|p| (p.x, p.y)).collect();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const PATH_COUNT: usize = 100;

    let planet_orbit_radius = sun_planet_binary_ccw(100.0, 5.0)
        .get_body(1)
        .unwrap()
        .position()
        .length();

    let paths: [Vec<(f32, f32)>; PATH_COUNT] = std::array::from_fn(|n| {
        simulate_dust(0.7, 1.29 + 0.00002 * n as f32)
    });

    let circle_points: Vec<(f32, f32)> = (0..100)
        .map(|n| {
            let angle = n as f32 * PI / 50.0;
            (
                planet_orbit_radius * angle.cos(),
                planet_orbit_radius * angle.sin(),
            )
        })
        .collect();


    let root = BitMapBackend::new("./pics/output0.png", (6000, 6000)).into_drawing_area();
    root.fill(&WHITE)?;

    let limit = 1.5*planet_orbit_radius;
    let mut chart = ChartBuilder::on(&root)
        .margin(0)
        .build_cartesian_2d(-limit..limit, -limit..limit)?;

    chart.configure_mesh().draw()?;

    for path in paths.into_iter() {
        chart.draw_series(LineSeries::new(path, BLUE))?;
    }
    chart.draw_series(LineSeries::new(circle_points, &BLACK))?;


    root.present()?;

    Ok(())
}