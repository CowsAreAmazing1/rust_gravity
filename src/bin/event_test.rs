use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).event(event).run();
}

struct Model {}

fn model(app: &App) -> Model {
    app.new_window().size(1200, 600).view(view).build().unwrap();

    Model {}
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn view(_app: &App, _model: &Model, _frame: Frame) {}

fn event(_app: &App, _model: &mut Model, event: Event) {
    if let Event::WindowEvent {
        simple: Some(window_event),
        ..
    } = event
    {
        match window_event {
            Moved(vec2) => println!("Window moved to: {:?}", vec2),
            KeyPressed(virtual_key_code) => println!("Key pressed: {:?}", virtual_key_code),
            KeyReleased(virtual_key_code) => println!("Key released: {:?}", virtual_key_code),
            ReceivedCharacter(_) => println!("Character received"),
            MouseMoved(vec2) => println!("Mouse moved to: {:?}", vec2),
            MousePressed(mouse_button) => println!("Mouse button pressed: {:?}", mouse_button),
            MouseReleased(mouse_button) => println!("Mouse button released: {:?}", mouse_button),
            MouseEntered => println!("Mouse entered window"),
            MouseExited => println!("Mouse exited window"),
            MouseWheel(mouse_scroll_delta, touch_phase) => println!(
                "Mouse wheel scrolled: {:?}, phase: {:?}",
                mouse_scroll_delta, touch_phase
            ),
            Resized(vec2) => println!("Window resized to: {:?}", vec2),
            HoveredFile(path_buf) => println!("File hovered: {:?}", path_buf),
            DroppedFile(path_buf) => println!("File dropped: {:?}", path_buf),
            HoveredFileCancelled => println!("File hover cancelled"),
            Touch(touch_event) => println!("Touch event: {:?}", touch_event),
            TouchPressure(touchpad_pressure) => println!("Touch pressure: {:?}", touchpad_pressure),
            Focused => println!("Window focused"),
            Unfocused => println!("Window unfocused"),
            Closed => println!("Window closed"),
        }
    }
}
