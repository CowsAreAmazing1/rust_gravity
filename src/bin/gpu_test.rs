
use main_gravity::{Attractor, Dust, System, Uniforms};
use nannou::prelude::*;


struct Model {
    system: System,
}

fn model(app: &App) -> Model {
    app.new_window().size(1500, 1500).view(view).build().unwrap();
    let window = app.main_window();
    let device = window.device();

    let mut system = System::new();

    // Sun
    let attractor = Attractor::new(Vec2::ZERO, Vec2::ZERO, 1000.0, 50.0);
    system.add_attractor(attractor);

    // Planets
    let planet_x = 200.0;
    let attractor = Attractor::new(vec2(-planet_x, 0.0), vec2(0.0, -(1000.0/planet_x).sqrt()), 100.0, 150.0);
    system.add_attractor(attractor);
    let attractor = Attractor::new(vec2( planet_x, 0.0), vec2(0.0,  (1000.0/planet_x).sqrt()), 100.0, 150.0);
    system.add_attractor(attractor);
    let attractor = Attractor::new(vec2(0.0, -planet_x), vec2(-(1000.0/planet_x).sqrt(), 0.0), 100.0, 150.0);
    system.add_attractor(attractor);
    let attractor = Attractor::new(vec2(0.0,  planet_x), vec2( (1000.0/planet_x).sqrt(), 0.0), 100.0, 150.0);
    system.add_attractor(attractor);

    let num_dusts = 5_000_000;

    for i in 0..num_dusts { // Reduce count for easier debugging
        let (xmid, ymid) = match (i as f32 / num_dusts as f32 * 4.0).floor() as u32 {
            0 => ( 500.0,  500.0),
            1 => (-500.0,  500.0),
            2 => ( 500.0, -500.0),
            3 => (-500.0, -500.0),
            _ => panic!("Ahh")
        };

        let position = vec2(
            random_range(xmid - 100.0, xmid + 100.0),
            random_range(ymid - 100.0, ymid + 100.0),
        );
        let velocity = position.normalize().perp() * 0.5;
        let new_dust = Dust::new(position, velocity, random_range(0.0, 200.0));
        system.add_dust(new_dust);
    }
    system.init_gpu(device);

    println!("System initialized with {} dust particles.", num_dusts);

    Model { system }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let window = app.main_window();
    let device = window.device();
    let queue = window.queue();

    model.system.update(1.0, device, queue);
}

fn view(app: &App, model: &Model, frame: Frame) {
    let window = app.main_window();
    let device = window.device();
    let queue = window.queue();
    let texture_view = frame.texture_view();

    let draw = app.draw();

    // Update uniforms before drawing
    if let Some(gpu_state) = &model.system.gpu_state {
        let window_rect = app.window_rect();
        let uniforms = Uniforms::new(
            1.0,
            Vec2::ZERO,
            window_rect.wh()
        );
        gpu_state.update_uniforms(queue, &uniforms);
    }

    model.system.draw(&draw, device, queue, texture_view);

    draw.to_frame(app, &frame).unwrap();
}

fn main() {
    nannou::app(model).update(update).run();
}