use nannou::prelude::*;
use nannou_egui::{self, egui, Egui};
use main_gravity::{System, Attractor};

fn main() {
    nannou::app(model).update(update).view(view).run();
}

struct Model {
    system_rk4: System,
    system_euler: System,
    ui: Egui,
    paused: bool,
    dt: f32,
    show_trails: bool,
    trail_points_rk4: Vec<Vec<Vec2>>,
    trail_points_euler: Vec<Vec<Vec2>>,
    max_trail_length: usize,
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .title("RK4 vs Euler Integration Comparison")
        .size(1200, 600)
        .view(view)
        .raw_event(raw_window_event)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();
    let ui = Egui::from_window(&window);

    // Create identical systems for comparison
    let mut system_rk4 = System::new();
    let mut system_euler = System::new();
    
    system_rk4.set_integration_method(true);  // Use RK4
    system_euler.set_integration_method(false); // Use Euler
    
    // Create a central massive body
    let sun_rk4 = Attractor::new(Vec2::new(-300.0, 0.0), Vec2::ZERO, 1000.0, 60.0);
    let sun_euler = Attractor::new(Vec2::new(300.0, 0.0), Vec2::ZERO, 1000.0, 60.0);
    
    system_rk4.add_attractor(sun_rk4);
    system_euler.add_attractor(sun_euler);
    
    // Create orbiting bodies
    let planet_rk4 = Attractor::new(Vec2::new(-150.0, 0.0), Vec2::new(0.0, 8.0), 1.0, 200.0);
    let planet_euler = Attractor::new(Vec2::new(450.0, 0.0), Vec2::new(0.0, 8.0), 1.0, 200.0);
    
    system_rk4.add_attractor(planet_rk4);
    system_euler.add_attractor(planet_euler);
    
    // Add a moon to each planet
    let moon_rk4 = Attractor::new(Vec2::new(-130.0, 0.0), Vec2::new(0.0, 12.0), 0.1, 300.0);
    let moon_euler = Attractor::new(Vec2::new(470.0, 0.0), Vec2::new(0.0, 12.0), 0.1, 300.0);
    
    system_rk4.add_attractor(moon_rk4);
    system_euler.add_attractor(moon_euler);

    Model {
        system_rk4,
        system_euler,
        ui,
        paused: false,
        dt: 0.01,
        show_trails: true,
        trail_points_rk4: vec![Vec::new(); 3], // 3 bodies
        trail_points_euler: vec![Vec::new(); 3],
        max_trail_length: 500,
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    if !model.paused {
        // Update both systems
        model.system_rk4.update(model.dt);
        model.system_euler.update(model.dt);
        
        // Store trail points
        if model.show_trails {
            let bodies_rk4 = model.system_rk4.get_bodies();
            let bodies_euler = model.system_euler.get_bodies();
            
            for (i, body) in bodies_rk4.iter().enumerate() {
                model.trail_points_rk4[i].push(body.position());
                if model.trail_points_rk4[i].len() > model.max_trail_length {
                    model.trail_points_rk4[i].remove(0);
                }
            }
            
            for (i, body) in bodies_euler.iter().enumerate() {
                model.trail_points_euler[i].push(body.position());
                if model.trail_points_euler[i].len() > model.max_trail_length {
                    model.trail_points_euler[i].remove(0);
                }
            }
        }
    }
    
    // Update UI
    let egui = &mut model.ui;
    egui.set_elapsed_time(_update.since_start);
    let ctx = egui.begin_frame();
    
    egui::Window::new("Simulation Controls").show(&ctx, |ui| {
        ui.label("Integration Method Comparison");
        ui.separator();
        
        if ui.button(if model.paused { "Resume" } else { "Pause" }).clicked() {
            model.paused = !model.paused;
        }
        
        ui.add(egui::Slider::new(&mut model.dt, 0.001..=0.1).text("Time Step"));
        
        ui.checkbox(&mut model.show_trails, "Show Trails");
        
        if model.show_trails {
            ui.add(egui::Slider::new(&mut model.max_trail_length, 50..=1000).text("Trail Length"));
        }
        
        if ui.button("Reset Trails").clicked() {
            for trail in &mut model.trail_points_rk4 {
                trail.clear();
            }
            for trail in &mut model.trail_points_euler {
                trail.clear();
            }
        }
        
        ui.separator();
        ui.label("Left: RK4 Integration (more accurate)");
        ui.label("Right: Euler Integration (less accurate)");
    });
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);
    
    // Draw separator line
    draw.line()
        .start(Vec2::new(0.0, -300.0))
        .end(Vec2::new(0.0, 300.0))
        .color(GRAY)
        .weight(2.0);
    
    // Draw labels
    draw.text("RK4 (Accurate)")
        .xy(Vec2::new(-300.0, 280.0))
        .color(WHITE)
        .font_size(16);
        
    draw.text("Euler (Less Accurate)")
        .xy(Vec2::new(300.0, 280.0))
        .color(WHITE)
        .font_size(16);
    
    // Draw trails for RK4 system
    if model.show_trails {
        for (i, trail) in model.trail_points_rk4.iter().enumerate() {
            if trail.len() > 1 {
                let color = match i {
                    0 => YELLOW,      // Sun
                    1 => BLUE,        // Planet
                    2 => WHITE,       // Moon
                    _ => GRAY,
                };
                
                for window in trail.windows(2) {
                    draw.line()
                        .start(window[0])
                        .end(window[1])
                        .color(color)
                        .weight(1.0);
                }
            }
        }
        
        // Draw trails for Euler system
        for (i, trail) in model.trail_points_euler.iter().enumerate() {
            if trail.len() > 1 {
                let color = match i {
                    0 => YELLOW,      // Sun
                    1 => BLUE,        // Planet
                    2 => WHITE,       // Moon
                    _ => GRAY,
                };
                
                for window in trail.windows(2) {
                    draw.line()
                        .start(window[0])
                        .end(window[1])
                        .color(color)
                        .weight(1.0);
                }
            }
        }
    }
    
    // Draw RK4 system bodies
    for body in model.system_rk4.get_bodies() {
        let radius = (body.mass().sqrt() * 2.0).max(2.0);
        draw.ellipse()
            .xy(body.position())
            .radius(radius)
            .color(body.color());
    }
    
    // Draw Euler system bodies
    for body in model.system_euler.get_bodies() {
        let radius = (body.mass().sqrt() * 2.0).max(2.0);
        draw.ellipse()
            .xy(body.position())
            .radius(radius)
            .color(body.color());
    }
    
    draw.to_frame(app, &frame).unwrap();
    model.ui.draw_to_frame(&frame).unwrap();
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.ui.handle_raw_event(event);
}