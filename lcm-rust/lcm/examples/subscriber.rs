extern crate lcm;
use lcm::Lcm;

fn main()
{
	let mut lcm = Lcm::new().unwrap();
	lcm.subscribe("example", |msg: String| println!("Message: {}", msg) );

	loop { lcm.handle().unwrap(); }
}
