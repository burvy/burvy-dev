use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

use gloo_timers::future::TimeoutFuture;
use leptos::ev;
use leptos::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    const ZERO: Self = Self { x: 0.0, y: 0.0 };

    const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

const TICK_MS: u32 = 16;
const DT: f32 = 1.0 / 60.0;

const PLAYER_HALF_SIZE: Vec2 = Vec2::new(25.0, 25.0);

const GRAVITY: f32 = 2000.0;
const MOVE_FORCE: f32 = 8000.0;
const JUMP_SPEED: f32 = 850.0;
const MAX_RUN_SPEED: f32 = 400.0;

const GROUND_DAMPING: f32 = 0.80;
const STOP_EPSILON: f32 = 1.0;
const CONTACT_SLOP: f32 = 0.001;

#[derive(Clone, Copy, Debug)]
struct Aabb {
    min: Vec2,
    max: Vec2,
}

impl Aabb {
    fn from_corners(a: Vec2, b: Vec2) -> Self {
        Self {
            min: Vec2::new(a.x.min(b.x), a.y.min(b.y)),
            max: Vec2::new(a.x.max(b.x), a.y.max(b.y)),
        }
    }

    fn from_center_radius(center: Vec2, radius: Vec2) -> Self {
        Self {
            min: center - radius,
            max: center + radius,
        }
    }

    fn width(self) -> f32 {
        self.max.x - self.min.x
    }

    fn height(self) -> f32 {
        self.max.y - self.min.y
    }
}

#[derive(Clone, Copy, Debug)]
struct Contact {
    normal: Vec2,
    depth: f32,
}

#[derive(Clone, Copy, Debug)]
struct Body {
    pos: Vec2,
    vel: Vec2,
    force: Vec2,
    half_size: Vec2,
    mass: f32,
    grounded: bool,
}

impl Body {
    fn new(pos: Vec2, half_size: Vec2) -> Self {
        Self {
            pos,
            vel: Vec2::ZERO,
            force: Vec2::ZERO,
            half_size,
            mass: 1.0,
            grounded: false,
        }
    }

    fn aabb(self) -> Aabb {
        Aabb::from_center_radius(self.pos, self.half_size)
    }

    fn apply_force(&mut self, force: Vec2) {
        self.force += force;
    }

    fn jump(&mut self) {
        if self.grounded {
            self.vel.y = -JUMP_SPEED;
            self.grounded = false;
        }
    }

    fn integrate_forces(&mut self, dt: f32) {
        let acc = self.force * (1.0 / self.mass);

        self.vel += acc * dt;
        self.force = Vec2::ZERO;
    }

    fn integrate_velocity(&mut self, dt: f32) {
        self.pos += self.vel * dt;
    }

    fn resolve_contact(mut self, contact: Contact) -> Self {
        self.pos += contact.normal * (contact.depth + CONTACT_SLOP);

        let speed_into_surface = self.vel.dot(contact.normal);

        if speed_into_surface < 0.0 {
            self.vel -= contact.normal * speed_into_surface;
        }

        if contact.normal.y < -0.5 {
            self.grounded = true;
        }

        self
    }
}

fn contact_between(a: Aabb, b: Aabb) -> Option<Contact> {
    let overlaps =
        a.max.x > b.min.x &&
        a.min.x < b.max.x &&
        a.max.y > b.min.y &&
        a.min.y < b.max.y;

    overlaps.then(|| {
        [
            Contact {
                normal: Vec2::new(-1.0, 0.0),
                depth: a.max.x - b.min.x,
            },
            Contact {
                normal: Vec2::new(1.0, 0.0),
                depth: b.max.x - a.min.x,
            },
            Contact {
                normal: Vec2::new(0.0, -1.0),
                depth: a.max.y - b.min.y,
            },
            Contact {
                normal: Vec2::new(0.0, 1.0),
                depth: b.max.y - a.min.y,
            },
        ]
        .into_iter()
        .min_by(|a, b| a.depth.total_cmp(&b.depth))
        .expect("contacts array should not be empty")
    })
}

