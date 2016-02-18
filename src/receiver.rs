//! Module for receiving OSC over a UDP socket.

extern crate std;
extern crate byteorder;

use std::net::UdpSocket;
use std::net::SocketAddrV4;
use std::net::ToSocketAddrs;

use std::io::{Error, Result, BufReader, BufWriter};
use std::io::ErrorKind::{InvalidInput};
use std::str::*;
use std::io::prelude::*;

// support for reading raw binary streams into numbers, for testing
use self::byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use self::byteorder::Error::{Io, UnexpectedEOF};

use std::time::Duration;

use data::*;
use data::OscPacket::*;
use data::OscArg::*;

use sender::packet_to_buffer;

use util::*;

// Max size of UDP buffer; apparently 1536 is a common UDP MTU.
const UDP_BUFFER_SIZE: usize = 1536;

// static BUNDLE_ID: &'static str = "#bundle"; // perhaps we may want to use this some other day
const BUNDLE_FIRST_CHAR: char = '#';

// smallest packet is a 0 character address (4 bytes) and a comma for type tag (4 bytes)
const MIN_OSC_PACKET_SIZE: usize = 8;
const PACKET_SIZE_ERR: &'static str = "Packet with less than 8 bytes.";

// we may want to generalize this beyond UDP later
/// Structure which contains the port used to receive Osc packets, and handles
/// the task of interpreting those packets as valid Osc.
pub struct OscReceiver {

	socket: UdpSocket

}

impl OscReceiver {

	/// Constructs a new OscReceiver using a socket address.  Returns Err if an
	/// error occurred when trying to bind to the socket.
	pub fn new<T:ToSocketAddrs>(addr: T) -> Result<OscReceiver> {
		match UdpSocket::bind(addr) {
		    Ok(s) => Ok(OscReceiver{socket: s}),
		    Err(e) => return Err(e),
		}
	}

