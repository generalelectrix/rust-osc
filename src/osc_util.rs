/// calculate how many more bytes we need to make the argument a multiple of four
pub fn four_byte_pad(len: usize) -> usize {
	let rem = len % 4;
	match rem {
		0 => 0,
		v => 4 - v
	}
}


// helper macro for this commonly-needed operation
macro_rules! pad_with_null {
	($operator:ident $operation:ident $n:expr) => (
		for _ in 0usize..four_byte_pad($n) {
			$operator.$operation(&[0u8]).unwrap();
		}
	)
}