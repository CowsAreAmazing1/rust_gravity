use main_gravity::prelude::*;
use nannou::prelude::*;

struct Model {
    system: System,
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

    let attractor = Attractor::new(vec2(-500.0, 0.0), vec2(5.0, 0.0), 100.0, 0.0);
    system.add_attractor(attractor);
    let attractor = Attractor::new(vec2(-500.0, 10.0), vec2(5.0, 0.0), 100.0, 0.0);
    system.add_attractor(attractor);
    let attractor = Attractor::new(vec2(-500.0, -13.0), vec2(5.0, 0.0), 100.0, 0.0);
    system.add_attractor(attractor);

    let mut setup = Setup::new();
    setup.add(Disc::new().radius(20.0).center_velocity_xy(-5.0, 0.0));
    system.include_setup_random(&setup, 6_000_000);
    system.init_gpu(device);
    system.dust.clear();

    let ih = InteractionHandler::from_rect(&app.window_rect());

    Model { system, ih }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let window = app.main_window();
    let device = window.device();
    let queue = window.queue();

    model
        .system
        .update(model.ih.dt, 10, Some(device), Some(queue));
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
