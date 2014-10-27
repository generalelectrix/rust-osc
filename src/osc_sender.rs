//! Module for sending OSC over a UDP socket.

extern crate std;

use std::io::net::udp::UdpSocket;
use std::io::net::ip::SocketAddr;

use std::io::{IoResult, MemWriter, BufWriter};

use std::str::*;

use osc_data::*;
use osc_util::*;

// we may want to generalize this beyond UDP later
/// Structure which contains the port used to send Osc packets, and handles
/// the task of converting Rust Osc objects into valic Osc messages
pub struct OscSender {

	socket: UdpSocket,
	dest: SocketAddr

}

impl OscSender {

	/// Constructs a new OscSender using a local socket address and a destination
	/// address.  Returns Err if an error occurred when trying to bind to the socket.
	pub fn new(local_addr: SocketAddr, dest_addr: SocketAddr) -> IoResult<OscSender> {
		match UdpSocket::bind(local_addr) {
		    Ok(s) => Ok(OscSender{socket: s, dest: dest_addr}),
		    Err(e) => return Err(e),
		}
	}


	/// Attempt to send a Rust OSC packet as an OSC UDP packet.
	pub fn send(&mut self, packet: OscPacket) -> IoResult<()> {
		// note that we trim off the first four bytes, as they are the packet length
		// and the socket automatically calcs and sends that
		self.socket.send_to(packet_to_buffer(packet).slice_from(4), self.dest)
	}


}

// convert an OscArg to its correpsonding type tag character
fn arg_to_type_tag(arg: &OscArg) -> char {
	match *arg {
		OscInt(_) => 'i',
		OscFloat(_) => 'f',
		OscStr(_) => 's',
		OscBlob(_) => 'b'
		/*
		OscInt64(_) => 'h',
		OscFloat64(_) => 'd',
		OscTime(_) => 't',
		OscSymbol(_) => 'S',
		OscChar(_) => 'c',
		OscColor(_) => 'r',
		OscMidi(_) => 'm',
		OscAssert(a) => {
			match a {
				True => 'T',
				False => 'F',
				Nil => 'N',
				Infinitum => 'I'
			}
		},
		// this was all nice and pretty until OscArray had to come fuck it all up
		// with OscArray I have to return a damn string instead of a char.  lame.
		// this right here is enough reason to just support OSC 1.0 for now
		OscArray(v) =>
		*/
	}
}

// format an Osc packet as a buffer of u8
// first four bytes are size of payload in bytes, remainder is payload
// this is public because it is VERY useful for receiver unit tests
// and is itself tested
pub fn packet_to_buffer(packet: OscPacket) -> Vec<u8> {

	let mut buf = MemWriter::new();

	// write a placeholder for the payload size
	buf.write_be_i32(0);


	match packet {
		OscMessage{ addr: addr, args: args} => {

			//--- write the address string

			buf.write_str(to_osc_string(addr).as_slice());

			//--- write the string of type tags

			// starts with a comma
			buf.write_char(',');

			// convert all the args to type tags and write them
			let tt_vec: Vec<u8> = args.iter().map(|a| arg_to_type_tag(a) as u8).collect();
			buf.write( tt_vec.as_slice() );

			// null-terminate type tag string
			buf.write_char('\0');

			// pad with nulls to obey osc string spec
			pad_with_null!(buf write_char args.len()+2)

			//--- write all the arguments

			for arg in args.into_iter() {
				write_arg(&mut buf, arg);
			}
		},
		OscBundle{time_tag: time_tag, conts: conts} => {

			//--- write the bundle identifier string
			buf.write_str("#bundle\0");

			//--- write the two parts of the time tag
			match time_tag {
				(sec, frac_sec) => {
					buf.write_be_u32(sec);
					buf.write_be_u32(frac_sec);
				}
			}

			//--- write each piece of the bundle payload, themselves Osc packets
			for packet in conts.into_iter() {
				buf.write(packet_to_buffer(packet).as_slice());
			}
		}
	}

	//--- write the length of the full message payload

	// get rid of the old writer
	let mut final_buf = buf.unwrap();
	let size = final_buf.len();

	// use a new writer to go back and write the size information
	BufWriter::new(final_buf.as_mut_slice()).write_be_i32( (size - 4) as i32);

	final_buf
}

// convert a string into a null-terminated, null-padded string
// length must be a multiple of 4 bytes!
fn to_osc_string(mut string: String) -> String {

	// add a null-terminator
	string.push('\0');

	// pad with nulls
	pad_with_null!(string push string.len());

	string
}