	/// Receive a Osc packet.  Blocks until a packet is available at the port.
	/// Can optionally specify a timeout on the blocking read.
	pub fn recv(&self, timeout: Option<Duration>) -> Result<OscPacket> {

		// initialize a receive buffer
		let buf = &mut[0; UDP_BUFFER_SIZE];

		let packet_len;

		try!(self.socket.set_read_timeout(timeout));

		match self.socket.recv_from(buf) {
			// ignoring source address from now, can bind it here if desired
			// if we didn't receive enough data, throw an error
			Ok((num, _)) if num < MIN_OSC_PACKET_SIZE => {
				return Err(Error::new(InvalidInput, PACKET_SIZE_ERR));
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

		read_packet(&buf[..packet_len])
	}
}

// interpret a buffer as an Osc packet; useful as bundles are recursive
fn read_packet(buf: &[u8]) -> Result<OscPacket> {
	if is_bundle(buf) {
		read_bundle(buf)
	}
	else {
		read_message(buf)
	}
}

// check if the message is a bundle by comparing the first character
fn is_bundle(buf: &[u8]) -> bool {
    buf[0] as char == BUNDLE_FIRST_CHAR
}

// read the buffer as a bundle, assuming proper OSC formatting
fn read_bundle(buf: &[u8]) -> Result<OscPacket> {
	let mut reader = &mut BufReader::new(buf);

	// ignore 8 byte bundle ID string
	reader.take(8).read_to_end(&mut vec!());;

	// read the 64 bit time tag
	let (sec, frac_sec): OscTimeTag;

	match reader.read_u32::<BigEndian>() {
		Ok(v) => sec = v,
		Err(Io(e)) => return Err(e),
        Err(UnexpectedEOF) => return Err(Error::new(InvalidInput, UnexpectedEOF))
	}

	match reader.read_u32::<BigEndian>() {
		Ok(v) => frac_sec = v,
		Err(Io(e)) => return Err(e),
        Err(UnexpectedEOF) => return Err(Error::new(InvalidInput, UnexpectedEOF))
	}

	// now interpret the bundle contents
	let mut bundle_conts = Vec::new();

	// until we're out of buffer, read elements
	let mut element_size: u32;
	loop {
		// get the length of the bundle element, should be a mult of 4
		match reader.read_i32::<BigEndian>() {
			Ok(n) => element_size = n as u32,
            Err(Io(e)) => return Err(e),
            Err(UnexpectedEOF) =>
                //  End of bundle
                break
		}

        // try to read the specified length
        let vec = &mut Vec::new();
        match reader.take(element_size as u64).read_to_end(vec) {
            Ok(num_read) => {
                if num_read == element_size as usize {
                    // if we got a valid vector, interpret it as a Osc packet
                    match read_packet(vec.as_ref()) {
                        Ok(pack) => bundle_conts.push(pack),
                        Err(e) => return Err(e)
                    }
                } else {
                    break
                }
            }
            _ => break
        }


	}

	Ok(OscBundle{time_tag: (sec, frac_sec), conts: bundle_conts})
}

// interpret a byte array as an Osc message
fn read_message(buf: &[u8]) -> Result<OscPacket> {

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
	if tt_str.chars().nth(0) != Some(',') {
		return Err(Error::new(InvalidInput, "Missing type tag comma."));
	}

	// now read the arguments
	let mut args = Vec::with_capacity(tt_str.len() - 1);

	// iterate over the args, skipping the comma ID
	for tt in tt_str.chars().skip(1) {
		match read_osc_arg(&mut reader, tt) {
			Ok(arg) => { args.push(arg); },
			Err(e) => { return Err(e); }
		}
	}

    Ok(OscMessage{addr: addr, args: args})
}

// Osc strings are null-terminated, we'll do this a lot
fn read_null_term_string(reader: &mut BufReader<&[u8]>) -> Result<String> {
	// read until null
    let m = &mut Vec::new();
	match reader.read_until(0u8, m) {
        Ok(n) => {
            if n == 0 {
                return Err(Error::new(InvalidInput, "No string to read."));
            }
        }
        Err(e) => return Err(e)
    }

    // Osc strings are always multiples of 4 bytes; read a few more if we didn't get a mult of 4
    reader.take(four_byte_pad(m.len()) as u64).read_to_end(&mut vec!());

    m.pop(); // remove the trailing null
    // try to convert to a string
    match std::str::from_utf8(m.as_ref()) {
        Ok(a) => {
            Ok(String::from(a))
        },
        // return an error if we can't parse this as a string
        Err(e) => {
            Err(Error::new(InvalidInput, e))
        }
    }
}

// read an osc argument based on a type tag
fn read_osc_arg(reader: &mut BufReader<&[u8]>, type_tag: char) -> Result<OscArg> {

	match type_tag {
		'i' => match reader.read_i32::<BigEndian>() {
			Ok(v) => Ok(OscInt(v)),
			Err(Io(e)) => Err(e),
            Err(UnexpectedEOF) => return Err(Error::new(InvalidInput, UnexpectedEOF))
		},
		'f' => match reader.read_f32::<BigEndian>() {
			Ok(v) => Ok(OscFloat(v)),
			Err(Io(e)) => Err(e),
            Err(UnexpectedEOF) => return Err(Error::new(InvalidInput, UnexpectedEOF))
		},
		's' => match read_null_term_string(reader) {
			Ok(v) => Ok(OscStr(v)),
			Err(e) => Err(e),
		},
		'b' => match read_blob(reader) {
			Ok(v) => Ok(OscBlob(v)),
			Err(e) => Err(e),
		},
		_ 	=> Err(Error::new(InvalidInput, format!("Invalid type tag {}", type_tag.to_string()) ))
	}
}

// read a blob
fn read_blob(reader: &mut BufReader<&[u8]>) -> Result<Vec<u8>> {
	let len : u32;
	match reader.read_i32::<BigEndian>() {
		Ok(v) => {len = v as u32;},
		Err(Io(e)) => {return Err(e);},
        Err(UnexpectedEOF) => return Err(Error::new(InvalidInput, UnexpectedEOF))
	}

    let mut vec = Vec::new();
    match reader.take(len as u64).read_to_end(&mut vec) {
		Ok(len) => {
			reader.consume(four_byte_pad(len));
			Ok(vec)
		},
		Err(e) => Err(e),
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
	let resmess = read_message(&buf[4..]).unwrap();

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
	let res = read_bundle(&buf[4..]).unwrap();

	assert_eq!(packet,res);
}


#[test]
fn test_read_null_term_string(){

	// remember that Osc strings are always multiples of 4 bytes!

	// simple case
	let buf1 = &[97u8, 98u8, 99u8, 0u8];
	let mut reader1 = BufReader::new(&buf1[..]);
	assert_eq!(read_null_term_string(&mut reader1).unwrap(),"abc".to_string());

	// multiple nulls and multiple calls
	let buf2 = &[97u8, 98u8, 0u8, 0u8, 99u8, 0u8];
	let mut reader2 = BufReader::new(&buf2[..]);
	assert_eq!(read_null_term_string(&mut reader2).unwrap(),"ab".to_string());
	assert_eq!(read_null_term_string(&mut reader2).unwrap(),"c".to_string());
	assert!(read_null_term_string(&mut reader2).is_err());


	// some corner cases
	let buf3 = [];
	let mut reader3 = BufReader::new(&buf3[..]);
	assert!(read_null_term_string(&mut reader3).is_err());

	let buf4 = [0u8];
	let mut reader4 = BufReader::new(&buf4[..]);
	assert_eq!(read_null_term_string(&mut reader4).unwrap(),"".to_string());
	assert!(read_null_term_string(&mut reader4).is_err());

	let buf5 = [0u8, 0u8, 0u8, 0u8, 0u8];
	let mut reader5 = BufReader::new(&buf5[..]);
	assert_eq!(read_null_term_string(&mut reader5).unwrap(),"".to_string());
	assert_eq!(read_null_term_string(&mut reader5).unwrap(),"".to_string());
	assert!(read_null_term_string(&mut reader5).is_err());
}


#[test]
fn test_read_blob(){

	let mut buf = BufWriter::new(vec!());
	buf.write_i32::<BigEndian>(9);
	buf.write( vec!(0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 10u8, 15u8, 20u8, 0u8, 0u8, 0u8).as_ref() );
     buf.flush();

	let tbuf = buf.get_ref();

	let mut treader = BufReader::new(tbuf.as_ref());
     let mut teststr = String::new();

	let res = read_blob(&mut treader).unwrap();

     treader.read_to_string(&mut teststr);

	assert_eq!(res, vec!(0u8, 1u8, 2u8, 3u8, 4u8, 5u8, 10u8, 15u8, 20u8 ));
	assert_eq!(teststr.len(), 0usize);


}
