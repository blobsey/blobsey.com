// Blob constants
const BLOB_STIFFNESS: f32 = 0.5;
const BLOB_BOUNCINESS: f32 = 0.2;

const BLOB_RADIUS: f32 = 300.0;
const BLOB_MAX_OUTER_CHORD_LENGTH: f32 = 15.0;

const BLOB_PARTICLE_MASS: f32 = 1.0;

use crate::constants::*;
use macroquad::{
    color::BLACK,
    math::Vec2,
    prelude::info,
    shapes::draw_line,
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
    velocity: Vec2, // (speed, direction)
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
                    velocity: Vec2 { x: 0.0, y: 0.0 },
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
            velocity: Vec2::ZERO,
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

        Blob {
            particles: particles,
            springs: springs,
        }
    }

    pub fn update(&mut self, dt: f32) {
        // // Apply gravity to all particles' velocities
        // for particle in &mut self.particles {
        //     particle.velocity.y += GRAVITY * dt;
        // }

        // // Apply all spring forces
        // for spring in &self.springs {
        //     let pos_a = self.particles[spring.particle_a].pos;
        //     let pos_b = self.particles[spring.particle_b].pos;

        //     // Calculate the force vector
        //     let connection_vector = pos_a - pos_b;
        //     let length = connection_vector.length();
        //     if length != 0.0 {
        //         let direction_vector = connection_vector / length; // vector of length 1
        //         // Hooke's law, F = k * (length - rest_length)
        //         let magnitude = BLOB_STIFFNESS * (length - spring.rest_length);
        //         let force_vector = direction_vector * magnitude;

        //         let mass_a = self.particles[spring.particle_a].mass;
        //         let mass_b = self.particles[spring.particle_b].mass;

        //         /* Apply force, once to the first particle, and apply an equal
        //         but opposite force to the second particle */
        //         self.particles[spring.particle_a].velocity -=
        //             force_vector * dt / mass_a;
        //         self.particles[spring.particle_b].velocity +=
        //             force_vector * dt / mass_b;
        //     }
        // }

        // let screen_width = screen_width();
        // let screen_height = screen_height();

        // // Update particle positions based on velocities
        // for particle in &mut self.particles {
        //     particle.pos += particle.velocity * dt;
        //     if particle.pos.x < 0.0 {
        //         particle.pos.x = 0.0;
        //         particle.velocity.x *= -BLOB_BOUNCINESS;
        //     }
        //     if particle.pos.x > screen_width {
        //         particle.pos.x = screen_width;
        //         particle.velocity.x *= -BLOB_BOUNCINESS;
        //     }
        //     if particle.pos.y < 0.0 {
        //         particle.pos.y = 0.0;
        //         particle.velocity.y *= -BLOB_BOUNCINESS;
        //     }
        //     if particle.pos.y > screen_height {
        //         particle.pos.y = screen_height;
        //         particle.velocity.y *= -BLOB_BOUNCINESS;
        //     }
        // }
    }

    pub fn draw(&self) {
        for spring in &self.springs {
            let pos_a = self.particles[spring.particle_a].pos;
            let pos_b = self.particles[spring.particle_b].pos;

            draw_line(
                pos_a.x, pos_a.y, // start point
                pos_b.x, pos_b.y, // end point
                1.0,    // thickness
                BLACK,   // color
            );
        }
    }
}
