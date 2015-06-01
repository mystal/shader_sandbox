extern crate clap;
extern crate conrod;
extern crate fps_counter;
#[macro_use]
extern crate gfx;
extern crate gfx_graphics;
extern crate graphics;
extern crate piston;
extern crate piston_window;
extern crate sdl2_window;

use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::marker::PhantomData;
use std::path::Path;
use std::rc::Rc;

use clap::App;
use conrod::{
    Colorable,
    Label,
    Positionable,
    Theme,
    Ui,
    Widget,
    WidgetId,
};
use conrod::color::white;
use fps_counter::FPSCounter;
use gfx::traits::*;
use gfx_graphics::GlyphCache;
use piston_window::*;
use piston::event::*;
use piston::input::*;
use piston::window::{ AdvancedWindow, Window, WindowSettings };
use sdl2_window::{ Sdl2Window, OpenGL };

const SCREEN_SIZE: [u32; 2] = [640, 480];

gfx_parameters!( MandelbrotShaderParams {
    screenSize@ screen_size: [f32; 2],
    iterations@ iterations: i32,
});

gfx_parameters!( ShadertoyShaderParams {
    iGlobalTime@ global_time: f32,
    iResolution@ resolution: [f32; 3],
    iMouse@ mouse_state: [f32; 4],
});

gfx_vertex!( Vertex {
    a_Pos@ pos: [f32; 2],
});

const FPS: WidgetId = 0;
const TIMER: WidgetId = 1;

struct UiData {
    fps: usize,
    play: bool,
    mouse_button_held: bool,
    mouse_position: [f64; 2],
}

fn main() {
    let args =
        App::new("shader_sandbox")
                 .args_from_usage(
                     "-s --shadertoy 'Treat provided shader as Shadertoy would.'
                     <shader_file> 'The shader to run.'")
                 .get_matches();

    let window = Rc::new(RefCell::new(Sdl2Window::new(
        OpenGL::_3_2,
        WindowSettings::new("Shader Sandbox", SCREEN_SIZE)
        .exit_on_esc(true)
        .samples(4)
    )));

    let events = PistonWindow::new(window, empty_app());

    let ref mut factory = events.factory.borrow().clone();

    let vertex_file = "src/simple.vs";
    let fragment_file = args.value_of("shader_file").unwrap();

    let mut vertex_source = String::new();
    let mut fragment_source = String::new();

    File::open(vertex_file).unwrap().read_to_string(&mut vertex_source);
    File::open(fragment_file).unwrap().read_to_string(&mut fragment_source);

    let fragment_source = format!(
        "uniform float iGlobalTime;
        uniform vec3 iResolution;
        uniform vec4 iMouse;

        {}

        void main() {{
            mainImage(gl_FragColor, gl_FragCoord.xy);
        }}", fragment_source);

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
    let mut params = ShadertoyShaderParams {
        global_time: 0.0,
        resolution: [0.0, 0.0, 0.0],
        mouse_state: [0.0, 0.0, 0.0, 0.0],
        _r: PhantomData,
    };

    //let params = MandelbrotShaderParams {
    //    screen_size: [SCREEN_SIZE[0] as f32, SCREEN_SIZE[1] as f32],
    //    iterations: 1000,
    //    _r: PhantomData,
    //};

    let mut ui_data = UiData {
        fps: 0,
        play: true,
        mouse_button_held: false,
        mouse_position: [0.0, 0.0],
    };

    let mut fps_counter = FPSCounter::new();

    let glyph_cache = {
        let font_path = Path::new("assets/VeraMono.ttf");
        GlyphCache::new(&font_path, factory.clone()).unwrap()
    };
    let mut ui  = Ui::new(glyph_cache, Theme::default());

    //let draw_ui = |c, g, ui: &mut Ui<GlyphCache<_, _>>, ui_data| {
    //};

    for e in events {
        if let Some(button) = e.press_args() {
            match button {
                Button::Keyboard(key) => {
                    match key {
                        keyboard::Key::Space => ui_data.play = !ui_data.play,
                        _ => (),
                    }
                },
                Button::Mouse(button) => {
                    match button {
                        mouse::MouseButton::Left => ui_data.mouse_button_held = true,
                        _ => (),
                    }
                },
            }
        }
        if let Some(button) = e.release_args() {
            match button {
                Button::Keyboard(key) => {
                },
                Button::Mouse(button) => {
                    match button {
                        mouse::MouseButton::Left => ui_data.mouse_button_held = false,
                        _ => (),
                    }
                },
            }
        }
        if let Some(mouse_position) = e.mouse_cursor_args() {
            ui_data.mouse_position = mouse_position;
        }
        if ui_data.mouse_button_held {
            params.mouse_state[0] = ui_data.mouse_position[0] as f32;
            params.mouse_state[1] = ui_data.mouse_position[1] as f32;
        }

        if let Some(args) = e.update_args() {
            if ui_data.play {
                params.global_time += args.dt as f32;
            }
        }
        if let Some(_) = e.render_args() {
            let size = e.size();
            params.resolution[0] = size.width as f32;
            params.resolution[1] = size.height as f32;
            e.draw_3d(|stream| {
                stream.clear(
                    gfx::ClearData {
                        color: [0.0, 0.0, 0.0, 1.0],
                        depth: 1.0,
                        stencil: 0,
                    }
                );
                stream.draw(&(&mesh, slice.clone(), &program, &params, &state)).unwrap();
            });
            ui_data.fps = fps_counter.tick();
            e.draw_2d(|context, g| {
                Label::new(&format!("{}", ui_data.fps))
                    .xy(180.0, 180.0)
                    .font_size(32)
                    .color(white())
                    .set(FPS, &mut ui);
                ui.draw(context, g);
                Label::new(&format!("{}", params.global_time as i32))
                    .xy(-180.0, 180.0)
                    .font_size(32)
                    .color(white())
                    .set(TIMER, &mut ui);
                ui.draw(context, g);
            });
        }
    }
}
