extern crate lcm;

// TODO: Fix this
mod exlcm {
    include!(concat!(env!("OUT_DIR"), "/exlcm/example_t.rs"));
}

fn main() {
    let mut lcm = lcm::Lcm::new().unwrap();

    lcm.subscribe("EXAMPLE", Box::new(|msg: exlcm::ExampleT| {
        println!("Received message: {:?}", msg)
    }));

    lcm.handle().unwrap();
    // loop { lcm.handle(); }
}