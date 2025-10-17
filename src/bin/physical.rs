
use nannou::prelude::*;
use main_gravity::{Attractor, Quad, Setup, System, Uniforms};



struct Model {
    system: System,
    zoom: f32,
    rotate: bool,
    dragging: bool,
    last_mouse_pos: Option<Vec2>,
    view_translation: Vec2,
}

impl Model {
    fn new(system: System) -> Self {
        Self {
            system,
            zoom: 1.0,
            rotate: true,
            dragging: false,
            last_mouse_pos: None,
            view_translation: Vec2::ZERO,
        }
    }
}

fn model(app: &App) -> Model {
    app.new_window().view(view).size(1500, 1500).build().unwrap();
    let window = app.main_window();
    let device = window.device();
    window.set_outer_position_pixels(50, 50);

    let mut system = System::new();
    
    // Create multiple bodies in a more complex system
    let mut sun = Attractor::new(Vec2::ZERO, Vec2::ZERO, 100.0, 0.0);
    let mut planet = Attractor::new(vec2(200.0, 0.0), Vec2::ZERO, 5.0, 100.0);    


    println!("planet vel: {:?}, sun vel: {:?}", planet.velocity, sun.velocity);
    sun.orbit_pair(&mut planet, false);
    println!("planet vel: {:?}, sun vel: {:?}", planet.velocity, sun.velocity);

    
    let soi = planet.position.x * (planet.mass / sun.mass).powf(2.0/5.0);
    let spawn_angle = PI / 2.0 + 0.01;
    let spawn_vec = vec2(spawn_angle.cos(), spawn_angle.sin());

    let mut setup = Setup::new();
    setup.add(Quad::new().center_position(planet.position + 0.4 * soi * spawn_vec).square(10.0).orbit(sun.position, sun.mass, false).orbit(planet.position, planet.mass, false));
    system.include_setup_random(&setup, 5_000_000);
    
    system.add_attractor(sun);
    system.add_attractor(planet);
    
    system.init_gpu(device);
    Model::new(system)
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let window = app.main_window();
    let device = window.device();
    let queue = window.queue();

    model.system.update(1.0, 5, device, queue);
}

fn view(app: &App, model: &Model, frame: Frame) {
    let window = app.main_window();
    let device = window.device();
    let queue = window.queue();
    let texture_view = frame.texture_view();

    let draw: Draw = app.draw();
    let scale_draw = draw.translate(-model.zoom * model.view_translation.extend(0.0)).scale(model.zoom);

    if let Some(gpu_state) = &model.system.gpu_state {
        let window_rect = app.window_rect();
        
        let uniforms = if model.rotate {
            let planet_angle = if let Some(planet) = model.system.get_body(1) {
                let com = model.system.center_of_mass();
                -(planet.position() - com).angle()
            } else {
                0.0
            };

            let trans_draw = scale_draw.rotate(planet_angle);
            model.system.draw(&trans_draw, device, queue, texture_view);

            Uniforms::with_rotation(
                model.zoom, 
                model.view_translation,
                window_rect.wh(), 
                planet_angle,
                model.system.center_of_mass(),
            )
        } else {
            model.system.draw(&scale_draw, device, queue, texture_view);
            Uniforms::new(
                model.zoom,
                model.view_translation,
                window_rect.wh()
            )
        };
        gpu_state.update_uniforms(queue, &uniforms);
    } else {
        model.system.draw(&scale_draw, device, queue, texture_view);
    }

    // scale_draw.translate(model.system.center_of_mass().extend(0.0))
    //     .ellipse()
    //     .radius(1.0)
    //     .color(WHITE);

    draw.to_frame(app, &frame).unwrap();
}

fn event(_app: &App, model: &mut Model, event: Event) {

    if let Event::WindowEvent { simple: Some(event), .. } = event {
        match event {
            WindowEvent::MouseWheel(scroll, _) => {
                let scale_factor = match scroll {
                    MouseScrollDelta::LineDelta(_, y) => 1.0 + y * 0.1,
                    MouseScrollDelta::PixelDelta(pos) => 1.0 + pos.y as f32 * 0.0001,
                };
                model.zoom *= scale_factor;
            },
            WindowEvent::MousePressed(MouseButton::Left) => {
                model.dragging = true;
            },
            WindowEvent::MouseReleased(MouseButton::Left) => {
                model.dragging = false;
                model.last_mouse_pos = None;
            },
            WindowEvent::MouseMoved(pos) => {
                if model.dragging {
                    if let Some(last_pos) = model.last_mouse_pos {
                        let delta = pos - last_pos;
                        let translation = delta / model.zoom;
                        model.view_translation -= translation;
                    }
                    model.last_mouse_pos = Some(pos);
                }
            },
            WindowEvent::KeyPressed(Key::R) => {
                model.rotate = !model.rotate;
                println!("Rotation toggled to: {}", model.rotate);
            }
            _ => {},
        }
    }
}

fn main() {
    nannou::app(model).update(update).event(event).run();
}