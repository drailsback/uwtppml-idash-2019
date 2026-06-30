
pub mod protocol {

use crate::utility::*;
use crate::constants::*;
use crate::init::*;

use std::time::{SystemTime};
//use std::time;
//use std::thread;

use std::num::Wrapping;

const DEBUG: bool       = constants::DEBUG;
const BATCH_SIZE: usize = constants::BATCH_SIZE;
const BATCH_SIZE_REVEAL: usize = constants::REVEAL_BATCH_SIZE;

const CR_0: (Wrapping<u64>, Wrapping<u64>, Wrapping<u64>) = constants::CR_0;
const CR_1: (Wrapping<u64>, Wrapping<u64>, Wrapping<u64>) = constants::CR_1;

const CR_BIN_0: (u64, u64, u64) = constants::CR_BIN_0;
const CR_BIN_1: (u64, u64, u64) = constants::CR_BIN_1;

/*
Takes a list of secret-shared u64's with LSB 0 or 1 and all other bits 0.
Returns a list of u64's whose sum modulo 2^64 is equal to the LSB of the input. 
*/
pub fn binary_vector_to_ring( x_list_u64: &Vec<u64>,
					  			ctx:     &mut init::Context ) -> Vec<Wrapping<u64>> {

	let len = (*x_list_u64).len(); 

	let mut x_list 		 : Vec<Wrapping<u64>> = vec![ Wrapping(0) ; len ];
	let dummy            : Vec<Wrapping<u64>> = vec![ Wrapping(0) ; len ];
	let mut x_list_ring  : Vec<Wrapping<u64>> = vec![ Wrapping(0) ; len ];
	let mut product_list : Vec<Wrapping<u64>>;

	for i in 0..len {
		x_list[i] = Wrapping(x_list_u64[i]); 
	}

	if (*ctx).asymmetric_bit == 1 {
		product_list = batch_multiply( &x_list, &dummy, ctx);
	} else {
		product_list = batch_multiply( &dummy, &x_list, ctx );
	}

	for i in 0..len {
		x_list_ring[i] = x_list[i] - Wrapping(2)*product_list[i];
	} 

	x_list_ring

}

/* takes a list of u64's, each of which are xor-shared, and returns
   additive-shared versions of the same values */
pub fn xor_share_to_additive( x_list_u64: &Vec<u64>, ctx: &mut init::Context, size: usize )
	-> Vec<Wrapping<u64>> {

	let len = x_list_u64.len();
	let mut x_additive_list = vec![ Wrapping(0u64) ; len ];
	
	for i in 0..len {
		
		let mut bin_list = vec![ 0u64 ; size ];

		for j in 0..size {
			bin_list[j] = (x_list_u64[i] >> j) & 1u64;
		} 

		let bin_list_ring = binary_vector_to_ring(&bin_list, ctx);

		let mut bitselect = Wrapping(1u64);
		for j in 0..size {
			x_additive_list[i] += bitselect * bin_list_ring[j];
			bitselect <<= 1;
		}
		
	}

	x_additive_list

}

pub fn dot_product( x_list            : &Vec<Wrapping<u64>>, 
     				y_list            : &Vec<Wrapping<u64>>,
	   			    ctx               : &mut init::Context, 
	   			    decimal_precision : u32,
	   			    truncate          : bool,
	   			    pretruncate       : bool ) -> Wrapping<u64> {

	let z_list = batch_multiply(x_list, y_list, ctx);

	if !truncate {
		return z_list.iter().sum()
	}

	if !pretruncate {
		return utility::truncate_local( 
					z_list.iter().sum(), decimal_precision, (*ctx).asymmetric_bit 
				)
	}
	

	let mut z_trunc_list = vec![ Wrapping(0) ; z_list.len() ];
	for i in 0..z_list.len() {
		z_trunc_list[i] = utility::truncate_local(
							z_list[i], decimal_precision, (*ctx).asymmetric_bit
						  );
	}
	z_trunc_list.iter().sum()

}


pub fn batch_multiply( x_list: &Vec<Wrapping<u64>>, y_list: &Vec<Wrapping<u64>>,
				   ctx: &mut init::Context ) -> Vec<Wrapping<u64>> {

	let mut z_list: Vec<Wrapping<u64>> = vec![ Wrapping(0) ; (*x_list).len()];

	let mut remainder = (*x_list).len();
	let mut index = 0;
	while remainder > BATCH_SIZE {

		if DEBUG { println!("[index={}][remainder={}]", index, remainder); }
		let mut x_sublist = [ Wrapping(0) ; BATCH_SIZE];
		let mut y_sublist = [ Wrapping(0) ; BATCH_SIZE];

		x_sublist.clone_from_slice(&(x_list[BATCH_SIZE*index..BATCH_SIZE*(index+1)]));
		y_sublist.clone_from_slice(&(y_list[BATCH_SIZE*index..BATCH_SIZE*(index+1)]));

		let z_sublist = batch_multiplication_submodule(x_sublist, y_sublist, BATCH_SIZE, ctx);

		z_list[BATCH_SIZE*index..BATCH_SIZE*(index+1)].clone_from_slice(&z_sublist);

		remainder -= BATCH_SIZE;
		index += 1;
	}

	if DEBUG {println!("[index={}][remainder={}]", index, remainder);}
	let mut x_sublist = [ Wrapping(0) ; BATCH_SIZE];
	let mut y_sublist = [ Wrapping(0) ; BATCH_SIZE];

	x_sublist[0..remainder].clone_from_slice(&(x_list[BATCH_SIZE*index..]));
	y_sublist[0..remainder].clone_from_slice(&(y_list[BATCH_SIZE*index..]));

	let z_sublist = batch_multiplication_submodule(x_sublist, y_sublist, remainder, ctx);

	z_list[BATCH_SIZE*index..].clone_from_slice(&(z_sublist[..remainder]));

	z_list

}

pub fn batch_multiplication_submodule( x_list : [Wrapping<u64> ; BATCH_SIZE], 
	               					   y_list : [Wrapping<u64> ; BATCH_SIZE],
	               					   tx_len : usize, 
	               					   ctx    : &mut init::Context)  -> [Wrapping<u64> ; BATCH_SIZE] {

	let rx_buffer = &ctx.rx_buffer;
	let tx_buffer = &ctx.tx_buffer;
	let wake_tx   = &ctx.wake_tx;
	let wake_rx   = &ctx.wake_rx;
	let wake_op   = &ctx.wake_op;

	let asymmetric_bit = Wrapping(ctx.asymmetric_bit as u64);
	
	let mut u_list: [ Wrapping<u64> ; BATCH_SIZE ] = [ Wrapping(0) ; BATCH_SIZE ];
	let mut v_list: [ Wrapping<u64> ; BATCH_SIZE ] = [ Wrapping(0) ; BATCH_SIZE ];
	let mut w_list: [ Wrapping<u64> ; BATCH_SIZE ] = [ Wrapping(0) ; BATCH_SIZE ];
	let mut d_list: [ Wrapping<u64> ; BATCH_SIZE ] = [ Wrapping(0) ; BATCH_SIZE ];
	let mut e_list: [ Wrapping<u64> ; BATCH_SIZE ] = [ Wrapping(0) ; BATCH_SIZE ];
	let mut z_list: [ Wrapping<u64> ; BATCH_SIZE ] = [ Wrapping(0) ; BATCH_SIZE ];
	
	for i in 0..tx_len {
		
		let (u, v, w) = if ctx.asymmetric_bit == 1 {CR_1} else {CR_0};//ctx.corr_rand.pop().unwrap();

		u_list[i] = u;
		v_list[i] = v;
		w_list[i] = w;

		d_list[i] = x_list[i] - u;
		e_list[i] = y_list[i] - v;
	}

	{ // acquire tx_buffer mutex
		let mut tx_buffer = tx_buffer.lock().unwrap();
		
		for i in (0..2*tx_len).step_by(2) {

			let d = d_list[i/2].0;
			let e = e_list[i/2].0;
			
			(*tx_buffer)[     8*i..8*(i+1) ].clone_from_slice(&utility::u64_to_byte_array(d));
			(*tx_buffer)[ 8*(i+1)..8*(i+2) ].clone_from_slice(&utility::u64_to_byte_array(e));
		}
	} // give tx_buffer mutex
	//thread::sleep(time::Duration::from_millis(1));
	{ // notify client thread to send tx_buffer
		if DEBUG {println!("MAIN THREAD: notifying client tx_buffer is ready");}
		let &(ref lock, ref cvar) = &**wake_tx;
		let mut transmit = lock.lock().unwrap();
		*transmit = true;
		cvar.notify_one();
		if DEBUG {println!("MAIN THREAD: client notified");}
	}
	//thread::sleep(time::Duration::from_millis(1));

	{ // notify server thread to recv rx_buffer
		if DEBUG {println!("MAIN THREAD: notifying server rx_buffer is ready");}
		let &(ref lock, ref cvar) = &**wake_rx;
		let mut receive = lock.lock().unwrap();
		*receive = true;
		cvar.notify_one();
		if DEBUG {println!("MAIN THREAD: server notified");}
	}
	//thread::sleep(time::Duration::from_millis(1));

	{
		if DEBUG {println!("MAIN THREAD: waiting to read rx buffer");}
       	let &(ref lock, ref cvar) = &**wake_op;
       	let mut read_ok = lock.lock().unwrap();
        while !*read_ok {		              
 			read_ok = cvar.wait(read_ok).unwrap();
 		}
 		*read_ok = false;
	}
	// thread::sleep(time::Duration::from_millis(1));
	{ // acquire rx_buffer mutex:
		if DEBUG {println!("MAIN THREAD: waiting to acquire rx buffer mutex");}
		let rx_buffer = rx_buffer.lock().unwrap();
		if DEBUG {println!("MAIN THREAD: rx_buffer mutex acquired");}
		
	
		for i in (0..2*tx_len).step_by(2) {
			
			let mut d_buf = [ 0 as u8 ; 8];
			let mut e_buf = [ 0 as u8 ; 8];
		
			d_buf.clone_from_slice( &(*rx_buffer)[     8*i..8*(i+1) ] );
			e_buf.clone_from_slice( &(*rx_buffer)[ 8*(i+1)..8*(i+2) ] );

			let d = d_list[i/2] + Wrapping(utility::byte_array_to_u64( d_buf ));
			let e = e_list[i/2] + Wrapping(utility::byte_array_to_u64( e_buf ));

			let u = u_list[i/2];
			let v = v_list[i/2];
			let w = w_list[i/2];

			z_list[i/2] = w + d*v + u*e + d*e*asymmetric_bit;

		}
	}
	//println!("{:?}", z_list.to_vec());
	z_list
}




pub fn reveal(x_list: &Vec<Wrapping<u64>>, 
			  ctx: &mut init::Context, 
			  decimal_precision: u32,
			  to_real: bool 
			  ) -> Vec<f64> {

	let len = (*x_list).len();
	let mut x_combined: Vec<Wrapping<u64>> = vec![ Wrapping(0) ; len];
	let mut x_revealed: Vec<f64> = vec![ 0.0 ; len ];
	let mut remainder = len.clone();
	let mut index = 0;

	while remainder > BATCH_SIZE_REVEAL {

		if DEBUG { println!("[index={}][remainder={}]", index, remainder); }
		
		let mut x_sublist = [ Wrapping(0) ; BATCH_SIZE_REVEAL];
		x_sublist.clone_from_slice(&(x_list[BATCH_SIZE_REVEAL*index..BATCH_SIZE_REVEAL*(index+1)]));
		
		let x_combined_sublist = reveal_submodule(x_sublist, BATCH_SIZE_REVEAL, ctx);

		x_combined[BATCH_SIZE_REVEAL*index..BATCH_SIZE_REVEAL*(index+1)].clone_from_slice(&x_combined_sublist);

		remainder -= BATCH_SIZE_REVEAL;
		index += 1;
	}

	if DEBUG {println!("[index={}][remainder={}]", index, remainder);}
	
	let mut x_sublist = [ Wrapping(0) ; BATCH_SIZE_REVEAL];
	x_sublist[0..remainder].clone_from_slice(&(x_list[BATCH_SIZE_REVEAL*index..]));

	let x_combined_sublist = reveal_submodule(x_sublist, BATCH_SIZE_REVEAL, ctx);
	x_combined[BATCH_SIZE_REVEAL*index..].clone_from_slice(&(x_combined_sublist[..remainder]));

	if to_real {
		for i in 0..len {
			
			let x = x_combined[i];

			if ( x.0 >> 63 ) == 0 {// TODO replace 63 with named constant RINGSIZE-1

				x_revealed[i] = (x.0 as f64) / (2u64.pow(decimal_precision) as f64); 
			
			} else {

				x_revealed[i] = -1.0 * ( (-x).0 as f64 ) / (2u64.pow(decimal_precision) as f64);
			}

		}
	} else {

		for i in 0..len {
			x_revealed[i] = x_combined[i].0 as f64;
		}
	}


	x_revealed
}


pub fn reveal_submodule( x_list : [Wrapping<u64> ; BATCH_SIZE_REVEAL], 
	               		 tx_len : usize, 
	               		 ctx    : &mut init::Context)  -> [Wrapping<u64> ; BATCH_SIZE_REVEAL] {

	let rx_buffer = &ctx.rx_buffer;
	let tx_buffer = &ctx.tx_buffer;
	let wake_tx   = &ctx.wake_tx;
	let wake_rx   = &ctx.wake_rx;
	let wake_op   = &ctx.wake_op;

	let mut x_revealed: [ Wrapping<u64> ; BATCH_SIZE_REVEAL ] = [ Wrapping(0) ; BATCH_SIZE_REVEAL ];

	{ // acquire tx_buffer mutex
		let mut tx_buffer = tx_buffer.lock().unwrap();
		
		for i in 0..tx_len {
			let x = x_list[i].0;
			(*tx_buffer)[8*i..8*(i+1)].clone_from_slice(&utility::u64_to_byte_array(x));
		}
	} // give tx_buffer mutex
	
	{ // notify client thread to send tx_buffer
		if DEBUG {println!("MAIN THREAD: notifying client tx_buffer is ready");}
		let &(ref lock, ref cvar) = &**wake_tx;
		let mut transmit = lock.lock().unwrap();
		*transmit = true;
		cvar.notify_one();
		if DEBUG {println!("MAIN THREAD: client notified");}
	}


	{ // notify server thread to recv rx_buffer
		if DEBUG {println!("MAIN THREAD: notifying server rx_buffer is ready");}
		let &(ref lock, ref cvar) = &**wake_rx;
		let mut receive = lock.lock().unwrap();
		*receive = true;
		cvar.notify_one();
		if DEBUG {println!("MAIN THREAD: server notified");}
	}


	{
		if DEBUG {println!("MAIN THREAD: waiting to read rx buffer");}
       	let &(ref lock, ref cvar) = &**wake_op;
       	let mut read_ok = lock.lock().unwrap();
        while !*read_ok {		              
 			read_ok = cvar.wait(read_ok).unwrap();
 		}
 		*read_ok = false;
	}

	{ // acquire rx_buffer mutex:
		if DEBUG {println!("MAIN THREAD: waiting to acquire rx buffer mutex");}
		let rx_buffer = rx_buffer.lock().unwrap();
		if DEBUG {println!("MAIN THREAD: rx_buffer mutex acquired");}
		
	
		for i in 0..tx_len {
			
			let mut x_other_buf = [ 0 as u8 ; 8];
			x_other_buf.clone_from_slice( &(*rx_buffer)[ 8*i..8*(i+1) ] );
			let x_other = Wrapping(utility::byte_array_to_u64( x_other_buf ));
			
			x_revealed[i] = x_list[i] + x_other;

		}
	}
	//println!("{:?}", z_list.to_vec());
	x_revealed
}

pub fn batch_bit_decomp(x_additive_list: &Vec<Wrapping<u64>>, ctx: &mut init::Context) -> Vec<u64> {

	let len = x_additive_list.len(); 

	let asymmetric_bit = (*ctx).asymmetric_bit;
	let inversion_mask: u64 = (- Wrapping(asymmetric_bit as u64)).0; 

	// println!("length: {}", len);
	// println!("asymmetric_bit: {}", asymmetric_bit);
	// println!("inversion_mask: {:x}", inversion_mask);

	let mut a_list: Vec<u64> = vec![0u64 ; len];
	let mut b_list: Vec<u64> = vec![0u64 ; len];
	let mut y_list: Vec<u64> = vec![0u64 ; len];
	let mut d_list: Vec<u64> = vec![0u64 ; len];
	
	let mut x_list: Vec<u64> = vec![0u64 ; len];

	for i in 0..len {
		
		a_list[i] = if asymmetric_bit == 1 { x_additive_list[i].0 } else { 0u64 };
		b_list[i] = if asymmetric_bit == 1 { 0u64 } else { x_additive_list[i].0 };
		y_list[i] = x_additive_list[i].0;

//		println!("[{}] a: {:x}, b: {:x}, y: {:x}", i, a_list[i], b_list[i], y_list[i]);
	}


	let ab_list = batch_bitwise_and(&a_list, &b_list, ctx, false);

	for i in 0..len {
		d_list[i] = ab_list[i] ^ inversion_mask;
		x_list[i] = y_list[i] & 1u64;
//		println!("[{}] ab: {:x}, d: {:x}, x: {:x}", i, ab_list[i], d_list[i], x_list[i]);
	}

	let mut c_slice = utility::u64_vec_to_slice(&ab_list, 0);
	let mut y_slice;// = Vec::new();
	let mut d_slice;// = Vec::new();
	let mut e_slice;// = Vec::new();

	for i in 1..64 {

		y_slice = utility::u64_vec_to_slice(&y_list, i);
		d_slice = utility::u64_vec_to_slice(&d_list, i);

		e_slice = batch_bitwise_and(&y_slice, &c_slice, ctx, true);

		for j in 0..len {
			let y = (y_slice[j / 64] >> ((Wrapping(63) - Wrapping(j)).0 % 64)) & 1u64;
			let c = (c_slice[j / 64] >> ((Wrapping(63) - Wrapping(j)).0 % 64)) & 1u64;
			x_list[j] |= (y ^ c) << i;
		}

		if i == 63 {
			break;
		}

		c_slice = batch_bitwise_and(&e_slice, &d_slice, ctx, true);

	}

	x_list
}


pub fn batch_bitwise_and( x_list: &Vec<u64>, 
						  y_list: &Vec<u64>, 
						  ctx: &mut init::Context,
						  invert_output: bool ) -> Vec<u64> {
	
	let len = (*x_list).len();
	let mut z_list: Vec<u64> = vec![ 0u64 ; len];

	let mut remainder = len;
	let mut index = 0;
	while remainder > BATCH_SIZE {

		let mut x_sublist = [ 0u64 ; BATCH_SIZE];
		let mut y_sublist = [ 0u64 ; BATCH_SIZE];

		x_sublist.clone_from_slice(&(x_list[BATCH_SIZE*index..BATCH_SIZE*(index+1)]));
		y_sublist.clone_from_slice(&(y_list[BATCH_SIZE*index..BATCH_SIZE*(index+1)]));

		let z_sublist = batch_bitwise_and_submodule(x_sublist, y_sublist, BATCH_SIZE, ctx);

		z_list[BATCH_SIZE*index..BATCH_SIZE*(index+1)].clone_from_slice(&z_sublist);

		remainder -= BATCH_SIZE;
		index += 1;
	}

	let mut x_sublist = [ 0u64 ; BATCH_SIZE];
	let mut y_sublist = [ 0u64 ; BATCH_SIZE];

	x_sublist[0..remainder].clone_from_slice(&(x_list[BATCH_SIZE*index..]));
	y_sublist[0..remainder].clone_from_slice(&(y_list[BATCH_SIZE*index..]));

	let z_sublist = batch_bitwise_and_submodule(x_sublist, y_sublist, remainder, ctx);

	z_list[BATCH_SIZE*index..].clone_from_slice(&(z_sublist[..remainder]));

	if invert_output {

		let inversion_mask = (- Wrapping((*ctx).asymmetric_bit as u64)).0;
		for i in 0..len {
			z_list[i] ^= inversion_mask;
		}
	}

	z_list	
}

pub fn batch_bitwise_and_submodule( x_list : [ u64 ; BATCH_SIZE], 
	               					y_list : [ u64 ; BATCH_SIZE],
	               					tx_len : usize, 
	               					ctx    : &mut init::Context)  -> [ u64 ; BATCH_SIZE] {

	let rx_buffer = &ctx.rx_buffer;
	let tx_buffer = &ctx.tx_buffer;
	let wake_tx   = &ctx.wake_tx;
	let wake_rx   = &ctx.wake_rx;
	let wake_op   = &ctx.wake_op;

	let asymmetric_bit = ctx.asymmetric_bit as u64;
	let inversion_mask: u64 = (- Wrapping(asymmetric_bit) ).0; 

	// println!("asymmetric_bit: {}", asymmetric_bit);
	// println!("inversion_mask: {:x}", inversion_mask);

	let mut u_list: [ u64 ; BATCH_SIZE ] = [ 0u64 ; BATCH_SIZE ];
	let mut v_list: [ u64 ; BATCH_SIZE ] = [ 0u64 ; BATCH_SIZE ];
	let mut w_list: [ u64 ; BATCH_SIZE ] = [ 0u64 ; BATCH_SIZE ];
	let mut d_list: [ u64 ; BATCH_SIZE ] = [ 0u64 ; BATCH_SIZE ];
	let mut e_list: [ u64 ; BATCH_SIZE ] = [ 0u64 ; BATCH_SIZE ];
	let mut z_list: [ u64 ; BATCH_SIZE ] = [ 0u64 ; BATCH_SIZE ];
	
	for i in 0..tx_len {
		
		let (u, v, w) = if ctx.asymmetric_bit == 1 {CR_BIN_1} else {CR_BIN_0};

		u_list[i] = u;
		v_list[i] = v;
		w_list[i] = w;

		d_list[i] = x_list[i] ^ u;
		e_list[i] = y_list[i] ^ v;

		// println!("x: {:x}", x_list[i]);
		// println!("y: {:x}", y_list[i]);

		// println!("u: {:x}", u);
		// println!("v: {:x}", v);
		// println!("w: {:x}", w);

		// println!("d: {:x}", d_list[i]);
		// println!("e: {:x}", e_list[i]);
	}

	{ // acquire tx_buffer mutex
		let mut tx_buffer = tx_buffer.lock().unwrap();
		
		for i in (0..2*tx_len).step_by(2) {

			let d = d_list[i/2];
			let e = e_list[i/2];
			
			(*tx_buffer)[     8*i..8*(i+1) ].clone_from_slice(&utility::u64_to_byte_array(d));
			(*tx_buffer)[ 8*(i+1)..8*(i+2) ].clone_from_slice(&utility::u64_to_byte_array(e));
		}
	} // give tx_buffer mutex
	
	{ // notify client thread to send tx_buffer
		if DEBUG {println!("MAIN THREAD: notifying client tx_buffer is ready");}
		let &(ref lock, ref cvar) = &**wake_tx;
		let mut transmit = lock.lock().unwrap();
		*transmit = true;
		cvar.notify_one();
		if DEBUG {println!("MAIN THREAD: client notified");}
	}


	{ // notify server thread to recv rx_buffer
		if DEBUG {println!("MAIN THREAD: notifying server rx_buffer is ready");}
		let &(ref lock, ref cvar) = &**wake_rx;
		let mut receive = lock.lock().unwrap();
		*receive = true;
		cvar.notify_one();
		if DEBUG {println!("MAIN THREAD: server notified");}
	}


	{
		if DEBUG {println!("MAIN THREAD: waiting to read rx buffer");}
       	let &(ref lock, ref cvar) = &**wake_op;
       	let mut read_ok = lock.lock().unwrap();
        while !*read_ok {		              
 			read_ok = cvar.wait(read_ok).unwrap();
 		}
 		*read_ok = false;
	}

	{ // acquire rx_buffer mutex:
		if DEBUG {println!("MAIN THREAD: waiting to acquire rx buffer mutex");}
		let rx_buffer = rx_buffer.lock().unwrap();
		if DEBUG {println!("MAIN THREAD: rx_buffer mutex acquired");}
		
		for i in (0..2*tx_len).step_by(2) {
			
			let mut d_buf = [ 0 as u8 ; 8];
			let mut e_buf = [ 0 as u8 ; 8];
		
			d_buf.clone_from_slice( &(*rx_buffer)[     8*i..8*(i+1) ] );
			e_buf.clone_from_slice( &(*rx_buffer)[ 8*(i+1)..8*(i+2) ] );

			let d = d_list[i/2] ^ utility::byte_array_to_u64( d_buf );
			let e = e_list[i/2] ^ utility::byte_array_to_u64( e_buf );

			// println!("d-complete: {:x}", d);
			// println!("e-complete: {:x}", e);

			let u = u_list[i/2];
			let v = v_list[i/2];
			let w = w_list[i/2];

			z_list[i/2] = w ^ (d&v) ^ (u&e) ^ (d&e&inversion_mask);

		}
	}
	//println!("{:?}", z_list.to_vec());
	z_list
}



}