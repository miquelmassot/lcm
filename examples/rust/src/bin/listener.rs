extern crate lcm;

// TODO: There must be a better way to include generated code.
mod exlcm {
    include!(concat!(env!("OUT_DIR"), "/exlcm/example_t.rs"));
}

fn main() {
    let mut lcm = lcm::Lcm::new().unwrap();

    lcm.subscribe("EXAMPLE", Box::new(|msg: exlcm::Example| {
        println!("Received message: {:?}", msg)
    }));

    loop {
        lcm.handle().unwrap_or_else(|e| {
            println!("Error handling message: {}", e);
        })
    }
}