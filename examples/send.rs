extern crate osc;

use osc::sender::*;
use osc::data::OscPacket::*;
use osc::data::OscArg::*;

fn main() {

	let local_addr = "localhost:7010";
	let dest_addr = "localhost:7009";

	let sender;

	match OscSender::new(local_addr, dest_addr) {
		Ok(s) => { sender = s; },
		Err(e) => { panic!(e); }
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

     for i in 0..5 {

		println!("trying to send");

		match sender.send(tests[i % tests.len()].clone()) {
		    Ok(_) => println!("send ok"),
		    Err(e) => println!("Error: {}", e)
		}

		//std::io::timer::sleep(1000);
	}

}