fn resolve_collisions(body: Body, colliders: &[Aabb]) -> Body {
    colliders
        .iter()
        .fold(Body { grounded: false, ..body }, |body, collider| {
            contact_between(body.aabb(), *collider)
                .map(|contact| body.resolve_contact(contact))
                .unwrap_or(body)
        })
}

fn step_body(mut body: Body, colliders: &[Aabb], xinput: f32) -> Body {
    body.force = Vec2::ZERO;

    // player input force
    body.apply_force(Vec2::new(xinput * MOVE_FORCE, 0.0));
    // gravitational force
    body.apply_force(Vec2::new(0.0, body.mass * GRAVITY));

    if body.grounded && xinput == 0.0 {
        body.vel.x *= GROUND_DAMPING;

        if body.vel.x.abs() < STOP_EPSILON {
            body.vel.x = 0.0;
        }
    }

    body.integrate_forces(DT);

    body.vel.x = body.vel.x.clamp(-MAX_RUN_SPEED, MAX_RUN_SPEED);

    body.integrate_velocity(DT);

    resolve_collisions(body, colliders)
}

#[component]
pub fn GamePage() -> impl IntoView {
    let (body, set_body) = signal(Body::new(
        Vec2::new(100.0, 100.0),
        PLAYER_HALF_SIZE,
    ));
    // button toggle
    let (toggle, set_toggle) = signal(false);
    let (lp, set_lp) = signal(false);
    let (rp, set_rp) = signal(false);

    let (colliders, _set_colliders) = signal(vec![
        Aabb::from_corners(
            Vec2::new(0.0, 500.0),
            Vec2::new(500.0, 550.0),
        ),
        Aabb::from_corners(
            Vec2::new(650.0, 350.0),
            Vec2::new(900.0, 400.0),
        ),
        Aabb::from_corners(
            Vec2::new(700.0, 350.0),
            Vec2::new(750.0, 300.0),
        ),
    ]);

    window_event_listener(ev::keydown, move |ev| {
        if ev.code() == "Space" && !ev.repeat() {
            ev.prevent_default();

            set_body.update(|body| {
                body.jump();
            });
        }
    });

    window_event_listener(ev::keydown, move |ev| {
        match ev.code().as_str() {
            "KeyA" => set_lp.set(true),
            "KeyD" => set_rp.set(true),
            _ => {}
        }
    });

    window_event_listener(ev::keyup, move |ev| {
        match ev.code().as_str() {
            "KeyA" => set_lp.set(false),
            "KeyD" => set_rp.set(false),
            _ => {}
        }
    });

    leptos::task::spawn_local(async move {
        loop {
            TimeoutFuture::new(TICK_MS).await;

            let xinput = (rp.get() as i32 - lp.get() as i32) as f32;
            let colliders = colliders.get();

            set_body.update(|body| {
                *body = step_body(*body, &colliders, xinput);
            });
        }
    });

    view! {
        <For
            each=move || colliders.get()
            key=|collider| (
                collider.min.x.to_bits(),
                collider.min.y.to_bits(),
                collider.max.x.to_bits(),
                collider.max.y.to_bits(),
            )
            let(collider)
        >
            <div
                class="grass"
                style:position="absolute"
                style:height=move || format!("{}px", collider.height())
                style:width=move || format!("{}px", collider.width())
                style:top=move || format!("{}px", collider.min.y)
                style:left=move || format!("{}px", collider.min.x)
            >
                67
            </div>
        </For>

        <button
            type="button"
            class="player"
            style:position="absolute"
            class:player-green=move || toggle.get()
            class:player-red=move || !toggle.get()
            style:top=move || { format!("{}px", body.get().pos.y - PLAYER_HALF_SIZE.x) }
            style:left=move || { format!("{}px", body.get().pos.x - PLAYER_HALF_SIZE.y) }
            style:width="50px"
            style:height="50px"
            on:click=move |_| set_toggle.set(!toggle.get())
        >
            67
        </button>
    }
}
