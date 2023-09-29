use floppa2::renderer::{Renderer, WindowSize};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

struct State {
    renderer: Renderer,
}

impl State {
    fn init(window: &GameWindow) -> State {
        State {
            renderer: Renderer::compatible_with(window.handle(), window.size()),
        }
    }

    fn resize(&mut self, size: WindowSize) {
        self.renderer.resize(size);
    }

    fn render(&self) {
        self.renderer.render();
    }
}

struct GameWindow {
    event_loop: EventLoop<()>,
    window: Window,
}

impl GameWindow {
    fn new() -> GameWindow {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();
        GameWindow { event_loop, window }
    }

    fn launch(self, mut state: State) {
        self.event_loop.run(move |event, _, cf| match event {
            Event::WindowEvent {
                window_id,
                event: win_event,
            } if window_id == self.window.id() => match win_event {
                WindowEvent::CloseRequested => *cf = ControlFlow::Exit,
                WindowEvent::Resized(new_size) => {
                    let new_size = WindowSize {
                        width: new_size.width,
                        height: new_size.height,
                    };
                    state.resize(new_size);
                    self.window.request_redraw();
                }
                _ => (),
            },
            Event::RedrawRequested(_) => state.render(),
            _ => (),
        })
    }

    fn handle(&self) -> impl HasRawWindowHandle + HasRawDisplayHandle + '_ {
        &self.window
    }

    fn size(&self) -> WindowSize {
        let inner_size = self.window.inner_size();
        WindowSize {
            width: inner_size.width,
            height: inner_size.height,
        }
    }
}

fn main() {
    let window = GameWindow::new();
    let state = State::init(&window);
    window.launch(state);
}
