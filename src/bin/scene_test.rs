
use main_gravity::{Attractor, Disc, Setup, System, Uniforms};
use nannou::prelude::*;


struct Model {
    system: System,
}

fn model(app: &App) -> Model {
    app.new_window().size(1500, 1500).view(view).build().unwrap();
    let window = app.main_window();
    let device = window.device();

    let mut system = System::new();


    // Launched attractor
    let attractor = Attractor::new(vec2(-210.0, 0.0), vec2(10.0, 0.0), 100.0, 150.0);
    system.add_attractor(attractor);
    

    let num_dusts = 500_000;
    let mut dusts = Vec::new();

    Setup::new()
        .add(Disc::new().center_position(vec2(200.0, 0.0)))
        .add(Disc::new().center_position(vec2(-200.0, 0.0)))
        .build(num_dusts, &mut dusts);
    system.dump_dust(dusts);
    

    system.init_gpu(device);

    println!("System initialized with {} dust particles.", num_dusts);

    Model { system }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let window = app.main_window();
    let device = window.device();
    let queue = window.queue();

    model.system.update(0.1, 10, device, queue);
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