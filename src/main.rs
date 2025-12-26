mod graphics;
mod game;

use std::time::Instant;

use graphics::Graphics;
use game::Game;
use winit::event_loop::{ControlFlow, EventLoop, ActiveEventLoop, EventLoopProxy};
use winit::application::ApplicationHandler;
use winit::window::{Window, WindowId};
use winit::event::WindowEvent;

enum Initializer {
    Init(Option<EventLoopProxy<Graphics>>),
    Ready(Game, Instant),
}

impl Initializer {
    fn new(event_loop: &EventLoop<Graphics>) -> Self {
        Self::Init(Some(event_loop.create_proxy()))
    }
}

impl ApplicationHandler<Graphics> for Initializer {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Self::Init(proxy) = self {
            if let Some(proxy) = proxy.take() {
                let mut win_attr = Window::default_attributes();
                win_attr = win_attr.with_title("Gabloctica");

                let window = event_loop.create_window(win_attr).expect("Create window error");
                pollster::block_on(Graphics::create(window, proxy));
            }
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, graphics: Graphics) {
        // Request a redraw now that graphics are ready
        graphics.request_redraw();
        *self = Self::Ready(Game::new(graphics), Instant::now());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        if let Self::Ready(game, last_time) = self {
            if let WindowEvent::RedrawRequested = event {
                game.update(last_time.elapsed().as_secs_f64());
                *last_time = Instant::now();
                game.draw()
            };

            let close = game.window_event(event);
            if close {
                event_loop.exit();
            }
        }
    }
}

fn main() {
    let event_loop = EventLoop::<Graphics>::with_user_event().build().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut initializer = Initializer::new(&event_loop);

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("error")).init();
    let _ = event_loop.run_app(&mut initializer);
}
