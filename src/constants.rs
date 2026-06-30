
/*

Global constants. To use in a module,

(1) at the top of the module, 'use crate::constants::*'
(2) make local copies of constants to be used: 'const NAME = constants::NAME'

*/

pub mod constants {

use std::num::Wrapping;

// switch to true for verbose debug output
pub const DEBUG: bool = false;

// never change
pub const MULT_ELEMS: usize = 2;
pub const SIZEOF_U64: usize = 8;

// can tweak batch size to optimize batch multiplication for different machines
pub const BATCH_SIZE: usize = 4096;
pub const REVEAL_BATCH_SIZE: usize = 2 * BATCH_SIZE;
pub const BUF_SIZE: usize = BATCH_SIZE * MULT_ELEMS * SIZEOF_U64;

pub const CR_0: (Wrapping<u64>, Wrapping<u64>, Wrapping<u64>) = 
				(
					Wrapping(  6833008916512791354 ), 
					Wrapping( 14547997512572730844 ),
					Wrapping(  4912126131370984821 )
				); 

pub const CR_1: (Wrapping<u64>, Wrapping<u64>, Wrapping<u64>) = 
				(
					Wrapping( 2741451256696586090 ), 
					Wrapping( 8847773937267267195 ), 
					Wrapping( 7432801596505970759 )
				); 

pub const CR_BIN_0: (u64, u64, u64) = 
				(
					16985030433743349914,
					16800953460621756407,
					13341120138288862608
				);

pub const CR_BIN_1: (u64, u64, u64) = 
				(
					9026346012512690793,
					5254509534812310812,
					4172330766887821171
				);


}
			