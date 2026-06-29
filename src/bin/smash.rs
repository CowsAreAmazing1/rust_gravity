use main_gravity::prelude::*;
use nannou::prelude::*;

struct Model {
    system: System<VV>,
    ih: InteractionHandler,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(1000, 1000)
        .view(view)
        .build()
        .unwrap();
    let window = app.main_window();
    let device = window.device();

    let mut system = System::new();

    let targets = [
        3.0 * vec2(50.0, 50.0),
        3.0 * vec2(50.0, -50.0),
        3.0 * vec2(-50.0, -50.0),
        3.0 * vec2(-50.0, 50.0),
        3.0 * vec2(50.0, 0.0),
        3.0 * vec2(0.0, 50.0),
        3.0 * vec2(0.0, -50.0),
        3.0 * vec2(-50.0, 0.0),
    ];

    for i in 0..1 {
        let t = 500.0 + 100.0 * i as f32;

        let pos = vec2(-t, 0.0);
        let vel = (targets[0] - pos).normalize() * 10.0;
        let mut attractor = Attractor::new(pos, vel, 100.0, 0.0);
        attractor.set_orbit(Vec2::ZERO, 100.0, false);
        system.add_attractor(attractor);

        let pos = vec2(0.0, t);
        let vel = (targets[1] - pos).normalize() * 10.0;
        let mut attractor = Attractor::new(pos, vel, 100.0, 0.0);
        attractor.set_orbit(Vec2::ZERO, 100.0, false);
        system.add_attractor(attractor);

        let pos = vec2(t, 0.0);
        let vel = (targets[2] - pos).normalize() * 10.0;
        let mut attractor = Attractor::new(pos, vel, 100.0, 0.0);
        attractor.set_orbit(Vec2::ZERO, 100.0, false);
        system.add_attractor(attractor);

        let pos = vec2(0.0, -t);
        let vel = (targets[3] - pos).normalize() * 10.0;
        let mut attractor = Attractor::new(pos, vel, 100.0, 0.0);
        attractor.set_orbit(Vec2::ZERO, 100.0, false);
        system.add_attractor(attractor);
    }

    let center = Attractor::new(Vec2::ZERO, Vec2::ZERO, 1000.0, 200.0);

    let mut setup = Setup::new();
    targets.iter().for_each(|&target| {
        setup.add(
            Quad::new()
                .square(70.0)
                .center_position(target)
                .orbit_attractor(&center, false),
        );
    });

    system.add_attractor(center);

    system.include_setup_random(&setup, 6_000_000);
    system.init_gpu(device);

    let ih = InteractionHandler::from_rect(&app.window_rect());

    Model { system, ih }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    if model.ih.play {
        let window = app.main_window();
        let device = window.device();
        let queue = window.queue();

        model
            .system
            .update(model.ih.dt, 10, Some(device), Some(queue));
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let window = app.main_window();
    let device = window.device();
    let queue = window.queue();
    let texture_view = frame.texture_view();

    if let Some(gpu_state) = &model.system.gpu_state {
        let uniform = model.ih.uniform();
        gpu_state.update_uniforms(queue, &uniform);
    }

    let draw = model.ih.draw(app.draw());
    model
        .system
        .draw(&draw, device, queue, texture_view, model.ih.scale);
    draw.to_frame(app, &frame).unwrap();
}

fn event(app: &App, model: &mut Model, event: Event) {
    model.ih.custom_event_handler(app, event);
}

fn main() {
    nannou::app(model).update(update).event(event).run();
}
