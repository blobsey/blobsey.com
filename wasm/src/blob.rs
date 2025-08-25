// Blob constants
const BLOB_STIFFNESS: f32 = 100.0;
const BLOB_BOUNCINESS: f32 = 0.50; // Sane values are 0.0 - 1.0
const BLOB_RADIUS: f32 = 180.0;
const BLOB_PARTICLE_RADIUS: f32 = 10.0;
const BLOB_PARTICLE_MASS: f32 = 0.25;
const VELOCITY_DAMPING: f32 = 0.975;
const EPSILON: f32 = 0.00000001;
use crate::constants::*;
use macroquad::{
    color::{BLACK, RED},
    math::Vec2,
    prelude::info,
    rand,
    shapes::{draw_circle, draw_line},
    window::{screen_height, screen_width},
};
use std::f32::consts::{PI, SQRT_2};

/* The blob is made up of several particles, each connected to their neighbors
by springs */
pub struct Blob {
    particles: Vec<Particle>,
    springs: Vec<Spring>,
    outline_particles_indices: Vec<usize>,
}

struct Particle {
    pos: Vec2,
    prev_pos: Vec2, // For Verlet integration
}

struct Spring {
    particle_a: usize,
    particle_b: usize,
    rest_length: f32,
}

impl Blob {
    pub fn new(origin: Vec2) -> Blob {
        // Poisson disk sampling to pack particles
        const SAMPLES: usize = 30;
        let cell_size = BLOB_PARTICLE_RADIUS * SQRT_2.recip();
        let grid_width = (BLOB_RADIUS * 2.0 / cell_size).ceil() as usize;
        let grid_height = grid_width;

        // For checking if there are samples too close
        let mut grid: Vec<Vec<Option<Vec2>>> =
            vec![vec![None; grid_width]; grid_height];

        // Will hold "active points"
        let mut active_list: Vec<Vec2> = Vec::new();

        // Pick the first sample in the center and add to the queue
        let first_sample = Vec2::new(0.0, 0.0);
        let grid_x = ((first_sample.x + BLOB_RADIUS) / cell_size) as usize;
        let grid_y = ((first_sample.y + BLOB_RADIUS) / cell_size) as usize;
        grid[grid_x][grid_y] = Some(first_sample);
        active_list.push(first_sample);

        while !active_list.is_empty() {
            let i =
                (rand::gen_range(0.0, 1.0) * active_list.len() as f32) as usize;
            let parent = active_list[i];

            // Try to generate k candidates around this parent
            let mut found = false;
            for j in 0..SAMPLES {
                // Generate candidates at random angles, just far enough away
                let angle = 2.0
                    * PI
                    * (rand::gen_range(0.0, 1.0) + j as f32 / SAMPLES as f32);
                let radius = BLOB_PARTICLE_RADIUS * 2.0 + EPSILON;
                let x = parent.x + radius * angle.cos();
                let y = parent.y + radius * angle.sin();
                let candidate = Vec2 { x: x, y: y };
                let distance_from_center = candidate.length();
                if distance_from_center <= BLOB_RADIUS - BLOB_PARTICLE_RADIUS {
                    // Check if candidate is far enough from existing samples
                    let candidate_grid_x =
                        ((candidate.x + BLOB_RADIUS) / cell_size) as usize;
                    let candidate_grid_y =
                        ((candidate.y + BLOB_RADIUS) / cell_size) as usize;

                    let mut is_far_enough = true;

                    'outer: for step_x in -2..=2 {
                        let check_x = candidate_grid_x as i32 + step_x;
                        if check_x < 0 || check_x >= grid_width as i32 {
                            // Out of X bounds
                            continue;
                        }

                        for step_y in -2..=2 {
                            let check_y = candidate_grid_y as i32 + step_y;
                            if check_y < 0 || check_y >= grid_height as i32 {
                                // Out of Y bounds
                                continue;
                            }

                            if let Some(existing_sample) =
                                grid[check_x as usize][check_y as usize]
                            {
                                let distance =
                                    (candidate - existing_sample).length();
                                if distance < BLOB_PARTICLE_RADIUS * 2.0 {
                                    is_far_enough = false;
                                    break 'outer;
                                }
                            }
                        }
                    }

                    if is_far_enough {
                        // Found a valid candidate, add to grid and active_list
                        grid[candidate_grid_x][candidate_grid_y] =
                            Some(candidate);
                        active_list.push(candidate);
                        found = true;
                        break; // Break from SAMPLES loop
                    }
                }
            }

            if !found {
                active_list.swap_remove(i);
            }
        }

        // Create particles pased on the Poisson disc sampling
        let mut particles: Vec<Particle> = grid
            .iter()
            .flatten()
            .filter_map(|&sample| sample)
            .map(|sample| Particle {
                pos: sample + origin, // Translate to origin
                prev_pos: sample + origin,
            })
            .collect();

        // Add outline particles
        let circumference = 2.0 * PI * BLOB_RADIUS;
        let num_outline_particles = (circumference / (BLOB_PARTICLE_RADIUS * 2.0)).round() as
        usize;
        let mut outline_particle_indices = Vec::new();

        for i in 0..num_outline_particles {
            let angle = 2.0 * PI * i as f32 / num_outline_particles as f32;
            let outline_pos = Vec2::new(
                BLOB_RADIUS * angle.cos(),
                BLOB_RADIUS * angle.sin()
            );

            outline_particle_indices.push(particles.len());

            particles.push(Particle {
                pos: outline_pos + origin,
                prev_pos: outline_pos + origin,
            });
        }

