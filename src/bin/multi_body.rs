
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
    
    // Create multiple bodies in a more complex system
    let center = Attractor::new(Vec2::ZERO, Vec2::ZERO, 2000.0, 0.0);
    system.add_attractor(center);
    
    // Create several orbiting bodies
    for i in 0..50 {
        let angle = (i as f32) * 2.0 * PI / 10.0;
        let distance = 100.0 + (i as f32) * 5.0;
        let position = Vec2::new(angle.cos() * distance, angle.sin() * distance);
        let velocity = Vec2::new(-angle.sin(), angle.cos()) * (1000.0 / distance).sqrt();
        let hue = (i as f32) * 72.0; // Different colors
        
        let body = Attractor::new(position, velocity, 100.0, hue);
        system.add_attractor(body);
    }
    
    Model { system }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    model.system.update(0.2);
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);
    
    model.system.draw(&draw);
    
    draw.to_frame(app, &frame).unwrap();
}