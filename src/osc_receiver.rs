extern crate std;

use std::io::net::udp::UdpSocket;
use std::io::net::ip::SocketAddr;

use std::io::{IoResult, IoError, InvalidInput, OtherIoError, BufReader, MemWriter};

use std::str::*;

use osc_data::*;

use osc_util::*;

use osc_sender::*;

// apparently 1536 is a common UDP MTU.
static UDP_BUFFER_SIZE: uint = 1536;

// static BUNDLE_ID: &'static str = "#bundle"; // perhaps we may want to use this some other day
static BUNDLE_FIRST_CHAR: char = '#';

// smallest packet is a 0 character address (4 bytes) and a comma for type tag (4 bytes)
static MIN_OSC_PACKET_SIZE: uint = 8;
static PACKET_SIZE_ERR: &'static str = "Packet with less than 8 bytes.";

// we may want to generalize this beyond UDP later
/// Structure which contains the port used to receive Osc packets, and handles
/// the task of interpreting those packets as valid Osc.
pub struct OscReceiver {

	socket: UdpSocket

}

impl OscReceiver {

	/// Constructs a new OscReceiver using a socket address.  Returns Err if an
	/// error occurred when trying to bind to the socket.
	pub fn new(addr: SocketAddr) -> IoResult<OscReceiver> {
		match UdpSocket::bind(addr) {
		    Ok(s) => Ok(OscReceiver{socket: s}),
		    Err(e) => return Err(e),
		}
	}

	/// Receive a Osc packet.  Blocks until a packet is available at the port.
	pub fn recv(&mut self) -> IoResult<OscPacket> {

		// initialize a receive buffer
		let mut buf = [0u8, ..UDP_BUFFER_SIZE];

		let mut packet_len;

		match self.socket.recvfrom(buf) {
			// ignoring source address from now, can bind it here if desired
			// if we didn't received enough data, throw an error
			Ok((num, _)) if num < MIN_OSC_PACKET_SIZE => {
				return Err(IoError{kind: InvalidInput, desc: PACKET_SIZE_ERR, detail: None});
			}
			// if we received at least 8 bytes, continue
		    Ok((num, _)) => {
		    	packet_len = num;
		    }
		    // return an error if we encountered one
		    Err(e) => {
		    	return Err(e);
		    }
		}

		read_packet(buf.slice_to(packet_len))
	}
}

// interpret a buffer as an Osc packet; useful as bundles are recursive
fn read_packet(buf: &[u8]) -> IoResult<OscPacket> {
	if is_bundle(buf) {
		read_bundle(buf)
	}
	else {
		read_message(buf)
	}
}

fn is_bundle(buf: &[u8]) -> bool {
	buf[0] as char == BUNDLE_FIRST_CHAR
}

// read the buffer as a bundle, assuming proper OSC formatting
fn read_bundle(buf: &[u8]) -> IoResult<OscPacket> {
	let mut reader = BufReader::new(buf);

	// ignore 8 byte bundle ID string
	reader.consume(8);

	// read the 64 bit time tag
	let (sec, frac_sec): OscTimeTag;

	match reader.read_be_u32() {
		Ok(v) => sec = v,
		Err(e) => return Err(e)
	}

	match reader.read_be_u32() {
		Ok(v) => frac_sec = v,
		Err(e) => return Err(e)
	}

	// now interpret the bundle contents
	let mut bundle_conts = Vec::new();

	// until we're out of buffer, read elements
	let mut element_size: uint;
	while !reader.eof() {
		// get the length of the bundle element, should be a mult of 4
		match reader.read_be_i32() {
			Ok(n) => element_size = n as uint,
			Err(e) => return Err(e)
		}

		// try to read the specified length
		match reader.read_exact(element_size) {
			Ok(b) => {
				// if we got a valid vector, interpret it as a Osc packet
				match read_packet(b.as_slice()) {
					Ok(pack) => bundle_conts.push(pack),
					Err(e) => return Err(e)
				}
			},
			Err(e) => { return Err(e); }
		}


	}

	Ok(OscBundle{time_tag: (sec, frac_sec), conts: bundle_conts})
}

// interpret a byte array as an Osc message
fn read_message(buf: &[u8]) -> IoResult<OscPacket> {

	let mut reader = BufReader::new(buf);

	let addr: String;

	// get the address
	match read_null_term_string(&mut reader) {
		Ok(a) => { addr = a; },
		Err(e) => { return Err(e); }
	}

	let tt_str: String;

	// now read the type tags
	match read_null_term_string(&mut reader) {
		Ok(tt) => { tt_str = tt; },
		Err(e) => { return Err(e); }
	}

	// check to make sure the first char is a comma
	if tt_str.as_slice().char_at(0) != ',' {
		return Err(IoError{kind: InvalidInput, desc: "Missing type tag comma.", detail: None});
	}

	// now read the arguments
	let mut args = Vec::with_capacity(tt_str.len() - 1);

	// iterate over the args, skipping the comma ID
	for tt in tt_str.as_slice().chars().skip(1) {
		match read_osc_arg(&mut reader, tt) {
			Ok(arg) => { args.push(arg); },
			Err(e) => { return Err(e); }
		}
	}

	// check if we've read all the data; this may be unnecessarily cautious
	if reader.eof() {
		Ok(OscMessage{addr: addr, args: args})
	}
	else {
		Err(IoError{kind: OtherIoError, desc: "Failed to read all data in buffer!", detail: None})
	}

}

