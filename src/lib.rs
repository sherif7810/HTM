use bit_vec::BitVec;
use rand::seq::SliceRandom;

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

    period: i32,
    min_overlap_duty_cycle: f32

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
               
               period: i32,
               min_overlap_duty_cycle: f32) -> Self {

        assert!(period >= 1);
        assert!(inhibition_radius > num_active_columns_per_inhibition_area);

        // Initialize columns with
        // `potential_radius` random connections.
        // 0.5 permanence and boost.
        let mut columns = Vec::new();

        for _ in 0..columns_length {
            let connected_synapses = (0..input_length).collect::<Vec<usize>>()
                .choose_multiple(&mut rand::thread_rng(), potential_radius)
                .zip(vec![0.5; potential_radius])
                .map(|(&synapse_i, p)| (synapse_i, p))
                .collect::<Vec<(usize, f32)>>();

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

            period,
            min_overlap_duty_cycle
        }
    }
    pub fn spatial_pooling_output(&mut self, input: &BitVec) -> BitVec {
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

        self.spatial_pooling_learning(input, overlap);

        active_columns
    }

    fn spatial_pooling_learning(&mut self, input: &BitVec, overlap: Vec<f32>) {
        let sp_output = self.spatial_pooling_output(input);
        let active_columns: Box<BitVec> = Box::new(sp_output.iter()
            .filter(|active| *active)
            .collect());
        let columns_indices: Box<Vec<usize>> = Box::new(sp_output.iter()
            .enumerate()
            .map(|(i, _)| i)
            .collect());
        let active_columns_indices: Box<Vec<usize>> = Box::new(sp_output.iter()
            .enumerate()
            .filter(|(_, active)| *active)
            .map(|(i, _)| i)
            .collect());

        // Learning
        for i in active_columns_indices.into_iter() {
            for (_, mut p) in &mut self.columns[i].connected_synapses {
                if p > self.permanence_threshold {
                    p += self.permanence_increment;
                    if p < 1. {
                        p = 1.0;
                    };
                } else {
                    p -= self.permanence_decrement;
                    if p > 1. {
                        p = 1.0;
                    }
                }
            }

        }

        self.update_active_duty_cycle(*active_columns);
        self.update_overlap_duty_cycle(overlap);
        
        for i in columns_indices.into_iter() {
            let neighbor_mean_active_duty_cycle = {
                let i_neighbors_duty_cycles = self.neighors(i).iter()
                    .map(|&i_neighbor_index| self.columns[i_neighbor_index].active_duty_cycle)
                    .collect::<Vec<f32>>();
                
                i_neighbors_duty_cycles.iter().sum::<f32>() / i_neighbors_duty_cycles.len() as f32
            };

            // BoostFunction
            self.columns[i].boost = if self.columns[i].active_duty_cycle >= neighbor_mean_active_duty_cycle {
                self.columns[i].boost + 1.0
            } else {
                self.columns[i].boost - 1.0
            };

            // Increase permanence for all connected synapses
            if self.columns[i].overlap_duty_cycle < self.min_overlap_duty_cycle {
                for (_, mut p) in &mut self.columns[i].connected_synapses {
                    p += self.permanence_increment;
                }
            }
        }
    }

    fn neighors(&self, i: usize) -> Vec<usize> {
        let mut neighbors_indices = Vec::new();
        let rng_min = {
            if (i as i32 - self.inhibition_radius as i32) < 0 {
               0
            } else { i - self.inhibition_radius }
        };
        let rng_max = {
            if i + self.inhibition_radius >= self.columns_length {
                self.columns_length - 1
            } else { i + self.inhibition_radius }
        };
        neighbors_indices.append(&mut (rng_min..i).collect::<Vec<usize>>());
        neighbors_indices.append(&mut (i + 1..rng_max).collect::<Vec<usize>>());
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
