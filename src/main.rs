use bit_vec::BitVec;
use my_htm::HTMLayer;
use std::num::NonZeroU32;

fn main() {
    let mut ip = BitVec::new();
    for _ in 0..500 {
        ip.push(true);
        ip.push(true);
    }
    for _ in 500..1000 {
        ip.push(false);
        ip.push(true);
    }

    let mut my_layer = HTMLayer::<2048>::new(ip.len(),
                                 8, 10, 10,
                                 2.0, 8.0, 2.0,
                                 1.0,
                                 NonZeroU32::new(4).unwrap(),2.0);

    let active_columns = my_layer.spatial_pooling_output(&ip);
    println!("Active columns = {:?}.", active_columns);
    let active_columns = my_layer.spatial_pooling_output(&ip);
    println!("Active columns = {:?}.", active_columns);
}
