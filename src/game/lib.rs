#![feature(slicing_syntax)]
#![allow(unstable)]

extern crate gfx;
extern crate wad;
extern crate math;

extern crate getopts;
#[macro_use]
extern crate gl;
extern crate libc;

#[macro_use]
extern crate log;
extern crate sdl2;
extern crate time;

use ctrl::GameController;
use ctrl::Gesture;
use gfx::ShaderLoader;
use level::Level;
use libc::c_void;
use math::{Mat4, Vec3};
use player::Player;
use sdl2::scancode::ScanCode;
use sdl2::video::GLAttr;
use sdl2::video::WindowPos;
use std::default::Default;
use wad::TextureDirectory;

pub mod camera;
pub mod ctrl;
pub mod player;
pub mod level;


const WINDOW_TITLE: &'static str = "Rusty Doom v0.0.7 - Toggle mouse with \
                                    backtick key (`))";
const OPENGL_DEPTH_SIZE: isize = 24;
const SHADER_ROOT: &'static str = "src/shaders";


pub struct MainWindow {
    window: sdl2::video::Window,
    _context: sdl2::video::GLContext,
}
impl MainWindow {
    pub fn new(width: usize, height: usize) -> MainWindow {
        sdl2::video::gl_set_attribute(GLAttr::GLContextMajorVersion,
                                      gl::platform::GL_MAJOR_VERSION);
        sdl2::video::gl_set_attribute(GLAttr::GLContextMinorVersion,
                                      gl::platform::GL_MINOR_VERSION);
        sdl2::video::gl_set_attribute(GLAttr::GLDepthSize, OPENGL_DEPTH_SIZE);
        sdl2::video::gl_set_attribute(GLAttr::GLDoubleBuffer, 1);
        sdl2::video::gl_set_attribute(
            GLAttr::GLContextProfileMask,
            sdl2::video::ll::SDL_GLprofile::SDL_GL_CONTEXT_PROFILE_CORE as isize);

        let window = sdl2::video::Window::new(
            WINDOW_TITLE, WindowPos::PosCentered, WindowPos::PosCentered,
            width as isize, height as isize,
            sdl2::video::OPENGL | sdl2::video::SHOWN).unwrap();

        let context = window.gl_create_context().unwrap();
        sdl2::clear_error();
        gl::load_with(|name| {
            match sdl2::video::gl_get_proc_address(name) {
                Some(glproc) => glproc as *const libc::c_void,
                None => {
                    warn!("missing GL function: {}", name);
                    std::ptr::null()
                }
            }
        });
        let mut vao_id = 0;
        check_gl_unsafe!(gl::GenVertexArrays(1, &mut vao_id));
        check_gl_unsafe!(gl::BindVertexArray(vao_id));
        MainWindow {
           window: window,
            _context: context,
        }
    }

    pub fn aspect_ratio(&self) -> f32 {
        let (w, h) = self.window.get_size();
        w as f32 / h as f32
    }

    pub fn swap_buffers(&self) {
        self.window.gl_swap_window();
    }
}

pub struct GameConfig<'a> {
    wad: &'a str,
    metadata: &'a str,
    level_index: usize,
    fov: f32,
}

pub struct Game {
    window: MainWindow,
    player: Player,
    level: Level,
}
impl Game {
    pub fn new(window: MainWindow, config: GameConfig) -> Game {
        let mut wad = wad::Archive::open(&Path::new(config.wad),
                                         &Path::new(config.metadata)).unwrap();
        let textures = TextureDirectory::from_archive(&mut wad).unwrap();
        let shader_loader = ShaderLoader::new(gl::platform::GLSL_VERSION_STRING,
                                              Path::new(SHADER_ROOT));
        let level = Level::new(&shader_loader,
                               &mut wad, &textures, config.level_index);

        check_gl_unsafe!(gl::ClearColor(0.06, 0.07, 0.09, 0.0));
        check_gl_unsafe!(gl::Enable(gl::DEPTH_TEST));
        check_gl_unsafe!(gl::DepthFunc(gl::LESS));

        let start = *level.get_start_pos();
        let mut player = Player::new(config.fov, window.aspect_ratio(),
                                     Default::default());
        player.set_position(&Vec3::new(start.x, 0.3, start.y));

        Game {
            window: window,
            player: player,
            level: level
        }
    }

