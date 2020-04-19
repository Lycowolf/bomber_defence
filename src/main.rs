// Bomber defence: unofficial Ludum Dare (Keep it Alive) 46 game

#![windows_subsystem = "windows"]
extern crate quicksilver;

use quicksilver::{
    Result,
    geom::{Circle, Rectangle, Vector, Shape},
    graphics::{Background::Col, Color},
    lifecycle::{Settings, State, Window, Event, run},
    input::{MouseButton, Key},
};
use rand::{thread_rng, Rng};
use std::mem::swap;
use std::cmp::max;

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;

const CANNON_HEIGHT: f32 = 100.0;
const CANNON_POWER: f32 = 10.0;
const CANNON_COOLDOWN: i32 = 12;

const PROJECTILE_RADIUS: f32 = 5.0;

const BOMB_DELAY_MIN: i32 = 20;
const BOMB_DELAY_MAX: i32 = 50;
const BOMB_SPEED_MAX: f32 = 2.0;

const EXPLOSION_DURATION: f32 = 30.0;
const EXPLOSION_RADIUS: f32 = 60.0;
const EXPLOSION_ALPHA_FRACTION: f32 = 2.0;

const VAULT_RADIUS: f32 = 30.0;
const DESTROYED_VAULT_RADIUS: f32 = 40.0;
const VAULT_DEPTH: f32 = 50.0;
const NUM_VAULTS: usize = 4;

const GRAVITY: f32 = 0.12;

fn cannon_position() -> Vector {
    Vector::new(WIDTH / 2.0, HEIGHT - CANNON_HEIGHT)
}

fn bomb_at(position: Vector) -> Rectangle {
    let bomb_shape = Rectangle::new((0, 0), (32, 12));
    bomb_shape.with_center(position)
}

fn explosion_size(time: f32) -> f32 {
    EXPLOSION_RADIUS * (EXPLOSION_DURATION - time) / EXPLOSION_DURATION
}

fn vault_position(index: usize) -> Vector {
    let position_x = (WIDTH / (NUM_VAULTS as f32 + 1.0)) * (index as f32 + 1.0); // we have spacers before each vault and then one at the end
    Vector::new(position_x, HEIGHT - VAULT_DEPTH)
}

#[derive(Debug, Clone, Copy)]
struct Projectile {
    position: Vector,
    velocity: Vector,
}

impl Projectile {
    fn new(aim_vector: Vector) -> Self {
        let velocity = aim_vector.normalize() * CANNON_POWER;
        let position = cannon_position() + velocity; // don't hit self on first tick
        Self { position, velocity }
    }
}

#[derive(Debug, Clone, Copy)]
struct Bomb {
    position: Vector,
    velocity: Vector,
}

impl Bomb {
    fn new(dropped_at: Vector, vel: Vector) -> Self {
        let position = dropped_at + vel; // don't hit self on first tick
        let velocity = vel;
        Self { position, velocity }
    }
}

#[derive(Debug, Clone, Copy)]
struct Explosion {
    position: Vector,
    timer: f32,
}

impl Explosion {
    fn new(position: Vector) -> Self {
        Self { position, timer: EXPLOSION_DURATION }
    }
}

struct Game {
    cannon_timer: i32,
    bomb_timer: i32,
    vaults: [bool; NUM_VAULTS],
    aim_vector: Vector,
    // normalized
    bombs: Vec<Bomb>,
    projectiles: Vec<Projectile>,
    explosions: Vec<Explosion>,
}

impl State for Game {
    fn new() -> Result<Game> {
        Ok(Game {
            cannon_timer: 0,
            bomb_timer: 0,
            vaults: Default::default(),
            aim_vector: Vector::new(0, 1),
            bombs: Vec::new(),
            projectiles: Vec::new(),
            explosions: Vec::new(),
        })
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::from_rgba(180, 200, 255, 1.0))?;

        // ground
        window.draw(&Rectangle::new((0, HEIGHT - CANNON_HEIGHT), (WIDTH, CANNON_HEIGHT)),
                    Col(Color::from_rgba(150, 60, 20, 1.0)),
        );
        // cannon
        window.draw(&Circle::new(cannon_position(), 10.0), Col(Color::BLACK));
        // vaults
        for (i, destroyed) in self.vaults.iter().enumerate() {
            if *destroyed {
                window.draw(&Circle::new(vault_position(i), DESTROYED_VAULT_RADIUS), Col(Color::BLACK));
            } else {
                window.draw(&Circle::new(vault_position(i), VAULT_RADIUS), Col(Color::GREEN));
            }
        }

        // projectiles
        for projectile in &mut self.projectiles {
            window.draw(&Circle::new(projectile.position, PROJECTILE_RADIUS), Col(Color::RED));
        }

        // bombs
        for bomb in &mut self.bombs {
            window.draw(&bomb_at(bomb.position), Col(Color::BLUE));
        }

