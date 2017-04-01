extern crate lcm;
extern crate example;
use example::exlcm;

fn main() {
    let mut lcm = lcm::Lcm::new().unwrap();

    lcm.subscribe("EXAMPLE",
                  |msg: exlcm::Example| println!("Received message: {:?}", msg));

    loop {
        lcm.handle().unwrap_or_else(|e| {
            println!("Error handling message: {}", e);
        })
    }
}
