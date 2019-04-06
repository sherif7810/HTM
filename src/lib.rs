use bit_vec::BitVec;
use rand::seq::SliceRandom;
use std::rc::Rc;

/// Hierarchical temporal memory (HTM) layer.
pub struct HTMLayer {
    input_length: usize,
    columns_length: usize,
    /// Global inhibition.
    num_active_columns_per_inhibition_area: usize,
    /// Local inhibition.
    inhibition_radius: usize,

    columns: Vec<Column>,
    potential_radius: usize,

    permanence_threshold: f32,
    permanence_increment: f32,
    permanence_decrement: f32,

    stimulus_threshold: f32,

    period: i32

}

/// A cortical column.
/// It connects to `HTMLayer`'s input with `potential_radius` synapses.
struct Column {
    /// Each synapse has a permanence value.
    connected_synapses: Vec<(usize, f32)>,
    /// It's used for learning.
    boost: f32,

    active_duty_cycle: f32,
    overlap_duty_cycle: f32
}

impl HTMLayer {
    pub fn new(input_length: usize, columns_length: usize,
               num_active_columns_per_inhibition_area: usize,
               inhibition_radius: usize,

               potential_radius: usize,

               permanence_threshold: f32,
               permanence_increment: f32, permanence_decrement: f32,

               stimulus_threshold: f32,
               
               period: i32) -> Self {

        assert!(period >= 1);

        // Initialize columns with
        // `potential_radius` random connections.
        // 0.5 permanence and boost.
        let mut columns = Vec::new();

        for _ in 0..columns_length {
            let connected_synapses: Vec<usize> = (0..input_length).collect::<Vec<usize>>()
                .choose_multiple(&mut rand::thread_rng(), potential_radius)
                .map(|&synapse_i| synapse_i)
                .collect();
            let half_vec = vec![0.5; connected_synapses.len()];
            let connected_synapses = connected_synapses.iter()
                .zip(half_vec)
                .map(|(i, p)| (*i, p)).collect();

            columns.push(Column {
                connected_synapses,
                boost: 0.5,
                active_duty_cycle: 0.0,
                overlap_duty_cycle: 0.0
            });
        }


        Self {
            input_length,
            columns_length,
            num_active_columns_per_inhibition_area,
            inhibition_radius,

            columns,
            potential_radius,

            permanence_threshold,
            permanence_increment,
            permanence_decrement,

            stimulus_threshold,

            period
        }
    }
    pub fn spatial_pooling_output(&self, input: BitVec) -> BitVec {
        // Overlap
        let mut overlap = Vec::new();
        for i in 0..self.columns_length {
            overlap.push(0.);
            for (input_bit_index, _) in &self.columns[i].connected_synapses {
                if input[*input_bit_index] == true { overlap[i] += 1.; }
            }
            overlap[i] *= self.columns[i].boost;
        }

        // Winning columns after inhibition
        let mut active_columns = BitVec::new();
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
                active_columns.push(true);
            } else {
                active_columns.push(false);
            }
        }

        active_columns
    }

    pub fn spatial_pooling_learning(&mut self, input: BitVec, overlap: Vec<f32>) {
        let active_columns = self.spatial_pooling_output(input);
        let active_columns_indices: Vec<usize> = active_columns.iter()
            .enumerate()
            .filter(|(i, active)| *active)
            .map(|(i, _)| i)
            .collect();

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

        self.update_active_duty_cycle(active_columns);
        self.update_overlap_duty_cycle(overlap);
        
    }

    fn neighors(&self, i: usize) -> Vec<usize> {
        let mut neighbors_indices = Vec::new();
        neighbors_indices.append(&mut (i - self.inhibition_radius..i - 1).collect::<Vec<usize>>());
        neighbors_indices.append(&mut (i + 1..i + self.inhibition_radius).collect::<Vec<usize>>());
        neighbors_indices
    }

    fn update_active_duty_cycle(&mut self, active_columns: BitVec) {
        for i in 0..self.columns.len() {
            self.columns[i].active_duty_cycle = (self.columns[i].active_duty_cycle * (self.period - 1) as f32 + active_columns[i] as u8 as f32) / self.period as f32;
        }
    }

    fn update_overlap_duty_cycle(&mut self, overlap: Vec<f32>) {
        for i in 0..self.columns.len() {
            self.columns[i].overlap_duty_cycle = (self.columns[i].overlap_duty_cycle * (self.period - 1) as f32 + overlap[i]) / self.period as f32;
        }
    }
}