    pub fn run(&mut self) {
        let quit_gesture = Gesture::AnyOf(
            vec![Gesture::QuitTrigger,
                 Gesture::KeyTrigger(ScanCode::Escape)]);
        let grab_toggle_gesture = Gesture::KeyTrigger(ScanCode::Grave);

        let mut cum_time = 0.0;
        let mut cum_updates_time = 0.0;
        let mut num_frames = 0.0;
        let mut t0 = 0.0;
        let mut control = GameController::new();
        let mut mouse_grabbed = true;
        loop {
            check_gl_unsafe!(
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));
            let t1 = time::precise_time_s();
            let mut delta = (t1 - t0) as f32;
            if delta < 1e-10 { delta = 1.0 / 60.0; }
            let delta = delta;
            t0 = t1;

            let updates_t0 = time::precise_time_s();

            control.update();
            if control.poll_gesture(&quit_gesture) {
                break;
            } else if control.poll_gesture(&grab_toggle_gesture) {
                mouse_grabbed = !mouse_grabbed;
                control.set_mouse_enabled(mouse_grabbed);
                control.set_cursor_grabbed(mouse_grabbed);
            }

            self.player.update(delta, &control);
            self.level.render(
                delta,
                &self.player.get_camera()
                .multiply_transform(&Mat4::new_identity()));

            let updates_t1 = time::precise_time_s();
            cum_updates_time += updates_t1 - updates_t0;

            cum_time += delta as f64;
            num_frames += 1.0 as f64;
            if cum_time > 2.0 {
                let fps = num_frames / cum_time;
                let cpums = 1000.0 * cum_updates_time / num_frames as f64;
                info!("Frame time: {:.2}ms ({:.2}ms cpu, FPS: {:.2})",
                      1000.0 / fps, cpums, fps);
                cum_time = 0.0;
                cum_updates_time = 0.0;
                num_frames = 0.0;
            }

            self.window.swap_buffers();
        }
    }
}


#[cfg(not(test))]
pub fn run() {
    use getopts::{optopt,optflag,getopts, usage};
    use std::os;

    let args: Vec<String> = os::args();
    let opts = [
        optopt("i", "iwad",
               "set initial wad file to use wad [default='doom1.wad']", "FILE"),
        optopt("m", "metadata",
               "path to toml toml metadata file [default='doom.toml']", "FILE"),
        optopt("l", "level",
               "the index of the level to render [default=0]", "N"),
        optopt("f", "fov",
               "horizontal field of view to please TotalHalibut [default=65]",
               "FLOAT"),
        optopt("r", "resolution",
               "the resolution at which to render the game [default=1280x720]",
               "WIDTHxHEIGHT"),
        optflag("d", "dump-levels", "list all levels and exit."),
        optflag("", "load-all", "loads all levels and exit; for debugging"),
        optflag("h", "help", "print this help message and exit"),
    ];

    let matches = match getopts(args.tail(), &opts) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    let wad_filename = matches
        .opt_str("i")
        .unwrap_or("doom1.wad".to_string());
    let meta_filename = matches
        .opt_str("m")
        .unwrap_or("doom.toml".to_string());
    let (width, height) = matches
        .opt_str("r")
        .map(|r| {
            let r: Vec<&str> = r.splitn(1, 'x').collect();
            match r.as_slice() {
                [w, h] => (w.parse().unwrap(), h.parse().unwrap()),
                _ => {
                    panic!("Invalid format for resolution, \
                            please use WIDTHxHEIGHT.");
                }
            }
        })
        .unwrap_or((1280, 720));
    let level_index = matches
        .opt_str("l")
        .map(|l| l[].parse()
                    .expect("Invalid value for --level. Expected integer."))
        .unwrap_or(0);
    let fov = matches
        .opt_str("f")
        .map(|f| f[].parse()
                    .expect("Invalid value for --fov. Expected float."))
        .unwrap_or(65.0);

    if matches.opt_present("h") {
        println!("{}", usage("A rust doom renderer.", &opts));
        return;
    }

    if matches.opt_present("d") {
        let wad = wad::Archive::open(
            &Path::new(&wad_filename[]), &Path::new(&meta_filename[])).unwrap();
        for i_level in range(0, wad.num_levels()) {
            println!("{:3} {:8}", i_level, wad.get_level_name(i_level));
        }
        return;
    }

    if matches.opt_present("load-all") {
        if !sdl2::init(sdl2::INIT_VIDEO) {
            panic!("main: sdl video init failed.");
        }
        let _win = MainWindow::new(width, height);
        let t0 = time::precise_time_s();
        let mut wad = wad::Archive::open(
            &Path::new(&wad_filename[]), &Path::new(&meta_filename[])).unwrap();
        let textures = TextureDirectory::from_archive(&mut wad).unwrap();
        for level_index in range(0, wad.num_levels()) {
            let shader_loader = ShaderLoader::new(
                gl::platform::GLSL_VERSION_STRING, Path::new(SHADER_ROOT));
            Level::new(&shader_loader, &mut wad, &textures, level_index);
        }
        println!("Done, loaded all levels in {:.4}s. Shutting down...",
                 time::precise_time_s() - t0);
        sdl2::quit();
        return;
    }

    if !sdl2::init(sdl2::INIT_VIDEO) { panic!("main: sdl video init failed."); }

    let mut game = Game::new(
        MainWindow::new(width, height),
        GameConfig {
            wad: &wad_filename[],
            metadata: &meta_filename[],
            level_index: level_index,
            fov: fov,
        });
    game.run();

    info!("Shutting down.");
    drop(game);
    sdl2::quit();
}
