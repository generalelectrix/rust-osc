//! Module for sending OSC over a UDP socket.

extern crate std;
extern crate byteorder;

use std::net::UdpSocket;
use std::net::SocketAddrV4;
use std::net::ToSocketAddrs;

use std::io::{Result, BufWriter};
use self::byteorder::{BigEndian, WriteBytesExt};

use std::io::prelude::*;

use data::*;
use data::OscPacket::*;
use data::OscArg::*;

use util::*;

// we may want to generalize this beyond UDP later
/// Structure which contains the port used to send Osc packets, and handles
/// the task of converting Rust Osc objects into valic Osc messages
pub struct OscSender<T: ToSocketAddrs> {

	socket: UdpSocket,
	dest: T

}

impl<T: ToSocketAddrs> OscSender<T> {

    /// Constructs a new OscSender using a local socket address and a destination
    /// address.  Returns Err if an error occurred when trying to bind to the socket.
    pub fn new(local_addr: T, dest_addr: T) -> Result<Self> {
        match UdpSocket::bind(local_addr) {
            Ok(s) => Ok(OscSender{socket: s, dest: dest_addr}),
            Err(e) => return Err(e),
        }
    }


	/// Attempt to send a Rust OSC packet as an OSC UDP packet.
	pub fn send(&self, packet: OscPacket) -> Result<usize> {
		// note that we trim off the first four bytes, as they are the packet length
		// and the socket automatically calcs and sends that
		self.socket.send_to(&packet_to_buffer(packet)[4..], &self.dest)
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

	let mut buf = Vec::new();

	// write a placeholder for the payload size
	buf.write_i32::<BigEndian>(0).unwrap();


	match packet {
		OscMessage{ addr, args} => {

			//--- write the address string

			buf.write(to_osc_string(addr).as_bytes()).unwrap();

			//--- write the string of type tags

			// starts with a comma
			buf.write(&[',' as u8]).unwrap();

			// convert all the args to type tags and write them
			let tt_vec: Vec<u8> = args.iter().map(|a| arg_to_type_tag(a) as u8).collect();
			buf.write( tt_vec.as_ref() ).unwrap();

			// null-terminate type tag string
			buf.write(&[0u8]).unwrap();

			// pad with nulls to obey osc string spec
			pad_with_null!(buf write args.len()+2);

			//--- write all the arguments

			for arg in args.into_iter() {
				write_arg(&mut buf, arg);
			}
		},
		OscBundle{time_tag, conts} => {

			//--- write the bundle identifier string
			buf.write("#bundle\0".as_bytes()).unwrap();

			//--- write the two parts of the time tag
			match time_tag {
				(sec, frac_sec) => {
					buf.write_u32::<BigEndian>(sec).unwrap();
					buf.write_u32::<BigEndian>(frac_sec).unwrap();
				}
			}

			//--- write each piece of the bundle payload, themselves Osc packets
			for packet in conts.into_iter() {
				buf.write(packet_to_buffer(packet).as_ref()).unwrap();
			}
		}
	}

	//--- write the length of the full message payload

	// get rid of the old writer
	let size = buf.len();

	// use a new writer to go back and write the size information
	BufWriter::new(&mut buf[..]).write_i32::<BigEndian>( (size - 4) as i32).unwrap();

    buf
}

// convert a string into a null-terminated, null-padded string
// length must be a multiple of 4 bytes!
fn to_osc_string(mut string: String) -> String {

	// add a null-terminator
	string.push('\0');

	// pad with nulls
    for _ in 0usize..four_byte_pad(string.len()) {
        string.extend(&['\0']);
    }

	string
}

// write an OscArg using a given writer
// it may be helpful later to redefine this as generic on a type that impl Writer
fn write_arg(buf: &mut Vec<u8>, arg: OscArg) {
	match arg {
		OscInt(v) 	=> { buf.write_i32::<BigEndian>(v).unwrap(); },
		OscFloat(v) => { buf.write_f32::<BigEndian>(v).unwrap(); },
		OscStr(v) 	=> { buf.write(to_osc_string(v).as_bytes()).unwrap(); },
		OscBlob(v) 	=> {
			buf.write_i32::<BigEndian>( v.len() as i32 ).unwrap();
			buf.write(v.as_ref()).unwrap();
			pad_with_null!(buf write v.len());
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

	let mut tbuf = Vec::new();
	tbuf.write_i32::<BigEndian>(44);

	tbuf.write("/test/addr\0\0".as_bytes());
	tbuf.write(",ifsb\0\0\0".as_bytes());
	tbuf.write_i32::<BigEndian>(123);
	tbuf.write_f32::<BigEndian>(0.0);
	tbuf.write("abc\0".as_bytes());
	tbuf.write_i32::<BigEndian>(5);
	tbuf.write(vec!(1u8, 2u8, 3u8, 4u8, 5u8, 0u8, 0u8, 0u8).as_ref());

	assert_eq!(buf, tbuf);
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


	let mut tbuf = Vec::new();
	tbuf.write_i32::<BigEndian>(96); // size of total packet

	tbuf.write("#bundle\0".as_bytes());
	tbuf.write_u32::<BigEndian>(0);
	tbuf.write_u32::<BigEndian>(1);

	tbuf.write_i32::<BigEndian>(12); // size of first bundle element
	tbuf.write("/t\0\0".as_bytes());
	tbuf.write(",i\0\0".as_bytes());
	tbuf.write_i32::<BigEndian>(123);

	tbuf.write_i32::<BigEndian>(60); // size of second bundle element

	tbuf.write("#bundle\0".as_bytes());
	tbuf.write_u32::<BigEndian>(123);
	tbuf.write_u32::<BigEndian>(456);

	tbuf.write_i32::<BigEndian>(16); // size of first bundle message
	tbuf.write("/a\0\0".as_bytes());
	tbuf.write(",fs\0".as_bytes());
	tbuf.write_f32::<BigEndian>(0.0);
	tbuf.write("abc\0".as_bytes());

	tbuf.write_i32::<BigEndian>(20); // size of second bundle message
	tbuf.write("/b\0\0".as_bytes());
	tbuf.write(",b\0\0".as_bytes());
	tbuf.write_i32::<BigEndian>(5);
	tbuf.write(vec!(1u8, 2u8, 3u8, 4u8, 5u8, 0u8, 0u8, 0u8).as_ref());

	assert_eq!(tbuf, res);
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
	let mut buf = Vec::new();

	let a1 = OscInt(123);
	let a2 = OscFloat(0.0);
	let a3 = OscStr("abc".to_string());
	let a4 = OscBlob(vec!(1u8, 2u8, 3u8, 4u8, 5u8));

	write_arg(&mut buf, a1);
	write_arg(&mut buf, a2);
	write_arg(&mut buf, a3);
	write_arg(&mut buf, a4);

	let mut tbuf = Vec::new();
	tbuf.write_i32::<BigEndian>(123);
	tbuf.write_f32::<BigEndian>(0.0);
	tbuf.write("abc\0".as_bytes());
	tbuf.write_i32::<BigEndian>(5);
	tbuf.write(vec!(1u8, 2u8, 3u8, 4u8, 5u8, 0u8, 0u8, 0u8).as_ref());

	assert_eq!(buf, tbuf);

}

