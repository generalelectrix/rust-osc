extern crate osc;

use osc::osc_receiver::*;

use std::net::{Ipv4Addr,SocketAddrV4};

fn main() {

	let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 7009);

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