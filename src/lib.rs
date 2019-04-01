use std::mem;

use bit_vec::BitVec;
use rand::seq::SliceRandom;

pub struct HTMLayer {
    input_length: usize,
    columns_length: usize,
    num_active_columns_per_inhibition_area: usize,
    inhibition_radius: usize, // Local inhibition

    columns: Vec<Column>,
    potential_radius: usize,

    permanence_threshold: f32,
    permanence_increment: f32,
    permanence_decrement: f32,

    stimulus_threshold: f32,

}

#[derive(Clone)]
pub struct Column {
    connected_synapses: Vec<(usize, f32)>,
    boost: f32
}

impl HTMLayer {
    pub fn new(input_length: usize, columns_length: usize,
               num_active_columns_per_inhibition_area: usize,
               inhibition_radius: usize,

               potential_radius: usize,

               permanence_threshold: f32,
               permanence_increment: f32, permanence_decrement: f32,

               stimulus_threshold: f32,) -> Self {

        // Initialize columns with
        // `potential_radius` random connections.
        // 0.5 permanence and boost.
        let mut columns = Vec::new();

        for i in 0..columns_length {
            let connected_synapses: Vec<usize> = (0..input_length).collect::<Vec<usize>>()
                .choose_multiple(&mut rand::thread_rng(), potential_radius)
                .map(|&synapse_i| synapse_i)
                .collect();
            let mut half_vec = vec![0.5; connected_synapses.len()];
            let connected_synapses = connected_synapses.iter()
                .zip(half_vec)
                .map(|(i, p)| (*i, p)).collect();

            columns.push(Column {
                connected_synapses: connected_synapses,
                boost: 0.5
            });
        }


        HTMLayer {
            input_length: input_length,
            columns_length: columns_length,
            num_active_columns_per_inhibition_area: num_active_columns_per_inhibition_area,
            inhibition_radius: inhibition_radius,

            columns: columns,
            potential_radius: potential_radius,

            permanence_threshold: permanence_threshold,
            permanence_increment: permanence_increment,
            permanence_decrement: permanence_decrement,

            stimulus_threshold: stimulus_threshold,
        }
    }
    pub fn spatial_pooling(&mut self, input: BitVec) {
        // Overlap
        let mut overlap = Vec::new();
        for i in 0..self.columns_length {
            overlap.push(0.);
            for (_, permanence) in &self.columns[i].connected_synapses {
                if *permanence > self.permanence_threshold { overlap[i] += 1.; }
            }
            overlap[i] *= self.columns[i].boost;
        }

        // Winning columns after inhibition
        let mut active_columns_indices = Vec::new();
        for i in 0..self.columns_length {
            let min_local_activity = {
                let neighbors = self.neighors(i);
                
                // kthScore
                let mut local_overlap = Vec::new();
                neighbors.iter().for_each(|&i| if overlap[i] > 0. { local_overlap.push(overlap[i]); });
                local_overlap.sort_by(|a, b| a.partial_cmp(b).unwrap()); // Can't sort floats.

                local_overlap[local_overlap.len() - self.num_active_columns_per_inhibition_area]
            };

            if overlap[i] > self.stimulus_threshold  && overlap[i] > min_local_activity {
                active_columns_indices.push(i);
            }
        }

        // Learning
        let permanence_threshold = self.permanence_threshold;
        let permanence_increment = self.permanence_increment;
        let permanence_decrement = self.permanence_decrement;
        for i in active_columns_indices {
            for (_, mut p) in &mut self.columns[i].connected_synapses {
                if p > permanence_threshold {
                    p += permanence_increment;
                    if p < 1. {
                        p = 1.0;
                    };
                } else {
                    p -= permanence_decrement;
                    if p > 1. {
                        p = 1.0;
                    }
                }
            }

        }
        unimplemented!();
    }

    fn neighors(&self, i: usize) -> Vec<usize> {
        let mut neighbors_indices = Vec::new();
        neighbors_indices.append(&mut (i - self.inhibition_radius..i - 1).collect::<Vec<usize>>());
        neighbors_indices.append(&mut (i + 1..i + self.inhibition_radius).collect::<Vec<usize>>());
        neighbors_indices
    }
}
