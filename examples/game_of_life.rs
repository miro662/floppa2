use cgmath::Vector2;
use floppa2::renderer::{Renderer, Color};
use rand::Rng;
use winit::{
    event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode, MouseButton},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder}, dpi::PhysicalSize,
};

const WINDOW_SIZE: u32 = 1200;
const CELL_SIZE: u32 = 20;
const CELL_COUNT: u32 = WINDOW_SIZE / CELL_SIZE; // 40

struct World {
    size: Vector2<usize>,
    data: Vec<bool>
}

impl World {
    fn empty(size: Vector2<usize>) -> World {
        World {size: size, data: vec![false; size.x * size.y]}
    }

    fn filled_at(size: Vector2<usize>, ratio: f32) -> World {
        let mut w = World::empty(size);
        let mut rng = rand::thread_rng();
        for x in 0..size.x {
            for y in 0..size.y {
                if rng.gen::<f32>() < ratio {
                    let position = w.to_position(x, y);
                    w.data[position] = true;
                }
            }
        }
        w
    }

    fn to_position(&self, x: usize, y: usize) -> usize {
        y * self.size.x + x
    }

    fn at(&self, x: isize, y: isize) -> Option<bool> {
        if x >= 0 && y >= 0 &&  x < self.size.x as isize && y < self.size.y as isize {
            Some(self.data[self.to_position(x as usize, y as usize)])
        } else {
            None
        }
    }

    fn at_or_false(&self, x: isize, y: isize) -> bool {
        self.at(x, y).is_some_and(|x| x)
    }

    fn toggle(&mut self, x: usize, y: usize) {
        let position = self.to_position(x, y);
        self.data[position] = !self.data[position];
    }

    fn step(&self) -> World {
        let mut new_world = World::empty(self.size);
        for x in 0..self.size.x {
            for y in 0..self.size.y {
                let position = new_world.to_position(x, y);
                let mut alive_neighbours = 0;
                for xx in -1..=1 {
                    for yy in -1..=1 {
                        if !(xx == 0 && yy == 0) && self.at_or_false(x as isize + xx, y as isize + yy) {
                            alive_neighbours += 1;
                        }
                    }
                }
                new_world.data[position] = alive_neighbours == 3 || (alive_neighbours == 2 && self.at_or_false(x as isize, y as isize));
            }
        };
        new_world
    }
}

struct Game {
    world: World,
    highlighted: Option<Vector2<u32>>
}

impl Game {
    fn new() -> Game {
        Game {
            world: World::filled_at((CELL_COUNT as usize, CELL_COUNT as usize).into(), 0.5),
            highlighted: None
        }
    }

    fn step(&mut self) {
        self.world = self.world.step();
    }

    fn render(&self, renderer: &Renderer) {
        let cell_size: Vector2<u32> = (CELL_SIZE, CELL_SIZE).into();
        renderer.render(|ctx| {
            for x in 0..CELL_COUNT {
                for y in 0..CELL_COUNT {
                    let cell_position = Vector2 { x: x * CELL_SIZE , y: y * CELL_SIZE };
                    let cell_state = self.world.at_or_false(x as isize, y as isize);
                    let cell_color = match (cell_state, Some((x, y).into()) == self.highlighted) {
                        (true, true) => Color {r: 1.0, g: 0.0, b: 0.0, a: 0.0},
                        (false, true) => Color {r: 0.2, g: 0.0, b: 0.0, a: 0.0},
                        (true, false) => Color::WHITE,
                        (false, false) => Color::BLACK
                    };
                    ctx.draw_rectangle(cell_size).at(cell_position).with_color(cell_color);
                }
            }
        });
    }

    fn on_mouse_move(&mut self, position: Vector2<f32>) {
        self.highlighted = Some((position.x as u32 / CELL_SIZE, position.y as u32 / CELL_SIZE).into())
    }

    fn on_mouse_left_button(&mut self) {
        if let Some(Vector2 {x, y}) = self.highlighted {
            self.world.toggle(x as usize, y as usize);
        }
    }
}


fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_inner_size(PhysicalSize{width: WINDOW_SIZE, height: WINDOW_SIZE}).build(&event_loop).unwrap();
    let window_size = Vector2{x: window.inner_size().width, y: window.inner_size().height};
    let renderer =  Renderer::compatible_with(&window, window_size.into());

    let mut game = Game::new();

    event_loop.run(move |event, _, cf| match event {
        Event::WindowEvent {
            window_id,
            event: win_event,
        } if window_id == window.id() => match win_event {
            WindowEvent::CloseRequested => *cf = ControlFlow::Exit,
            WindowEvent::KeyboardInput { input: KeyboardInput {
                virtual_keycode: Some(key),
                state: ElementState::Released,
                ..
            }, .. } => match key {
                VirtualKeyCode::N => {
                    game.step();
                    window.request_redraw()
                },
                _ => ()
            },
            WindowEvent::CursorMoved { position, .. } => {
                game.on_mouse_move((position.x as f32, WINDOW_SIZE as f32 - position.y as f32).into());
                window.request_redraw();
            },
            WindowEvent::MouseInput { button: MouseButton::Left, state: ElementState::Released, ..} => {
                game.on_mouse_left_button();
                window.request_redraw();
            },
            WindowEvent::Resized(new_size) => {
                renderer.resize((new_size.width, new_size.height).into());
                window.request_redraw();
            }
            _ => (),
        },
        Event::RedrawRequested(_) => game.render(&renderer),
        _ => (),
    });
}
