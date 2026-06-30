

pub mod single_protocol {

use crate::utility::*;
use crate::constants::*;
use crate::init::*;

use std::time::{SystemTime};
use std::time;
use std::thread;

use std::num::Wrapping;

const DEBUG: bool       = constants::DEBUG;
const BATCH_SIZE: usize = constants::BATCH_SIZE;
const BATCH_SIZE_REVEAL: usize = constants::REVEAL_BATCH_SIZE;

const TEST_CORR_RAND_0: (Wrapping<u64>, Wrapping<u64>, Wrapping<u64>) = constants::TEST_CORR_RAND_0;
const TEST_CORR_RAND_1: (Wrapping<u64>, Wrapping<u64>, Wrapping<u64>) = constants::TEST_CORR_RAND_1;

const TEST_BIN_CORR_RAND_0: (u64, u64, u64) = constants::TEST_BIN_CORR_RAND_0;
const TEST_BIN_CORR_RAND_1: (u64, u64, u64) = constants::TEST_BIN_CORR_RAND_1;


pub fn multiply( x   : Wrapping<u64>, 
	             y   : Wrapping<u64>, 
	             ctx : &mut init::Context) -> Wrapping<u64> {

	let rx_buffer = &ctx.rx_buffer;
	let tx_buffer = &ctx.tx_buffer;
	let wake_tx   = &ctx.wake_tx;
	let wake_rx   = &ctx.wake_rx;
	let wake_op   = &ctx.wake_op;

	let asymmetric_bit = Wrapping(ctx.asymmetric_bit as u64);
	let (u, v, w)      = ctx.corr_rand.pop().unwrap();
	let mut d = x - u;
	let mut e = y - v;
	
	// println!("u: {}, v: {}, w: {}", u.0, v.0, w.0);
	// println!("x: {}, y: {}", x.0, y.0);
	// println!("d1: {}, e1: {}", d.0, e.0);
	{ // acquire tx_buffer mutex
		
		let mut tx_buffer = tx_buffer.lock().unwrap();
		for elem in (*tx_buffer).iter_mut() { *elem = 0; }
		(*tx_buffer)[0x00..0x08].clone_from_slice(&utility::u64_to_byte_array(d.0));
		(*tx_buffer)[0x08..0x10].clone_from_slice(&utility::u64_to_byte_array(e.0));
		
	} // give tx_buffer mutex
	
	{ // notify client thread to send tx_buffer
		if DEBUG {println!("MAIN THREAD: notifying client tx_bugger is ready");}
		let &(ref lock, ref cvar) = &**wake_tx;
		let mut transmit = lock.lock().unwrap();
		*transmit = true;
		cvar.notify_one();
		if DEBUG {println!("MAIN THREAD: client notified");}
		
	}

	{ // notify server thread to recv rx_buffer
		if DEBUG {println!("MAIN THREAD: notifying server rx_bugger is ready");}
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

		let mut d_buf = [ 0 as u8 ; 8];
		let mut e_buf = [ 0 as u8 ; 8];

		d_buf.clone_from_slice( &(*rx_buffer)[..8] );
		e_buf.clone_from_slice( &(*rx_buffer)[8..16] );

		d = d + Wrapping(utility::byte_array_to_u64( d_buf ));
		e = e + Wrapping(utility::byte_array_to_u64( e_buf ));

	} // give rx_buffer mutex

	w + d*v + u*e + d*e*asymmetric_bit

}



// 3*bitlength - 2 binary triples
pub fn bit_decompose(x_additive: Wrapping<u64>, ctx: &mut init::Context) -> u64 {

	let asymmetric_bit = (*ctx).asymmetric_bit;
	let x_prev = x_additive.0;
	/*			u v w     u0  v0  w0   u1  v1  w1
	bin shares: 0 0 0 --> 0/1 0/1 0/1, 0/1 0/1 0/1
	bin shares: 1 0 0 --> 0/1 0/1 0/1, 1/0 0/1 0/1
	bin shares: 0 1 0 --> 0/1 0/1 0/1, 0/1 1/0 0/1
	bin shares: 1 1 1 --> 0/1 0/1 0/1, 1/0 1/0 1/0 
	*/

	let inversion_mask: u64 = (- Wrapping(asymmetric_bit as u64)).0; 

	//println!("Inversion mask {:x}", inversion_mask);

	let a = if asymmetric_bit == 1 {x_prev} else {0u64};
	let b = if asymmetric_bit == 1 {0u64} else {x_prev};
	let y = a ^ b;
	
	//println!("a = {:x}, b = {:x}, y = {:x}", a, b, y);

	let mut c = 0u64;
	let mut d = 0u64;
	let mut e = 0u64;
	let mut x = 0u64;

	let ab = bitwise_and(a, b, ctx);

	let mut x_str = String::new();
	//println!("a & b = {:x}", ab);

	d |= ab ^ inversion_mask;
	
	let mut bit_select = 1u64;
	
	c |= ab & bit_select;
	x |= y & bit_select;

	x_str.push((((x&bit_select) + 0x30) as u8) as char);

	//println!("[0] y: {}, d: {}, c: {}, x: {}", y&bit_select, d&bit_select, c&bit_select, x&bit_select);

	for i in 1..64 {
		
		bit_select <<= 1;

		e |= (bitwise_and(y, c << 1, ctx) ^ inversion_mask) & bit_select;
		c |= (bitwise_and(e, d, ctx) ^ inversion_mask) & bit_select;
		x |= (y ^ (c << 1)) & bit_select;

		// println!("[{}] y: {}, d: {}, e: {}, c: {}, x: {}",
		// 	i, (y&bit_select)>>i,(d&bit_select)>>i, (e&bit_select)>>i, (c&bit_select)>>i, (x&bit_select)>>i);
		x_str.push(((((x&bit_select)>>i) + 0x30) as u8) as char); 
	}

	println!("{}", x_str);
	x
}


pub fn bitwise_and(x: u64, y: u64, ctx: &mut init::Context) -> u64 { 

	let rx_buffer = &ctx.rx_buffer;
	let tx_buffer = &ctx.tx_buffer;
	let wake_tx   = &ctx.wake_tx;
	let wake_rx   = &ctx.wake_rx;
	let wake_op   = &ctx.wake_op;

	let asymmetric_bit = ( - Wrapping(ctx.asymmetric_bit as u64)).0 ;
	let (u, v, w) = if asymmetric_bit == 0 {TEST_BIN_CORR_RAND_0} else {TEST_BIN_CORR_RAND_1};
	let mut d = x ^ u;
	let mut e = y ^ v;
	
	// println!("asymmetric_bit: {:x}", asymmetric_bit);
	// println!("d': {:x}", d);
	// println!("e': {:x}", e);

	{ // acquire tx_buffer mutex	
		let mut tx_buffer = tx_buffer.lock().unwrap();
		//for elem in (*tx_buffer).iter_mut() { *elem = 0; }
		(*tx_buffer)[0x00..0x08].clone_from_slice(&utility::u64_to_byte_array(d));
		(*tx_buffer)[0x08..0x10].clone_from_slice(&utility::u64_to_byte_array(e));
		
	} // give tx_buffer mutex
	
	{ // notify client thread to send tx_buffer
		if DEBUG {println!("MAIN THREAD: notifying client tx_buffer is ready");}
		let &(ref lock, ref cvar) = &**wake_tx;
		let mut transmit = lock.lock().unwrap();
		*transmit = true;
		cvar.notify_one();
		if DEBUG {println!("MAIN THREAD: client notified");}
		
	}
	thread::sleep(time::Duration::from_millis(1));
	{ // notify server thread to recv rx_buffer
		if DEBUG {println!("MAIN THREAD: notifying server rx_buffer is ready");}
		let &(ref lock, ref cvar) = &**wake_rx;
		let mut receive = lock.lock().unwrap();
		*receive = true;
		cvar.notify_one();
		if DEBUG {println!("MAIN THREAD: server notified");}
	}
	thread::sleep(time::Duration::from_millis(1));
	{
		if DEBUG {println!("MAIN THREAD: waiting to read rx buffer");}
       	let &(ref lock, ref cvar) = &**wake_op;
       	let mut read_ok = lock.lock().unwrap();
        while !*read_ok {		              
 			read_ok = cvar.wait(read_ok).unwrap();
 		}
 		*read_ok = false;
	}
	thread::sleep(time::Duration::from_millis(1));
	{ // acquire rx_buffer mutex:
		if DEBUG {println!("MAIN THREAD: waiting to acquire rx buffer mutex");}
		let rx_buffer = rx_buffer.lock().unwrap();
		if DEBUG {println!("MAIN THREAD: rx_buffer mutex acquired");}

		let mut d_buf = [ 0 as u8 ; 0x08];
		let mut e_buf = [ 0 as u8 ; 0x08];

		d_buf.clone_from_slice( &(*rx_buffer)[0x00..0x08] );
		e_buf.clone_from_slice( &(*rx_buffer)[0x08..0x10] );

		d = d ^ utility::byte_array_to_u64( d_buf );
		e = e ^ utility::byte_array_to_u64( e_buf );

		// println!("d : {:x}", d);
		// println!("e : {:x}", e);

	} // give rx_buffer mutex

	//println!("total: {:x}",w ^ (d & v) ^ (u & e) ^ (d & e & asymmetric_bit) );
	thread::sleep(time::Duration::from_millis(1));
	w ^ (d & v) ^ (u & e) ^ (d & e & asymmetric_bit)

}



}