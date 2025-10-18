use nannou::prelude::*;
use main_gravity::{Attractor, System};

fn main() {
    nannou::app(model).update(update).view(view).run();
}

struct Model {
    system: System,
}

fn model(app: &App) -> Model {
    let _window = app.new_window().view(view).build().unwrap();

    let mut system = System::new();

    let planet = Attractor::new(vec2(0.0, 0.0), vec2(0.0, 0.0), 500.0, 120.0);
    system.add_attractor(planet);
    
    let moon = Attractor::new(vec2(150.0, 0.0), vec2(0.0, (500.0 / 150.0).sqrt()), 10.0, 250.0);
    system.add_attractor(moon);

    Model { system }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let window = app.main_window();
    let queue = window.queue();
    let device = window.device();

    model.system.update(0.5, 5, Some(device), Some(queue));
}

fn view(app: &App, model: &Model, frame: Frame) {
    let window = app.main_window();
    let device = window.device();
    let queue = window.queue();
    let texture_view = frame.texture_view();

    let draw = app.draw();
    draw.background().color(BLACK);

    model.system.draw(&draw, device, queue, texture_view);
    
    draw.to_frame(app, &frame).unwrap();
}