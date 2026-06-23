use main_gravity::prelude::*;
use nannou::prelude::*;

struct Model {
    system: System,
    ih: InteractionHandler,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(1500, 1500)
        .view(view)
        .build()
        .unwrap();
    let window = app.main_window();
    let device = window.device();

    let mut system = System::new();

    // Launched attractor
    let attractor = Attractor::new(vec2(-250.0, 0.0), vec2(30.0, 0.1), 300.0, 0.0);
    system.add_attractor(attractor);

    let mut setup = Setup::new();
    setup
        .add(Disc::new().center_position(vec2(200.0, 0.0)))
        .add(Disc::new().center_position(vec2(100.0, 0.0)))
        .add(Disc::new().center_position(vec2(0.0, 0.0)))
        .add(Disc::new().center_position(vec2(-100.0, 0.0)))
        .add(Disc::new().center_position(vec2(-200.0, 0.0)))
        .add(
            Disc::new()
                .center_position(attractor.position())
                .center_velocity(attractor.velocity())
                .speed_scale(0.5)
                .inner_radius(10.0)
                .outer_radius(20.0)
                .orbit_attractor(&attractor, true),
        )
        .add(
            Quad::new()
                .width(100.0)
                .height(5.0)
                .rotate(1.0)
                .center_position(attractor.position() + vec2(100.0, 0.0))
                .center_velocity_xy(-30.0, 0.0),
        );

    system.include_setup(&setup, 8_000_000);
    system.init_gpu(device);

    let ih = InteractionHandler::from_rect(&window.rect())
        .set_dt(0.01)
        .set_scale(2.2);

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

    let draw = model.ih.draw(app.draw());

    // Update uniforms before drawing
    if let Some(gpu_state) = &model.system.gpu_state {
        let uniforms = model.ih.uniform();
        gpu_state.update_uniforms(queue, &uniforms);
    }

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
