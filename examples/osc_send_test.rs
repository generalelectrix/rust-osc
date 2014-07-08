#![feature(globs)]

extern crate osc;

use osc::osc_sender::*;
use osc::osc_data::*;

use std::io::net::ip::{Ipv4Addr,SocketAddr};

fn main() {

	let local_addr = SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: 7010 };
	let dest_addr = SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: 7009 };

	let mut sender;

	match OscSender::new(local_addr, dest_addr) {
		Ok(s) => { sender = s; },
		Err(e) => { fail!(e); }
	}

	let tests = vec!(
		OscMessage{
			addr: "/test/addr/1".to_string(),
			args: vec!( OscInt(123), OscFloat(2.0), OscStr("I'm a string".to_string()), OscBlob(vec!(5u8, 10u8, 15u8)) )
		},
		OscMessage{
			addr: "/test/addr/2".to_string(),
			args: vec!( OscInt(123456) )
		},
		OscBundle{
			time_tag: (123,456),
			conts: vec!( OscMessage{addr: "/subaddr".to_string(), args: vec!(OscInt(789)) } )
		},
		OscBundle{
			time_tag: (789,1001),
			conts: vec!(
				OscMessage{addr: "/subaddr".to_string(), args: vec!(OscInt(789)) },
				OscBundle{
					time_tag: (1,0),
					conts: vec!( OscMessage{addr: "/subsubaddr".to_string(), args: vec!(OscBlob(vec!(5u8, 10u8, 15u8, 20u8, 25u8)))})
				}
			)
		}
	);

	let mut i = 0;

	loop {

		println!("trying to send");

		match sender.send(tests.get(i % tests.len()).clone()) {
		    Ok(_) => println!("send ok"),
		    Err(e) => println!("Error: {}", e)
		}

		i += 1;

		if i == 4 {break;}

		//std::io::timer::sleep(1000);
	}

}