// Osc strings are null-terminated, we'll do this a lot
fn read_null_term_string(reader: &mut BufReader) -> IoResult<String> {
	// read until null
	match reader.read_until(0u8) {
		Ok(mut m) => {

			// Osc strings are always multiples of 4 bytes; read a few more if we didn't get a mult of 4
			reader.consume(four_byte_pad(m.len()));

			m.pop(); // remove the trailing null
			// try to convert to a string
			match std::str::from_utf8(m.as_slice()) {
				Some(a) => {
					Ok(a.into_string())
				},
				// return an error if we can't parse this as a string
				None => {
					Err(IoError{kind: InvalidInput, desc: "Could not parse input as string.", detail: None})
				}
			}
		},
		Err(e) => {
			Err(e)
		}
	}
}

// read an osc argument based on a type tag
fn read_osc_arg(reader: &mut BufReader, type_tag: char) -> IoResult<OscArg> {

	match type_tag {
		'i' => match reader.read_be_i32() {
			Ok(v) => Ok(OscInt(v)),
			Err(e) => Err(e)
		},
		'f' => match reader.read_be_f32() {
			Ok(v) => Ok(OscFloat(v)),
			Err(e) => Err(e)
		},
		's' => match read_null_term_string(reader) {
			Ok(v) => Ok(OscStr(v)),
			Err(e) => Err(e)
		},
		'b' => match read_blob(reader) {
			Ok(v) => Ok(OscBlob(v)),
			Err(e) => Err(e)
		},
		_ 	=> Err(IoError{kind: InvalidInput, desc: "Invalid type tag.", detail: Some(type_tag.to_str())})
	}
}

// read a blob
fn read_blob(reader: &mut BufReader) -> IoResult<Vec<u8>> {
	let len;
	match reader.read_be_i32() {
		Ok(v) => {len = v as uint;},
		Err(e) => {return Err(e);}
	}

	match reader.read_exact(len) {
		Ok(v) => {
			reader.consume(four_byte_pad(len));
			Ok(v)
		},
		Err(e) => Err(e)
	}

}

// these tests would be a pain without the sender functions
#[test]
fn test_read_message(){

	let tmess = OscMessage {
		addr: "/test/addr".to_string(),
		args: vec!(
			OscInt(123),
			OscFloat(1.23),
			OscStr("teststr".to_string()),
			OscBlob(vec!(0u8, 5u8, 10u8)))
	};

	let buf = packet_to_buffer(tmess.clone());
	let resmess = read_message(buf.slice_from(4)).unwrap();

	assert_eq!(tmess,resmess);
}

#[test]
fn test_read_bundle(){
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

	let buf = packet_to_buffer(packet.clone());
	let res = read_bundle(buf.slice_from(4)).unwrap();

	assert_eq!(packet,res);
}


#[test]
fn test_read_null_term_string(){

	// remember that Osc strings are always multiples of 4 bytes!

	// simple case
	let buf1 = ~[97u8, 98u8, 99u8, 0u8];
	let mut reader = BufReader::new(buf1.as_slice());
	assert_eq!(read_null_term_string(&mut reader),Ok("abc".to_string()));

	// multiple nulls and multiple calls
	let buf2 = ~[97u8, 98u8, 0u8, 0u8, 99u8, 0u8];
	reader = BufReader::new(buf2.as_slice());
	assert_eq!(read_null_term_string(&mut reader),Ok("ab".to_string()));
	assert_eq!(read_null_term_string(&mut reader),Ok("c".to_string()));
	assert!(read_null_term_string(&mut reader).is_err());


	// some corner cases
	let buf3 = ~[];
	reader = BufReader::new(buf3.as_slice());
	assert!(read_null_term_string(&mut reader).is_err());

	let buf4 = ~[0u8];
	reader = BufReader::new(buf4.as_slice());
	assert_eq!(read_null_term_string(&mut reader),Ok("".to_string()));
	assert!(read_null_term_string(&mut reader).is_err());

	let buf5 = ~[0u8, 0u8, 0u8, 0u8, 0u8];
	reader = BufReader::new(buf5.as_slice());
	assert_eq!(read_null_term_string(&mut reader),Ok("".to_string()));
	assert_eq!(read_null_term_string(&mut reader),Ok("".to_string()));
	assert!(read_null_term_string(&mut reader).is_err());
}


#[test]
fn test_read_blob(){

	let mut buf = MemWriter::new();
	buf.write_be_i32(9);
	buf.write( vec!(0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 10u8, 15u8, 20u8, 0u8, 0u8, 0u8).as_slice() );

	let tbuf = buf.unwrap();

	let mut treader = BufReader::new(tbuf.as_slice());

	let res = read_blob(&mut treader).unwrap();

	assert_eq!(res, vec!(0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 10u8, 15u8, 20u8 ));
	assert!(treader.eof());


}