// write an OscArg using a given writer
// it may be helpful later to redefine this as generic on a type that impl Writer
fn write_arg(buf: &mut MemWriter, arg: OscArg) {
	match arg {
		OscInt(v) 	=> { buf.write_be_i32(v); },
		OscFloat(v) => { buf.write_be_f32(v); },
		OscStr(v) 	=> { buf.write_str(to_osc_string(v).as_slice()); },
		OscBlob(v) 	=> {
			buf.write_be_i32( v.len() as i32 );
			buf.write(v.as_slice());
			pad_with_null!(buf write_char v.len());
		}
	}
}

// many possibilities here, just check a couple by hand
#[test]
fn test_packet_to_buffer_message() {

	let a1 = OscInt(123);
	let a2 = OscFloat(0.0);
	let a3 = OscStr("abc".to_string());
	let a4 = OscBlob(vec!(1u8, 2u8, 3u8, 4u8, 5u8));

	let mess = OscMessage{
		addr: "/test/addr".to_string(),
		args: vec!(a1, a2, a3, a4)
	};

	let buf = packet_to_buffer(mess);

	let mut tbuf = MemWriter::new();
	tbuf.write_be_i32(44);

	tbuf.write_str("/test/addr\0\0");
	tbuf.write_str(",ifsb\0\0\0");
	tbuf.write_be_i32(123);
	tbuf.write_be_f32(0.0);
	tbuf.write_str("abc\0");
	tbuf.write_be_i32(5);
	tbuf.write(vec!(1u8, 2u8, 3u8, 4u8, 5u8, 0u8, 0u8, 0u8).as_slice());

	let tres = tbuf.unwrap();

	assert_eq!(buf, tres);
}

// many possibilities here, just check a couple by hand
#[test]
fn test_packet_to_buffer_bundle() {

	let a1 = OscInt(123);
	let a2 = OscFloat(0.0);
	let a3 = OscStr("abc".to_string());
	let a4 = OscBlob(vec!(1u8, 2u8, 3u8, 4u8, 5u8));

	let packet = OscBundle{
		time_tag: (0,1),
		conts: vec!(
			OscMessage{ addr:"/t".to_string(), args: vec!(a1)},
			OscBundle{
				time_tag: (123,456),
				conts: vec!(
					OscMessage{ addr:"/a".to_string(), args: vec!(a2, a3)},
					OscMessage{ addr:"/b".to_string(), args: vec!(a4)}
				)
			}
		)
	};

	let res = packet_to_buffer(packet);


	let mut tbuf = MemWriter::new();
	tbuf.write_be_i32(96); // size of total packet

	tbuf.write_str("#bundle\0");
	tbuf.write_be_u32(0);
	tbuf.write_be_u32(1);

	tbuf.write_be_i32(12); // size of first bundle element
	tbuf.write_str("/t\0\0");
	tbuf.write_str(",i\0\0");
	tbuf.write_be_i32(123);

	tbuf.write_be_i32(60); // size of second bundle element

	tbuf.write_str("#bundle\0");
	tbuf.write_be_u32(123);
	tbuf.write_be_u32(456);

	tbuf.write_be_i32(16); // size of first bundle message
	tbuf.write_str("/a\0\0");
	tbuf.write_str(",fs\0");
	tbuf.write_be_f32(0.0);
	tbuf.write_str("abc\0");

	tbuf.write_be_i32(20); // size of second bundle message
	tbuf.write_str("/b\0\0");
	tbuf.write_str(",b\0\0");
	tbuf.write_be_i32(5);
	tbuf.write(vec!(1u8, 2u8, 3u8, 4u8, 5u8, 0u8, 0u8, 0u8).as_slice());


	let tres = tbuf.unwrap();

	assert_eq!(tres, res);
}


#[test]
fn test_to_osc_string() {

	let t0 = "a".to_string();
	assert_eq!("a\0\0\0".to_string(),to_osc_string(t0));

	let t1 = "hello".to_string();
	assert_eq!("hello\0\0\0".to_string(),to_osc_string(t1));

	let t2 = "".to_string();
	assert_eq!("\0\0\0\0".to_string(),to_osc_string(t2));
}

// there may be some pathological corner cases I haven't considered here
#[test]
fn test_write_arg(){
	let mut buf = MemWriter::new();

	let a1 = OscInt(123);
	let a2 = OscFloat(0.0);
	let a3 = OscStr("abc".to_string());
	let a4 = OscBlob(vec!(1u8, 2u8, 3u8, 4u8, 5u8));

	write_arg(&mut buf, a1);
	write_arg(&mut buf, a2);
	write_arg(&mut buf, a3);
	write_arg(&mut buf, a4);

	let res = buf.unwrap();

	let mut tbuf = MemWriter::new();
	tbuf.write_be_i32(123);
	tbuf.write_be_f32(0.0);
	tbuf.write_str("abc\0");
	tbuf.write_be_i32(5);
	tbuf.write(vec!(1u8, 2u8, 3u8, 4u8, 5u8, 0u8, 0u8, 0u8).as_slice());

	let tres = tbuf.unwrap();

	assert_eq!(res, tres);

}

