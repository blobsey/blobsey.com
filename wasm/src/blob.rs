// Blob constants
const BLOB_STIFFNESS: f32 = 0.5;
const BLOB_NUM_PARTICLES: usize = 64;
const BLOB_RADIUS: f32 = 100.0;

// Angular increment between vertices in radians (2π/n for regular n-gon)
const ANGLE_STEP: f32 = 2.0 * std::f32::consts::PI / BLOB_NUM_PARTICLES as f32;

use crate::constants::*;
use macroquad::{
    color::BLACK,
    math::Vec2,
    shapes::draw_line,
    window::{screen_height, screen_width},
};

/* The blob is made up of several particles, each connected to their neighbors
by springs */
pub struct Blob {
    particles: [Particle; BLOB_NUM_PARTICLES], // Fixed size, hope I won't regret this!
    springs: [Spring; BLOB_NUM_PARTICLES],
}

struct Particle {
    pos: Vec2,
    velocity: Vec2, // (speed, direction)
    mass: f32,
}

impl Particle {
    fn new(origin: Vec2, angle_radians: f32) -> Self {
        Particle {
            pos: Vec2::new(
                BLOB_RADIUS * angle_radians.cos() + origin.x,
                BLOB_RADIUS * angle_radians.sin() + origin.y,
            ),
            velocity: Vec2::new(0.0, 0.0),
            mass: 1.0,
        }
    }
}

struct Spring {
    particle_a: usize,
    particle_b: usize,
    rest_length: f32,
}

impl Blob {
    pub fn new(origin: Vec2) -> Blob {
        // Make a bunch of particles, BLOB_RADIUS distance away from origin
        let particles = std::array::from_fn(|i| Particle::new(origin, (i as f32) * ANGLE_STEP));

        let springs = std::array::from_fn(|i| {
            let next = (i + 1) % BLOB_NUM_PARTICLES; // wrap around for last particle
            let rest_length = (particles[i].pos - particles[next].pos).length();
            Spring {
                particle_a: i,
                particle_b: next,
                rest_length: rest_length,
            }
        });

        Blob {
            particles: particles,
            springs: springs,
        }
    }

    pub fn update(&mut self, dt: f32) {
        let screen_width = screen_width();
        let screen_height = screen_height();

        // Apply gravity to all particles' velocities
        for particle in &mut self.particles {
            particle.velocity.y += GRAVITY * dt;
        }

        // Apply spring forces to all particles
        for spring in &self.springs {
            let pos_a = self.particles[spring.particle_a].pos;
            let pos_b = self.particles[spring.particle_b].pos;
            let displacement = pos_a - pos_b;
            let distance = displacement.length();

            if distance > 0.0 { // Avoid division by zero
                let force_magnitude = BLOB_STIFFNESS * (distance - spring.rest_length);
                let force_direction = displacement / distance; // normalize
                let force = force_direction * force_magnitude;

                // Apply force to particle A (towards B)
                self.particles[spring.particle_a].velocity -=
                    force * dt / self.particles[spring.particle_a].mass;

                // Apply equal and opposite force to particle B (towards A)
                self.particles[spring.particle_b].velocity +=
                    force * dt / self.particles[spring.particle_b].mass;
            }
        }

        // Update particle positions based on velocities
        for particle in &mut self.particles {
            particle.pos += particle.velocity * dt;

            // Boundary collisions with damping
            if particle.pos.x < 0.0 {
                particle.pos.x = 0.0;
                particle.velocity.x = -particle.velocity.x * 0.1; // bounce with damping
            }
            if particle.pos.x > screen_width {
                particle.pos.x = screen_width;
                particle.velocity.x = -particle.velocity.x * 0.1;
            }
            if particle.pos.y < 0.0 {
                particle.pos.y = 0.0;
                particle.velocity.y = -particle.velocity.y * 0.1;
            }
            if particle.pos.y > screen_height {
                particle.pos.y = screen_height;
                particle.velocity.y = -particle.velocity.y * 0.1;
            }
        }
    }

    pub fn draw(&self) {
        for spring in &self.springs {
            let pos_a = self.particles[spring.particle_a].pos;
            let pos_b = self.particles[spring.particle_b].pos;

            draw_line(
                pos_a.x, pos_a.y, // start point
                pos_b.x, pos_b.y, // end point
                10.0,    // thickness
                BLACK,   // color
            );
        }
    }
}
