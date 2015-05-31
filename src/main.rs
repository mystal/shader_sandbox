extern crate conrod;
extern crate fps_counter;
#[macro_use]
extern crate gfx;
extern crate piston;
extern crate piston_window;
extern crate sdl2_window;

use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::marker::PhantomData;
use std::rc::Rc;

use fps_counter::FPSCounter;
use gfx::traits::*;
use piston_window::*;
//use piston::event::*;
use piston::window::{ AdvancedWindow, WindowSettings };
use sdl2_window::{ Sdl2Window, OpenGL };

const SCREEN_SIZE: [u32; 2] = [640, 480];

gfx_parameters!( ShaderParams {
    screenSize@ screen_size: [f32; 2],
});

gfx_vertex!( Vertex {
    a_Pos@ pos: [f32; 2],
});

fn main() {
    let window = Rc::new(RefCell::new(Sdl2Window::new(
        OpenGL::_3_2,
        WindowSettings::new("piston-example-gfx_cube", SCREEN_SIZE)
        .exit_on_esc(true)
        .samples(4)
    ).capture_cursor(true)));

    let events = PistonWindow::new(window, empty_app());

    let ref mut factory = events.factory.borrow().clone();

    let mut vertex_source = String::new();
    let mut fragment_source = String::new();

    File::open("src/simple.vs").unwrap().read_to_string(&mut vertex_source);
    File::open("src/mandelbrot.fs").unwrap().read_to_string(&mut fragment_source);

    let program = {
        let vertex = gfx::ShaderSource {
            glsl_150: Some(vertex_source.as_bytes()),
            .. gfx::ShaderSource::empty()
        };
        let fragment = gfx::ShaderSource {
            glsl_150: Some(fragment_source.as_bytes()),
            .. gfx::ShaderSource::empty()
        };
        factory.link_program_source(vertex, fragment).unwrap()
    };

    let vertex_data = [
        Vertex { pos: [-1.0, -1.0] },
        Vertex { pos: [1.0, -1.0] },
        Vertex { pos: [1.0, 1.0] },
        Vertex { pos: [-1.0, 1.0] },
    ];
    let mesh = factory.create_mesh(&vertex_data);
    let slice = mesh.to_slice(gfx::PrimitiveType::TriangleFan);

    let state = gfx::DrawState::new();
    let params = ShaderParams {
        screen_size: [SCREEN_SIZE[0] as f32, SCREEN_SIZE[1] as f32],
        _r: PhantomData,
    };

    let mut fps_counter = FPSCounter::new();

    for e in events {
        e.draw_3d(|stream| {
            //let args = e.render_args().unwrap();
            stream.clear(
                gfx::ClearData {
                    color: [0.0, 0.0, 0.0, 1.0],
                    depth: 1.0,
                    stencil: 0,
                }
            );
            stream.draw(&(&mesh, slice.clone(), &program, &params, &state)).unwrap();
            let frames = fps_counter.tick();
            println!("FPS: {}", frames);
        });
        //e.draw_2d(|stream| {
        //});
    }
}