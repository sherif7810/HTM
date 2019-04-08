use bit_vec::BitVec;
use my_htm::HTMLayer;

fn main() {
    let mut ip = BitVec::new();
    for _ in 0..1000 {
        ip.push(false);
        ip.push(true);
    }

    let my_layer = HTMLayer::new(ip.len(), 512, 2, 2, 8, 2.0, 2.0, 2.0, 2.0, 4, 2.0);

    my_layer.spatial_pooling_output(ip);
}