use std::f32::consts::PI;

use main_gravity::prelude::*;
use nannou::glam::Vec2;
use plotters::prelude::*;

fn system_setup(pos_mul: f32, vel_mul: f32) -> System<VV> {
    let mut system = sun_planet_binary_ccw(100.0, 5.0);
    let planet = system.get_attractor(1).unwrap();

    let dust = Dust::new(pos_mul * planet.position(), vel_mul * planet.velocity());
    system.add_dust(dust);

    system
}

enum SimResult {
    FailedHigh(Vec<Vec2>),
    FailedLow(Vec<Vec2>),
    Success(SimData),
}

impl SimResult {
    fn push_to(&self, paths: &mut Vec<Vec<(f32, f32)>>) {
        let path_points = match self {
            SimResult::FailedHigh(path) => path,
            SimResult::FailedLow(path) => path,
            SimResult::Success(data) => &data.dust_path,
        }
        .iter()
        .map(|p| (p.x, p.y))
        .collect::<Vec<(f32, f32)>>();
        paths.push(path_points);
    }
}

struct SimData {
    dust_path: Vec<Vec2>,
    final_vel: Vec2,
}

fn simulate_dust(pos_mul: f32, vel_mul: f32, planet_orbit_radius: f32) -> SimResult {
    let mut system = system_setup(pos_mul, vel_mul);
    let initial_pos = system.get_dust(0).unwrap().position();

    let mut dust_path = Vec::new();
    let mut prev_system = system.clone();

    system.update_until(
        |sys| {
            let transform = sys.rotate_around(1).unwrap();
            let dust_pos = sys.get_dust(0).unwrap().position();
            let dust_pos_rot = transform(dust_pos);

            dust_path.push(dust_pos_rot);

            let prev_dust_pos = prev_system.get_dust(0).unwrap().position();
            let prev_dust_rot = prev_system.rotate_around(1).unwrap()(prev_dust_pos);
            if prev_dust_rot.y < 0.0 && dust_pos_rot.y >= 0.0 {
                return true;
            }
            prev_system = sys.clone();

            dust_pos_rot.length() > 2.0 * planet_orbit_radius
                || dust_pos_rot.x < 0.0
                || dust_pos_rot.x > 1.5 * planet_orbit_radius // || dust_pos_rot.y < 0.0
        },
        0.1,
        5,
        100_000,
        None,
        None,
    );

    let new_dust = system
        .get_dust(0)
        .unwrap()
        .coord_swap(system.rotate_around(1).unwrap());
    if new_dust.position().x > planet_orbit_radius {
        SimResult::FailedHigh(dust_path)
    } else if new_dust.position().x < initial_pos.x {
        SimResult::FailedLow(dust_path)
    } else {
        SimResult::Success(SimData {
            dust_path,
            final_vel: new_dust.velocity(),
        })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let planet_orbit_radius = sun_planet_binary_ccw::<VV>(100.0, 5.0)
        .get_attractor(1)
        .unwrap()
        .position()
        .length();

    let mut paths = Vec::new();

    let pos_mul = 0.6;
    let mut vel_mul_a = 0.8;
    let mut vel_mul_b = 3.0;

    simulate_dust(pos_mul, vel_mul_a, planet_orbit_radius).push_to(&mut paths);
    simulate_dust(pos_mul, vel_mul_b, planet_orbit_radius).push_to(&mut paths);

    for _ in 0..60 {
        let midpoint = 0.5 * (vel_mul_a + vel_mul_b);
        let sim_result = simulate_dust(pos_mul, midpoint, planet_orbit_radius);
        match sim_result {
            SimResult::FailedHigh(_) => {
                vel_mul_b = midpoint;
                println!("HIGH: {}", midpoint);
            }
            SimResult::FailedLow(_) => {
                vel_mul_a = midpoint;
                println!("LOW: {}", midpoint);
            }
            SimResult::Success(ref sim_result) => {
                println!("SUCCESS");
                let simres_a = simulate_dust(pos_mul, vel_mul_a, planet_orbit_radius);
                let simres_b = simulate_dust(pos_mul, vel_mul_b, planet_orbit_radius);

                let val_m = sim_result.final_vel.dot(Vec2::X);

                if let SimResult::FailedHigh(_) = simres_a {
                    println!("Unexpected HIGH at A");
                    continue;
                } else if let SimResult::FailedLow(_) = simres_a {
                    vel_mul_a = midpoint;
                    continue;
                }

                if let SimResult::FailedLow(_) = simres_b {
                    println!("Unexpected LOW at B");
                    continue;
                } else if let SimResult::FailedHigh(_) = simres_b {
                    vel_mul_b = midpoint;
                    continue;
                }

                let val_a = if let SimResult::Success(data) = simres_a {
                    data.final_vel.angle_between(Vec2::X)
                } else {
                    0.0
                };
                let val_b = if let SimResult::Success(data) = simres_b {
                    data.final_vel.angle_between(Vec2::X)
                } else {
                    0.0
                };

                if val_a * val_m < 0.0 {
                    vel_mul_b = midpoint;
                } else if val_b * val_m < 0.0 {
                    vel_mul_a = midpoint;
                }
            }
        }

        sim_result.push_to(&mut paths);
        println!("{}", paths.len());
    }

    println!("Final pos_mul: {}", pos_mul);
    println!("Final vel_mul: {}", 0.5 * (vel_mul_a + vel_mul_b));
    // 0.7 1.29

    let circle_points = ((0..100).chain(0..1))
        .map(|n| {
            let angle = n as f32 * PI / 50.0;
            (
                planet_orbit_radius * angle.cos(),
                planet_orbit_radius * angle.sin(),
            )
        })
        .collect::<Vec<(f32, f32)>>();

    let size = 1000;
    let root = BitMapBackend::new("./pics/output0.png", (size, size)).into_drawing_area();
    root.fill(&WHITE)?;

    let limit = 2.0 * planet_orbit_radius;
    let mut chart = ChartBuilder::on(&root)
        .margin(0)
        .build_cartesian_2d(-limit..limit, -limit..limit)?;

    chart.configure_mesh().draw()?;

    let num_paths = paths.len();
    for (i, path) in paths.into_iter().enumerate() {
        let col = if i == num_paths - 1 {
            MAGENTA
        } else {
            match i {
                0 => RED,
                1 => GREEN,
                _ => BLUE,
            }
        };
        chart.draw_series(LineSeries::new(path, col))?;
    }

    chart.draw_series(LineSeries::new(circle_points, &BLACK))?;

    root.present()?;

    Ok(())
}
