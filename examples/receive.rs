extern crate osc;

use osc::receiver::*;

fn main() {

	let addr = "localhost:7009";

	let mut receiver;

	match OscReceiver::new(addr) {
		Ok(r) => {receiver = r;},
		Err(e) => { panic!(e); }
	}

	loop {

		println!("trying to receive");

		match receiver.recv(None) {
		    Ok(o) => println!("contents: {:?}",o),
		    Err(e) => println!("Error: {:?}", e)
		}

	}

}
