extern crate osc;

use osc::receiver::*;

fn main() {

	let addr = "localhost:7009";

	let receiver = OscReceiver::new(addr).unwrap();

	loop {

		println!("trying to receive");

		match receiver.recv(None) {
		    Ok(o) => println!("contents: {:?}",o),
		    Err(e) => println!("Error: {:?}", e)
		}

	}

}
