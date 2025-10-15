
use main_gravity::{Attractor, Quad, Setup, System, Uniforms};
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

    let mut setup = Setup::new();
    setup
        .add(Quad::new().square(200.0).center_position(vec2( 500.0,  500.0)).orbit(Vec2::ZERO, 800.0, false))
        .add(Quad::new().square(200.0).center_position(vec2(-500.0,  500.0)).orbit(Vec2::ZERO, 800.0, false))
        .add(Quad::new().square(200.0).center_position(vec2( 500.0, -500.0)).orbit(Vec2::ZERO, 800.0, false))
        .add(Quad::new().square(200.0).center_position(vec2(-500.0, -500.0)).orbit(Vec2::ZERO, 800.0, false));
    system.include_setup(&setup, 1_000_000);

    system.init_gpu(device);

    Model { system }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let window = app.main_window();
    let device = window.device();
    let queue = window.queue();

    model.system.update(1.0, 10, device, queue);
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