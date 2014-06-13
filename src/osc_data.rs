#![macro_escape]

// there are really two kinds of messages in OSC, they are either a message or a bundle
#[deriving(Show,Clone,PartialEq,PartialOrd)]
pub enum OscPacket {
	// a message is the real meat
	OscMessage{
		pub addr: String,
		pub args: Vec<OscArg>
	},
	// a bundle is intended to synchronize multiple commands
	// essentially it bundles together multiple OSC packets
	OscBundle{
		pub time_tag: OscTimeTag,
		pub conts: Vec<OscPacket>
	}
}

/// Find out if a packet contains a specified OSC address.
pub fn packet_has_addr(packet: &OscPacket, addr_match: &str) -> bool {
	match *packet {
		OscMessage{addr: ref addr, args: _} => addr_match == addr.as_slice(),
		OscBundle{time_tag: _, conts: ref conts} => {
			for subpacket in conts.iter() {
				if packet_has_addr(subpacket, addr_match) { return true; }
			}
			false
		}
	}
}

/// Get the args associated with the given address.
pub fn get_args_with_addr(packet: OscPacket, addr_match: &str) -> Option<Vec<OscArg>> {

	match packet {
		OscMessage{addr: addr, args: args} => {
			if addr_match == addr.as_slice() {
				Some(args)
			}
			else {
				None
			}
		},
		OscBundle{ time_tag: _, conts: conts} => {
			let mut arg_vec = Vec::new();
			for subpacket in conts.move_iter() {
				match get_args_with_addr(subpacket, addr_match) {
					Some(a) => arg_vec.push_all_move(a),
					None => ()
				}
			}
			if arg_vec.is_empty() {
				return None;
			}
			Some(arg_vec)
		}
	}
}

// enum to contain the allowed OSC argument types
// this is the 1.0 specification, may want to look into 1.1 in the future
#[deriving(Show,Clone,PartialEq,PartialOrd)]
pub enum OscArg {
	OscInt(i32),
	OscFloat(f32),
	OscStr(String),
	OscBlob(Vec<u8>)
	/* OSC 1.1 stuff
	OscInt64(i64),
	OscFloat64(f64),
	OscTime(OscTimeTag),
	OscSymbol(~str),
	OscChar(Ascii),
	OscColor((u8, u8, u8, u8)),
	OscMidi(MidiMessage),
	OscAssert(OscAssertion),
	OscArray(Vec<OscArg>)
	*/
}

// some placeholders for possible eventual OSC 1.1 support
/*
enum OscAssertion {
	True,
	False,
	Nil,
	Infinitum
}

struct MidiMessage {
	port_id: u8,
	status_byte: u8,
	data1: u8,
	data2: u8
}
*/

/// Convert an OscArg to its correpsonding type tag character
pub fn arg_to_type_tag(arg: &OscArg) -> char {
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

// experimental macro for making dealing with osc args easier
/// Helper macro to check if an OscArg is a given type, produces a bool
#[macro_export]
macro_rules! arg_is_type(
	($arg:ident, $targ_var:ident) => (
		match $arg {
			$targ_var(_) => true,
			_ => false
		}
	)
)


// experimental macro for making dealing with osc args easier
/// Helper macro to unwrap an OscArg as a given type, produces None if the types don't match
#[macro_export]
macro_rules! unwrap_if(
	($arg:ident is $kind:ident) => (
		match $arg {
			$kind(v) => Some(v),
			_ => None
		}
	)
)

pub type OscTimeTag = (u32, u32);


#[test]
fn test_packet_has_addr(){
	let p1 = OscMessage{addr: "hello/test/address".to_string(), args: vec!(OscInt(0))};
	let p2 = OscBundle{
		time_tag: (0,1),
		conts: vec!(
			OscMessage{addr: "hello/another/test".to_string(), args: vec!(OscFloat(1.0))},
			OscMessage{addr: "whatwhat/test/again".to_string(), args: vec!(OscStr("payload".to_string()))}
			)
	};

	assert!(packet_has_addr(&p1, "hello/test/address"));
	assert!(packet_has_addr(&p2, "hello/another/test"));
	assert!(packet_has_addr(&p2, "whatwhat/test/again"));

	assert!(! packet_has_addr(&p1, "ouch"));
	assert!(! packet_has_addr(&p2, "wooooo"));
	assert!(! packet_has_addr(&p2, ""));
}

#[test]
fn test_get_args_with_addr(){
	let p1 = OscMessage{addr: "hello/test/address".to_string(), args: vec!(OscInt(123), OscFloat(1.0), OscStr("I am a test string".to_string()))};
	let p2 = OscBundle{
		time_tag: (0,1),
		conts: vec!(
			OscMessage{addr: "hello/another/test".to_string(), args: vec!(OscFloat(3.0), OscFloat(1.5))},
			OscMessage{addr: "whatwhat/test/again".to_string(), args: vec!(OscStr("payload".to_string()))}
			)
	};

	let p3 = OscBundle{
		time_tag: (0,1),
		conts: vec!(
			OscMessage{addr: "double/addr/test".to_string(), args: vec!(OscFloat(3.0), OscFloat(1.5))},
			OscMessage{addr: "double/addr/test".to_string(), args: vec!(OscStr("payload".to_string()))}
			)
	};

	assert_eq!(get_args_with_addr(p1.clone(), "hello/test/address"), Some(vec!(OscInt(123), OscFloat(1.0), OscStr("I am a test string".to_string()))));
	assert_eq!(get_args_with_addr(p1, "hello"), None);

	assert_eq!(get_args_with_addr(p2.clone(), "hello/another/test"), Some(vec!(OscFloat(3.0), OscFloat(1.5))));
	assert_eq!(get_args_with_addr(p2.clone(), "whatwhat/test/again"), Some(vec!(OscStr("payload".to_string()))));
	assert_eq!(get_args_with_addr(p2.clone(), "whatwhat"), None);

	assert_eq!(get_args_with_addr(p3, "double/addr/test"), Some(vec!(OscFloat(3.0), OscFloat(1.5), OscStr("payload".to_string()))));
}