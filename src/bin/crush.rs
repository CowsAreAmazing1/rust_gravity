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

    let num = 8;
    let scale = 3.0;
    for i in 0..num {
        let t = map_range(i, 0, num, 0.0, TAU);

        let pos: Vec2 = 300.0 * Vec2::from(t.sin_cos());
        let vel = -scale * pos.normalize();
        let mut attractor = Attractor::new(pos, vel, -10.0, 0.0);
        attractor.set_orbit(Vec2::ZERO, 200.0, true);
        *attractor.velocity_mut() = attractor.velocity() + vel;
        system.add_attractor(attractor);
    }

    let mut setup = Setup::new();
    setup.add(Disc::new().radius(100.0));

    system.include_setup_random(&setup, 8_000_000);
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
