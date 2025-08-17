// Blob constants
const BLOB_STIFFNESS: f32 = 1.0;
const BLOB_RADIUS: f32 = 100.0;
const BLOB_GRID_SPACING: f32 = 5.0;
const BLOB_MAX_SPRING_REST_LENGTH: f32 =
    BLOB_GRID_SPACING * std::f32::consts::SQRT_2; // Connect to diagonal neighbors
const BLOB_BOUNCINESS: f32 = 0.2;

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
    particles: Vec<Particle>,
    springs: Vec<Spring>,
}

struct Particle {
    pos: Vec2,
    velocity: Vec2, // (speed, direction)
    mass: f32,
}

impl Particle {
    fn new(pos: Vec2) -> Self {
        Particle {
            pos: pos,
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
        // Create particles in a grid pattern within a circle
        let mut particles = Vec::new();

        /* Divide the area to fill into "steps" based on grid spacing and calculate
        how many "steps" needed to reach the bounds of the blob's bounding box */
        let half_grid_steps = (BLOB_RADIUS / BLOB_GRID_SPACING) as i32;

        for x in -half_grid_steps..half_grid_steps {
            for y in -half_grid_steps..half_grid_steps {
                // Calculate actual position based on which "step" we're on
                let grid_x = x as f32 * BLOB_GRID_SPACING;
                let grid_y = y as f32 * BLOB_GRID_SPACING;

                // Check if particle is within circular radius
                let distance_from_center =
                    (grid_x * grid_x + grid_y * grid_y).sqrt();
                if distance_from_center <= BLOB_RADIUS {
                    let particle_pos = origin + Vec2::new(grid_x, grid_y);
                    particles.push(Particle::new(particle_pos));
                }
            }
        }

        /* Connect each particle with all other possible particles, i.e. any
        particle which is <= BLOB_MAX_SPRING_REST_LENGTH distance away  */
        let mut springs = Vec::new();
        for i in 0..particles.len() {
            for j in (i + 1)..particles.len() {
                let distance = (particles[i].pos - particles[j].pos).length();
                if distance <= BLOB_MAX_SPRING_REST_LENGTH {
                    springs.push(Spring {
                        particle_a: i,
                        particle_b: j,
                        rest_length: distance as f32,
                    });
                }
            }
        }

        Blob {
            particles: particles,
            springs: springs,
        }
    }

    pub fn update(&mut self, dt: f32) {
        // Apply gravity to all particles' velocities
        for particle in &mut self.particles {
            particle.velocity.y += GRAVITY * dt;
        }

        // Apply all spring forces
        for spring in &self.springs {
            let pos_a = self.particles[spring.particle_a].pos;
            let pos_b = self.particles[spring.particle_b].pos;

            // Calculate the force vector
            let connection_vector = (pos_a - pos_b);
            let length = connection_vector.length();
            if length != 0.0 {
            let direction_vector = connection_vector / length; // vector of length 1
            // Hooke's law, F = k * (length - rest_length)
            let magnitude = BLOB_STIFFNESS * (length - spring.rest_length);
            let force_vector = direction_vector * magnitude;

            let mass_a = self.particles[spring.particle_a].mass;
            let mass_b = self.particles[spring.particle_b].mass;

            /* Apply force, once to the first particle, and apply an equal
            but opposite force to the second particle */
            self.particles[spring.particle_a].velocity -=
                force_vector * dt / mass_a;
            self.particles[spring.particle_b].velocity +=
                force_vector * dt / mass_b;
            }
        }

        let screen_width = screen_width();
        let screen_height = screen_height();

        // Update particle positions based on velocities
        for particle in &mut self.particles {
            particle.pos += particle.velocity * dt;
            if particle.pos.x < 0.0 {
                particle.pos.x = 0.0;
                particle.velocity.x *= -BLOB_BOUNCINESS;
            }
            if particle.pos.x > screen_width {
                particle.pos.x = screen_width;
                particle.velocity.x *= -BLOB_BOUNCINESS;
            }
            if particle.pos.y < 0.0 {
                particle.pos.y = 0.0;
                particle.velocity.y *= -BLOB_BOUNCINESS;
            }
            if particle.pos.y > screen_height {
                particle.pos.y = screen_height;
                particle.velocity.y *= -BLOB_BOUNCINESS;
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
