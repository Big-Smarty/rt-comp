#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

extern crate chrono;
extern crate log;
extern crate simplelog;

use vulkano::memory::allocator::StandardMemoryAllocator;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use chrono::offset::Utc;
use chrono::DateTime;
use simplelog::*;
use std::time::SystemTime;

use std::fs::File;

pub mod element;
pub mod renderer;
pub mod shaders;
pub mod signal_handler;

pub struct Engine {
    pub renderer: renderer::Renderer,
    pub signal_handler: signal_handler::SignalHandler,
    elements: Vec<element::Element>,
}

impl Engine {
    pub fn new() -> Engine {
        CombinedLogger::init(vec![
            TermLogger::new(
                LevelFilter::Info,
                Config::default(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            ),
            WriteLogger::new(
                LevelFilter::Trace,
                Config::default(),
                File::create(format!(
                    "bs-rt-{}",
                    DateTime::<Utc>::from(SystemTime::now()).format("%H:%M:%S")
                ))
                .unwrap(),
            ),
        ])
        .unwrap();
        Engine {
            renderer: renderer::Renderer::new(),
            signal_handler: signal_handler::SignalHandler::new(),
            elements: Vec::new(),
        }
    }
    pub fn run(self) {
        self.renderer
            .context
            .event_loop
            .run(|event, _, control_flow| match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => (),
            })
    }
    pub fn allocator(&self) -> std::sync::Arc<StandardMemoryAllocator> {
        self.renderer.context.allocator.clone()
    }
    pub fn add_element(&mut self, element: element::Element) {
        self.elements.push(element);
    }
}
