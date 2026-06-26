use main_gravity::prelude::*;
use nannou::prelude::*;

struct Model {
    system: System<VV>,
    ih: InteractionHandler,
}

fn model(app: &App) -> Model {
    app.new_window().size(1500, 800).view(view).build().unwrap();
    let window = app.main_window();
    let device = window.device();

    let mut system = System::new();

    // Sun
    let attractor = Attractor::new(Vec2::ZERO, Vec2::ZERO, 1000.0, 50.0);
    system.add_attractor(attractor);

    // Planets
    let planet_x = 200.0;
    let attractor = Attractor::new(
        vec2(-planet_x, 0.0),
        vec2(0.0, -(1000.0 / planet_x).sqrt()),
        100.0,
        150.0,
    );
    system.add_attractor(attractor);
    let attractor = Attractor::new(
        vec2(planet_x, 0.0),
        vec2(0.0, (1000.0 / planet_x).sqrt()),
        100.0,
        150.0,
    );
    system.add_attractor(attractor);

    let dist = 200.0;
    let mut setup = Setup::new();
    setup
        .add(
            Quad::new()
                .square(200.0)
                .center_position(vec2(dist, dist))
                .orbit(Vec2::ZERO, 800.0, false),
        )
        .add(
            Quad::new()
                .square(200.0)
                .center_position(vec2(-dist, dist))
                .orbit(Vec2::ZERO, 800.0, false),
        )
        .add(
            Quad::new()
                .square(200.0)
                .center_position(vec2(dist, -dist))
                .orbit(Vec2::ZERO, 800.0, false),
        )
        .add(
            Quad::new()
                .square(200.0)
                .center_position(vec2(-dist, -dist))
                .orbit(Vec2::ZERO, 800.0, false),
        );

    system.include_setup_random(&setup, 8_000_000);
    system.init_gpu(device);

    let ih = InteractionHandler::from_rect(&window.rect());

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

fn main() {
    nannou::app(model).update(update).event(event).run();
}
