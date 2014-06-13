#![feature(globs)]

extern crate osc;

use osc::osc_receiver::*;

use std::io::net::ip::{Ipv4Addr,SocketAddr};

fn main() {

	let addr = SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: 7009 };

	let mut receiver;

	match OscReceiver::new(addr) {
		Ok(r) => {receiver = r;},
		Err(e) => { fail!(e); }
	}

	loop {

		println!("trying to receive");

		match receiver.recv() {
		    Ok(o) => println!("contents: {}",o),
		    Err(e) => println!("Error: {}", e)
		}

	}

}