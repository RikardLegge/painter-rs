extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use std::f64::consts;
use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };

struct Point {
    x: i32,
    y: i32
}

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    rotation: f64,   // Rotation for the square.
    position: Point,   // Rotation for the square
}

impl App {
    fn render(&mut self, args: &RenderArgs, window: &Window) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED:   [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let square = rectangle::square(0.0, 0.0, 100.0);
        let rotation = self.rotation;
        let x = -self.position.x as f64 + 400.0;
        let y = -self.position.y as f64 + 400.0;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(GREEN, gl);

            let transform = c.transform
                .trans(x, y)
                .rot_rad(rotation)
                .trans(-50.0, -50.0);

            // Draw a box rotating around the middle of the screen.
            rectangle(RED, square, transform, gl);
        });

        window.window.set_position(self.position.x, self.position.y);
    }

    fn update(&mut self, args: &UpdateArgs) {
        // Rotate 2 radians per second.
        self.rotation += consts::PI * args.dt;

        let x = self.rotation.cos()*100.0;
        let y = self.rotation.sin()*100.0;

        self.position.x = x as i32 + 200;
        self.position.y = y as i32 + 200;
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new(
        "spinning-square",
        [400, 400]
    )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        rotation: 0.0,
        position: Point {x: 0, y: 0}
    };

    let mut events = window.events();
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            app.render(&r, &window);
        }

        if let Some(u) = e.update_args() {
            app.update(&u);
        }
    }
}