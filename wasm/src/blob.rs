// Blob constants
const BLOB_STIFFNESS: f32 = 25.0;
const BLOB_BOUNCINESS: f32 = 0.50; // Sane values are 0.0 - 1.0
const BLOB_RADIUS: f32 = 100.0;
const BLOB_MAX_OUTER_CHORD_LENGTH: f32 = 50.0;
const BLOB_PARTICLE_RADIUS: f32 = 10.0;
const BLOB_PARTICLE_MASS: f32 = 0.1;

use crate::constants::*;
use macroquad::{
    color::{BLACK, RED},
    math::Vec2,
    prelude::info,
    shapes::{draw_circle, draw_line},
    window::{screen_height, screen_width},
};
use std::f32::consts::{PI, SQRT_2};

/* The blob is made up of several particles, each connected to their neighbors
by springs */
pub struct Blob {
    particles: Vec<Particle>,
    springs: Vec<Spring>,
}

struct Particle {
    pos: Vec2,
    prev_pos: Vec2, // For Verlet integration
    mass: f32,
}

struct Spring {
    particle_a: usize,
    particle_b: usize,
    rest_length: f32,
}

impl Blob {
    pub fn new(origin: Vec2) -> Blob {
        // Create concentric rings of particles
        let mut particles_per_layer: Vec<Vec<Particle>> = Vec::new();

        /* Angle for the triangle made by min chord_length and radius. This will
        be usually smaller than the actual angle, which must be strictly one of
        2pi/3, 2pi/4, 2pi/5, etc. because those angles make n-gons */
        let raw_angle =
            2.0 * (BLOB_MAX_OUTER_CHORD_LENGTH / (2.0 * BLOB_RADIUS)).asin();
        let num_outer_particles = {
            let n = (2.0 * PI / raw_angle).ceil() as usize;
            // Find the smallest power of 2 such that 3 * 2^k >= n
            let target = (n + 2) / 3; // Round up n/3
            let power_of_2 = target.next_power_of_two().max(1);
            3 * power_of_2
        }
        .max(6);

        let mut particles_this_layer = num_outer_particles;
        let mut angle = (2.0 * PI) / particles_this_layer as f32;
        let mut radius_this_layer = BLOB_RADIUS;
        let mut chord_length = 2.0 * radius_this_layer * (angle / 2.0).sin();
        while radius_this_layer > 0.0 {
            let mut current_layer = Vec::new();
            for particle_index in 0..particles_this_layer {
                let layer_angle = 2.0
                    * PI
                    * (particle_index as f32 / particles_this_layer as f32);
                let x = origin.x + radius_this_layer * layer_angle.cos();
                let y = origin.y + radius_this_layer * layer_angle.sin();

                // Add to current layer
                current_layer.push(Particle {
                    pos: Vec2 { x: x, y: y },
                    prev_pos: Vec2 { x: x, y: y },
                    mass: BLOB_PARTICLE_MASS,
                });
            }
            particles_per_layer.push(current_layer);
            /* Super ugly, but this will redo the chord_length calculation for
            each layer based on the new particle count */
            particles_this_layer = particles_this_layer / 2;
            angle = (2.0 * PI) / particles_this_layer as f32;
            radius_this_layer -= chord_length;
            chord_length = 2.0 * radius_this_layer * (angle / 2.0).sin();
        }

        let mut springs = Vec::new();
        let mut layer_start = 0;
        for (layer_index, layer) in particles_per_layer.iter().enumerate() {
            // Connect neighbors within the same layer
            for i in 0..layer.len() {
                let next_i = (i + 1) % layer.len(); // Wrap around to form ring
                let rest_length =
                    (layer[i].pos - layer[(i + 1) % layer.len()].pos).length();

                // Calculate the index it will have after flattening
                let particle_a = layer_start + i;
                let particle_b = layer_start + next_i;

                springs.push(Spring {
                    particle_a,
                    particle_b,
                    rest_length: rest_length,
                });
            }

            // Cross-layer connections (isosceles triangles)
            if layer_index > 0 {
                let outer_layer = &particles_per_layer[layer_index - 1];
                let outer_layer_start = layer_start - outer_layer.len(); // Previous layer start

                for i in 0..layer.len() {
                    let inner_particle = layer_start + i;

                    // Calculate relative indices in the outer layer
                    let outer_same_idx = (i * 2) % outer_layer.len();
                    let outer_left_idx = (outer_same_idx + outer_layer.len()
                        - 1)
                        % outer_layer.len();
                    let outer_right_idx =
                        (outer_same_idx + 1) % outer_layer.len();

                    // Create springs to each of the 3 outer particles
                    for &outer_idx in
                        &[outer_same_idx, outer_left_idx, outer_right_idx]
                    {
                        let outer_particle = outer_layer_start + outer_idx;
                        let rest_length = (layer[i].pos
                            - outer_layer[outer_idx].pos)
                            .length();

                        springs.push(Spring {
                            particle_a: inner_particle,
                            particle_b: outer_particle,
                            rest_length,
                        });
                    }
                }
            }
            layer_start += layer.len();
        }

        // Store innermost layer info before adding center
        let innermost_size = particles_per_layer.last().unwrap().len();

        // Add center particle as final layer
        particles_per_layer.push(vec![Particle {
            pos: origin,
            prev_pos: origin,
            mass: BLOB_PARTICLE_MASS,
        }]);

        // Flatten all particles
        let particles: Vec<Particle> = particles_per_layer
            .into_iter()
            .flatten()
            .collect();

        // Connect center (last particle) to innermost ring
        let center_idx = particles.len() - 1;
        let innermost_start = center_idx - innermost_size;
        for i in 0..innermost_size {
            let inner_idx = innermost_start + i;
            springs.push(Spring {
                particle_a: center_idx,
                particle_b: inner_idx,
                rest_length: (particles[center_idx].pos - particles[inner_idx].pos).length(),
            });
        }
        // let mut particles: Vec<Particle> = Vec::new();
        // let offset = BLOB_RADIUS as i32 / 2;
        // let step = 50;
        // for i in (-offset..offset).step_by(step) {
        //     for j in (-offset..offset).step_by(step) {
        //         particles.push(Particle {
        //             pos: Vec2 {
        //                 x: origin.x + i as f32,
        //                 y: origin.y + j as f32,
        //             },
        //             prev_pos: Vec2 {
        //                 x: origin.x + i as f32,
        //                 y: origin.y + j as f32,
        //             },
        //             mass: BLOB_PARTICLE_MASS,
        //         });
        //     }
        // }

        // let mut springs: Vec<Spring> = Vec::new();
        // for i in 0..particles.len() {
        //     let max_rest_len = SQRT_2 * step as f32;
        //     for j in (i + 1)..particles.len() {
        //         let distance = (particles[i].pos - particles[j].pos).length();
        //         if distance <= max_rest_len {
        //             springs.push(Spring {
        //                 particle_a: i,
        //                 particle_b: j,
        //                 rest_length: distance,
        //             });
        //         }
        //     }
        // }

        Blob {
            particles: particles,
            springs: springs,
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
            forces[i].y += GRAVITY * &self.particles[i].mass;
        }

        // Particle collision forces
        for i in 0..self.particles.len() {
            for j in (i + 1)..self.particles.len() {
                let particle_a = &self.particles[i];
                let particle_b = &self.particles[j];

                let collision_vec = particle_a.pos - particle_b.pos;
                let distance = collision_vec.length();
                let min_distance = 2.0 * BLOB_PARTICLE_RADIUS;

                if distance < min_distance && distance > 0.0 {
                    let unit_vec = collision_vec / distance;
                    let overlap = min_distance - distance;
                    let force_magnitude = overlap * BLOB_STIFFNESS * 2.0; // Stronger repulsion
                    let force_vec = unit_vec * force_magnitude;

                    forces[i] += force_vec;
                    forces[j] -= force_vec;
                }
            }
        }

        for (i, particle) in self.particles.iter_mut().enumerate() {
            // acceleration = F/m, needed for Verlet integration
            let acceleration = forces[i] / particle.mass;

            // Verlet integration: Pₙ₊₁ = 2Pₙ - Pₙ₋₁ + accel * dt²
            let next_pos =
                2.0 * particle.pos - particle.prev_pos + acceleration * dt * dt;

            particle.prev_pos = particle.pos;
            particle.pos = next_pos;

            let screen_width = screen_width();
            let screen_height = screen_height();

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

        info!("{:?}", self.particles[0].pos);
    }

    pub fn draw(&self) {
        for spring in &self.springs {
            let pos_a = self.particles[spring.particle_a].pos;
            let pos_b = self.particles[spring.particle_b].pos;

            draw_line(
                pos_a.x, pos_a.y, // start point
                pos_b.x, pos_b.y, // end point
                1.0,     // thickness
                BLACK,   // color
            );
        }

        for particle in &self.particles {
            draw_circle(particle.pos.x, particle.pos.y, 5.0, RED);
        }
    }
}
