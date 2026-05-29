use main_gravity::prelude::*;
use nannou::prelude::*;

fn main() {
    nannou::app(model)
        .update(update)
        .event(event)
        .view(view)
        .run();
}

struct Model {
    system: System,
    ih: InteractionHandler,
}

fn model(app: &App) -> Model {
    app.new_window().view(view).build().unwrap();
    let window = app.main_window();
    let device = window.device();

    let mut system = System::new();

    let planet = Attractor::new(vec2(0.0, 0.0), vec2(0.0, 0.0), 500.0, 120.0);
    system.add_attractor(planet);

    let moon = Attractor::new(
        vec2(150.0, 0.0),
        vec2(0.0, (500.0 / 150.0).sqrt()),
        10.0,
        250.0,
    );
    system.add_attractor(moon);

    let mut setup = Setup::new();
    setup.add(
        Quad::new()
            .square(100.0)
            .center_position(vec2(150.0, 0.0))
            .orbit_attractor(&moon, false),
    );

    system.include_setup(&setup, 1_000_000);
    system.init_gpu(device);
    system.dust.clear();

    let ih = InteractionHandler::from_rect(&window.rect());

    Model { system, ih }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let window = app.main_window();
    let device = window.device();
    let queue = window.queue();

    model.system.update(0.1, 10, Some(device), Some(queue));
}

fn view(app: &App, model: &Model, frame: Frame) {
    let window = app.main_window();
    let device = window.device();
    let queue = window.queue();
    let texture_view = frame.texture_view();

    let draw = model.ih.draw(app.draw());

    // Update uniforms before drawing
    if let Some(gpu_state) = &model.system.gpu_state {
        let uniforms = model.ih.uniform();
        gpu_state.update_uniforms(queue, &uniforms);
    }

    model.system.draw(&draw, device, queue, texture_view, 1.0);

    draw.to_frame(app, &frame).unwrap();
}

fn event(app: &App, model: &mut Model, event: Event) {
    model.ih.custom_event_handler(app, event);
}
