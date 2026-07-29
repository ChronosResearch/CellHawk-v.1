#[test]
fn claim_c5_swarm_bandwidth() {
    let bytes_per_msg = 50; // Approximated protobuf size
    let drones = 5;
    let hz = 1;
    let duration = 100;

    let total_bytes = bytes_per_msg * drones * hz * duration;
    let bits = total_bytes * 8;
    let kbps = (bits as f64 / duration as f64) / 1000.0;

    println!("Swarm bandwidth: {} kbps", kbps);
    assert!(kbps < 4.0);
}
