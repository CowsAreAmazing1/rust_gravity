use main_gravity::prelude::*;
use nannou::prelude::*;

struct Model {
    system: System,
    ih: InteractionHandler,
}

fn model(app: &App) -> Model {
    app.new_window().view(view).size(1000, 800).build().unwrap();
    let window = app.main_window();
    let device = window.device();
    window.set_outer_position_pixels(50, 50);

    let mut system = System::new();

    let mut sun = Attractor::new(Vec2::ZERO, Vec2::ZERO, 100.0, 0.0);
    let mut planet = Attractor::new(vec2(200.0, 0.0), Vec2::ZERO, 4.0, 100.0);

    sun.orbit_pair(&mut planet, false);

    // let soi = planet.position().x * (planet.mass() / sun.mass()).powf(2.0/5.0);
    // let spawn_angle = PI / 2.0 + 0.01;
    // let spawn_vec = vec2(spawn_angle.cos(), spawn_angle.sin());

    let mut setup = Setup::new();
    // setup.add(Quad::new().center_position(planet.position() + 0.4 * soi * spawn_vec).square(10.0).orbit(sun.position(), sun.mass(), false).orbit(planet.position(), planet.mass(), false));
    setup.add(
        Quad::new()
            .center_position(0.3 * sun.position() + 0.7 * planet.position())
            .square(1.0)
            .center_velocity_xy(0.0, 0.94),
    );
    system.include_setup_random(&setup, 5_000_000);

    system.add_attractor(sun);
    system.add_attractor(planet);

    system.init_gpu(device);
    system.dust.clear();

    let ih = InteractionHandler::from_rect(&window.rect());

    Model { system, ih }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let window = app.main_window();
    let device = window.device();
    let queue = window.queue();

    model
        .system
        .update(model.ih.dt, 5, Some(device), Some(queue));

    let com = model.system.center_of_mass();
    let planet = model.system.get_body(1).unwrap();
    let planet_angle = (planet.position() - com).angle();

    model.ih.rotation_angle = -planet_angle;
    model.ih.rotation_center = model.system.center_of_mass();
}

fn view(app: &App, model: &Model, frame: Frame) {
    let window = app.main_window();
    let device = window.device();
    let queue = window.queue();
    let texture_view = frame.texture_view();

    let draw: Draw = model.ih.draw(app.draw());

    if let Some(gpu_state) = &model.system.gpu_state {
        let uniforms = model.ih.uniform();
        gpu_state.update_uniforms(queue, &uniforms);
    }

    model
        .system
        .draw(&draw, device, queue, texture_view, model.ih.scale);

    draw.translate(model.system.center_of_mass().extend(0.0))
        .ellipse()
        .radius(1.0)
        .color(WHITE);

    draw.to_frame(app, &frame).unwrap();
}

fn event(app: &App, model: &mut Model, event: Event) {
    model.ih.custom_event_handler(app, event);
}

fn main() {
    nannou::app(model).update(update).event(event).run();
}
