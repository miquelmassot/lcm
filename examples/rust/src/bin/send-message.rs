extern crate lcm;

mod exlcm {
    include!(concat!(env!("OUT_DIR"), "/exlcm/example_t.rs"));
}

fn main() {
    let mut lcm = lcm::Lcm::new().unwrap();

    let mut my_data = exlcm::ExampleT::new();
    // let mut my_data = exlcm::Example::new();
    my_data.timestamp = 0;

    my_data.position[0] = 1.0;
    my_data.position[1] = 2.0;
    my_data.position[2] = 3.0;

    my_data.orientation[0] = 1.0;
    my_data.orientation[1] = 0.0;
    my_data.orientation[2] = 0.0;
    my_data.orientation[3] = 0.0;

    my_data.num_ranges = 15;
    my_data.ranges = (0..15).collect();

    my_data.name = "example string".to_string();
    my_data.enabled = true;

    match lcm.publish("EXAMPLE", &my_data) {
        Ok(()) => println!("Sent message."),
        Err(e) => println!("Failed to send message: {}", e)
    }
}