        // explosions
        for explosion in &self.explosions {
            let alpha = 1.0 - ((EXPLOSION_DURATION - explosion.timer) / (EXPLOSION_DURATION * EXPLOSION_ALPHA_FRACTION));
            window.draw(
                &Circle::new(explosion.position, explosion_size(explosion.timer)),
                Col(Color::from_rgba(255, 200, 80, alpha)));
        }

        Ok(())
    }

    fn update(&mut self, window: &mut Window) -> Result<()> {
        // game over; stop game
        if self.vaults.iter().all(|destroyed| *destroyed) {return Ok(())}

        // autofire
        self.cannon_timer = max(0, self.cannon_timer - 1);
        if (window.keyboard()[Key::Space].is_down() || window.mouse()[MouseButton::Left].is_down())
            && self.cannon_timer == 0 {
            self.cannon_timer = CANNON_COOLDOWN;
            self.projectiles.push(Projectile::new(self.aim_vector));
        }

        let y_fudge: f32 = 1000.0; // skybox extends this much above the screen
        let air_box = Rectangle::new((0, -y_fudge), (WIDTH, HEIGHT - CANNON_HEIGHT + y_fudge));

        // projectiles
        for projectile in &mut self.projectiles {
            projectile.velocity += Vector::new(0.0, GRAVITY);
            projectile.position += projectile.velocity;
        }
        let keep_projectiles: Vec<Projectile> = self.projectiles.clone().into_iter()
            .filter(|&p| { air_box.contains(p.position) })
            .collect();
        self.projectiles = keep_projectiles;

        // bombs
        self.bomb_timer -= 1;
        if self.bomb_timer <= 0 {
            self.bomb_timer = thread_rng().gen_range(BOMB_DELAY_MIN, BOMB_DELAY_MAX);
            let position_x = thread_rng().gen_range(0, WIDTH as i32);
            self.bombs.push(Bomb::new(Vector::new(position_x, 10), Vector::ZERO));
        }
        for bomb in &mut self.bombs {
            bomb.velocity += Vector::new(0.0, GRAVITY);
            if bomb.velocity.len2() > BOMB_SPEED_MAX.powi(2) { bomb.velocity = bomb.velocity.normalize() * BOMB_SPEED_MAX };
            bomb.position += bomb.velocity;
        }
        let mut keep_bombs = Vec::new();
        for bomb in &self.bombs {
            if air_box.contains(bomb.position) {
                keep_bombs.push(*bomb)
            } else {
                self.explosions.push(Explosion::new(bomb.position))
            }
        }
        self.bombs = keep_bombs;

        // explosions
        let mut keep_explosions = Vec::new();
        for explosion in &mut self.explosions {
            explosion.timer -= 1.0;
            if explosion.timer >= 0.0 {
                keep_explosions.push(*explosion);
            }
        }
        self.explosions = keep_explosions;

        // chain explosions
        let mut keep_bombs = Vec::new();
        let mut new_explosions = Vec::new();
        'chain: for bomb in &self.bombs {
            for explosion in &self.explosions {
                if Circle::new(explosion.position, explosion_size(explosion.timer)).overlaps(&bomb_at(bomb.position)) {
                    new_explosions.push(Explosion::new(bomb.position));
                    continue 'chain;
                }
            }
            keep_bombs.push(*bomb)
        }
        self.explosions.extend(new_explosions);
        self.bombs = keep_bombs;

        // vault-destroying explosions
        let mut new_explosions = Vec::new();
        for explosion in &self.explosions {
            for (i, destroyed) in self.vaults.iter_mut().enumerate() {
                if !(*destroyed) && Circle::new(explosion.position, explosion_size(explosion.timer))
                    .overlaps(&Circle::new(vault_position(i), VAULT_RADIUS)) {
                    swap(destroyed, &mut true);
                    new_explosions.push(Explosion::new(vault_position(i)));
                }
            }
        }
        self.explosions.extend(new_explosions);

        // projectile collisions
        // expectation: one projectile hits at most one bomb
        let mut keep_projectiles = Vec::new();
        for projectile in &self.projectiles {
            let mut remove_bomb: Option<usize> = None;
            for (i, bomb) in self.bombs.iter().enumerate() {
                if Circle::new(projectile.position, PROJECTILE_RADIUS).overlaps(&bomb_at(bomb.position)) {
                    self.explosions.push(Explosion::new(bomb.position));
                    remove_bomb = Some(i);
                    break;
                }
            }
            if let Some(i) = remove_bomb {
                self.bombs.remove(i);
            } else {
                keep_projectiles.push(*projectile);
            }
        }
        self.projectiles = keep_projectiles;

        Ok(())
    }

    fn event(&mut self, event: &Event, _window: &mut Window) -> Result<()> {
        match event {
            Event::MouseMoved(pointer) => {
                self.aim_vector = (*pointer - cannon_position()).normalize();
            }
            _ => {}
        }
        Ok(())
    }
}

fn main() {
    run::<Game>("Bomber Defence", Vector::new(WIDTH, HEIGHT), Settings::default());
}