

pub mod utility {

	use std::num::Wrapping;

	
	pub fn truncate_local( x                 : Wrapping<u64>, 
						   decimal_precision : u32, 
						   asymmetric_bit    : u8) -> Wrapping<u64> {

		if asymmetric_bit == 0 {

			return  - Wrapping( ( - x).0 >> decimal_precision )  
		}

		Wrapping( x.0 >> decimal_precision )
	}


	pub fn u64_to_byte_array(x: u64) -> [u8 ; 8] {

		[ ((x >> 56) & 0xff) as u8,
		  ((x >> 48) & 0xff) as u8,
		  ((x >> 40) & 0xff) as u8,
		  ((x >> 32) & 0xff) as u8,
		  ((x >> 24) & 0xff) as u8,
		  ((x >> 16) & 0xff) as u8,
		  ((x >>  8) & 0xff) as u8,
		          (x & 0xff) as u8 ]
	}

	pub fn byte_array_to_u64(buf: [u8; 8]) -> u64 {


		((buf[0] as u64) << 56) +
		((buf[1] as u64) << 48 )+
		((buf[2] as u64) << 40) +
		((buf[3] as u64) << 32) +
		((buf[4] as u64) << 24) +
		((buf[5] as u64) << 16) +
		((buf[6] as u64) <<  8) +
		 (buf[7] as u64) 
	}


	pub fn u64_vec_to_slice( list: &Vec<u64>, bit: u32 ) -> Vec<u64> {

		let len = list.len();

		let mut slice: Vec<u64> = 
			vec![ 0u64 ; (len / 64) + ( if len % 64 == 0 {0} else {1} ) ];

		for i in 0..len {
			slice[i/64] |= ((list[i] >> bit) & 1u64) << ((Wrapping(63) - Wrapping(i)).0 % 64);
		} 

		slice
	}

	
}