        // Create springs between nearby particles
        let mut springs: Vec<Spring> = Vec::new();
        let spring_distance = BLOB_PARTICLE_RADIUS * 2.75; // Connect to close neighbors
        for i in 0..particles.len() {
            for j in (i + 1)..particles.len() {
                let distance = (particles[i].pos - particles[j].pos).length();
                if distance <= spring_distance {
                    springs.push(Spring {
                        particle_a: i,
                        particle_b: j,
                        rest_length: distance, // Use current distance as rest length
                    });
                }
            }
        }

        Blob {
            particles: particles,
            springs: springs,
            outline_particles_indices: outline_particle_indices,
        }
    }

    pub fn update(&mut self, dt: f32) {
        let mut forces = vec![Vec2::ZERO; self.particles.len()];

        // Spring forces
        for spring in &self.springs {
            let first_particle = &self.particles[spring.particle_a];
            let second_particle = &self.particles[spring.particle_b];

            let spring_vec = first_particle.pos - second_particle.pos;
            let spring_len = spring_vec.length();
            if spring_len > 0.0 {
                let unit_vec = spring_vec / spring_len;
                let displacement = spring_len - spring.rest_length;
                // Hooke's law, Force = stiffness * displacement
                let force = BLOB_STIFFNESS * displacement;
                let force_vec = unit_vec * force;

                forces[spring.particle_a] -= force_vec;
                forces[spring.particle_b] += force_vec;
            }
        }

        // Gravity
        for i in 0..forces.len() {
            forces[i].y += GRAVITY * BLOB_PARTICLE_MASS;
        }

        for (i, particle) in self.particles.iter_mut().enumerate() {
            // acceleration = F/m, needed for Verlet integration
            let acceleration = forces[i] / BLOB_PARTICLE_MASS;

            // Verlet integration: Pₙ₊₁ = 2Pₙ - Pₙ₋₁ + accel * dt²
            let next_pos =
                2.0 * particle.pos - particle.prev_pos + acceleration * dt * dt;

            // Apply velocity damping to reduce oscillations
            let velocity = next_pos - particle.pos;
            let damped_velocity = velocity * VELOCITY_DAMPING;
            let damped_next_pos = particle.pos + damped_velocity;

            particle.prev_pos = particle.pos;
            particle.pos = damped_next_pos;

            /* Boundaries checks. If we hit a wall, "fake" the prev_pos such that
            it is reflected beyond the boundary. This is done through some tricky
            math, i.e. starting with the base velocity formulas where x is the
            distance in one direction:
                velocity = (x - xₙ₋₁) / dt
            now we want to find the fake previous pos which would be from the
            fake "reflected" velocity, i.e. we wanna find x'ₙ₋₁
                reflected_velocity = (x' - x'ₙ₋₁) / dt
            the new position would be the boundary since this is a bounce:
                reflected_velocity = (boundary - x'ₙ₋₁) / dt
            so solving for prev_pos:
                x'ₙ₋₁ = boundary - reflected_velocity * dt
            since reflected_velocity is really just -velocity, rewrite as:
                x'ₙ₋₁ = boundary + velocity * dt
            plug the other side of the original velocity equation:
                x'ₙ₋₁ = boundary + ((x - xₙ₋₁) / dt) * dt
            simplify...
                x'ₙ₋₁ = boundary + (x - xₙ₋₁)
            finally apply some damping to the velocity:
                x'ₙ₋₁ = boundary + (x - xₙ₋₁) * bounciness
            */
            let screen_width = screen_width();
            let screen_height = screen_height();
            if particle.pos.x < 0.0 {
                particle.pos.x = 0.0;
                particle.prev_pos.x =
                    (particle.pos.x - particle.prev_pos.x) * BLOB_BOUNCINESS;
            } else if particle.pos.x > screen_width {
                particle.pos.x = screen_width;
                particle.prev_pos.x = screen_width
                    + (particle.pos.x - particle.prev_pos.x) * BLOB_BOUNCINESS;
            }

            if particle.pos.y < 0.0 {
                particle.pos.y = 0.0;
                particle.prev_pos.y =
                    (particle.pos.y - particle.prev_pos.y) * BLOB_BOUNCINESS;
            } else if particle.pos.y > screen_height {
                particle.pos.y = screen_height;
                particle.prev_pos.y = screen_height
                    + (particle.pos.y - particle.prev_pos.y) * BLOB_BOUNCINESS;
            }
        }

        // Check all particle pairs for collisions and "bump" them apart
        for i in 0..self.particles.len() {
            for j in (i + 1)..self.particles.len() {
                let distance =
                    (self.particles[i].pos - self.particles[j].pos).length();
                let min_distance = BLOB_PARTICLE_RADIUS * 2.0;

                if distance < min_distance && distance > 0.0 {
                    let overlap = min_distance - distance;
                    let direction = (self.particles[i].pos
                        - self.particles[j].pos)
                        / distance;

                    let separation =
                        direction * (overlap * 0.5);

                    self.particles[i].pos += separation;
                    self.particles[j].pos -= separation;
                }
            }
        }
    }

    pub fn draw(&self) {
        for i in 0..self.outline_particles_indices.len() {
            let j = (i + 1) % self.outline_particles_indices.len();

            let index_i = self.outline_particles_indices[i];
            let index_j = self.outline_particles_indices[j];

            let pos_a = self.particles[index_i].pos;
            let pos_b = self.particles[index_j].pos;

            draw_line(
                pos_a.x, pos_a.y, // start point
                pos_b.x, pos_b.y, // end point
                10.0,     // thickness
                BLACK,   // color
            );
        }
    }
}
