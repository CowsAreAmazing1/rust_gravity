use nannou::{
    event::{Key, MouseButton, MouseScrollDelta, WindowEvent},
    geom::Rect,
    glam::{vec2, Vec2},
    App, Draw, Event,
};

use crate::Uniforms;

pub struct InteractionHandler {
    dragging: bool,
    last_mouse_pos: Option<Vec2>,
    pub rotate: bool,
    pub play: bool,

    pub dt: f32,
    pub scale: f32,
    pub camera_translation: Vec2, // Camera drag translation
    window_size: Vec2,            // [width, height] in pixels
    pub rotation_angle: f32,      // Rotation angle in radians
    pub rotation_center: Vec2,    // Point to rotate around (in world coordinates)
}

impl InteractionHandler {
    pub fn from_rect(rect: &Rect) -> Self {
        InteractionHandler {
            window_size: rect.wh(),
            ..Default::default()
        }
    }

    pub fn uniform(&self) -> Uniforms {
        if self.rotate {
            Uniforms::with_rotation(
                self.scale,
                self.camera_translation,
                self.window_size,
                self.rotation_angle,
                self.rotation_center,
            )
        } else {
            Uniforms::new(self.scale, self.camera_translation, self.window_size)
        }
    }

    pub fn draw(&self, draw: Draw) -> Draw {
        let camera_translation = self.camera_translation.extend(0.0);
        let rotation_center = self.rotation_center.extend(0.0);

        let mut new_draw = draw.translate(-camera_translation);

        if self.rotate {
            let rotated_center = rotation_center - camera_translation;

            // Match the shader order: translate the view first, then rotate around the
            // translated center, then apply the final scale.
            new_draw = new_draw
                .translate(-rotated_center)
                .rotate(self.rotation_angle)
                // .translate(rotated_center)
                ;
        }

        new_draw.scale(self.scale)
    }

    pub fn custom_event_handler(&mut self, app: &App, event: Event) {
        if let Event::WindowEvent {
            simple: Some(event),
            ..
        } = event
        {
            match event {
                WindowEvent::MouseWheel(scroll, _) => {
                    let scale_factor = match scroll {
                        MouseScrollDelta::LineDelta(_, y) => 1.0 + y * 0.1,
                        MouseScrollDelta::PixelDelta(pos) => 1.0 + pos.y as f32 * 0.0001,
                    };
                    if app.keys.down.contains(&Key::LShift) {
                        self.dt *= scale_factor;
                        println!("Timestep: {}", self.dt);
                    } else {
                        self.scale *= scale_factor;
                        println!("Scale: {}", self.scale);
                    }
                }
                WindowEvent::MousePressed(MouseButton::Left) => {
                    self.dragging = true;
                }
                WindowEvent::MouseReleased(MouseButton::Left) => {
                    self.dragging = false;
                    self.last_mouse_pos = None;
                }
                WindowEvent::MouseMoved(pos) => {
                    if self.dragging {
                        if let Some(last_pos) = self.last_mouse_pos {
                            let delta = pos - last_pos;
                            let translation = delta / self.scale;
                            self.camera_translation -= translation;
                        }
                        self.last_mouse_pos = Some(pos);
                    }
                }
                WindowEvent::KeyPressed(key) => match key {
                    Key::R => {
                        self.rotate = !self.rotate;
                        println!("Rotation toggled to: {}", self.rotate);
                    }
                    Key::Space => {
                        self.play = !self.play;
                        println!(
                            "Simulation {}",
                            if self.play { "resumed" } else { "paused" }
                        )
                    }
                    _ => {}
                },
                WindowEvent::Resized(size) => self.window_size = size,
                _ => {}
            }
        }
    }
}

impl Default for InteractionHandler {
    fn default() -> Self {
        Self {
            dragging: false,
            last_mouse_pos: None,
            rotate: false,
            play: true,
            dt: 1.1,
            scale: 1.0,
            camera_translation: Vec2::ZERO,
            window_size: vec2(100.0, 100.0),
            rotation_angle: 0.0,
            rotation_center: Vec2::ZERO,
        }
    }
}
