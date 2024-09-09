extern crate lcm;
use lcm::Lcm;

fn main()
{
	let mut lcm = Lcm::new().unwrap();
	lcm.publish("example", &"Hello, World!".to_string()).unwrap();
}
