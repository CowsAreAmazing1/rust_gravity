use main_gravity::{Attractor, Disc, Setup, System, Uniforms};
use nannou::prelude::*;

struct Model {
    system: System,
    uniform: Uniforms,
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

    let attractor = Attractor::new(vec2(-500.0, 0.0), vec2(5.0, 0.0), 1000.0, 0.0);
    system.add_attractor(attractor);

    let mut setup = Setup::new();
    setup.add(Disc::new());
    system.include_setup(&setup, 100_000);
    system.init_gpu(device);
    system.dust.clear();

    let uniform = Uniforms::new(1.0, Vec2::ZERO, window.rect().wh());

    Model { system, uniform }
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

    if let Some(gpu_state) = &model.system.gpu_state {
        gpu_state.update_uniforms(queue, &model.uniform);
        println!("{:?}", model.uniform.camera_translation);
    }

    let draw = app.draw();
    model
        .system
        .draw(&draw, device, queue, texture_view, model.uniform.scale);
    draw.to_frame(app, &frame).unwrap();
}

fn event(app: &App, model: &mut Model, event: Event) {
    if let Event::WindowEvent {
        simple: Some(event),
        ..
    } = event
    {
        match event {
            WindowEvent::MouseWheel(scroll, _) => {
                let scale_factor = match scroll {
                    MouseScrollDelta::LineDelta(_, y) => 1.0 + y * 0.1,
                    MouseScrollDelta::PixelDelta(pos) => 1.0 + pos.y as f32 * 0.001,
                };
                model.uniform.scale *= scale_factor;
            }
            WindowEvent::Resized(size) => {
                model.uniform.aspect_ratio = size.x / size.y;
            }
            WindowEvent::MouseMoved(pos) => {
                if app.mouse.buttons.left().is_down() {
                    let rect = app.window_rect();
                    let translation = [
                        pos.x / model.uniform.scale / rect.right(),
                        pos.y / model.uniform.scale / rect.top(),
                    ];
                    model.uniform.camera_translation = translation;
                }
            }
            _ => {}
        }
    }
}

fn main() {
    nannou::app(model).update(update).event(event).run();
}
