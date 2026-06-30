pub mod ti_receiver {

use crate::constants::*;
use crate::init::*;
use crate::utility::*;
use crate::logistic_regression::*;
use std::net::{TcpStream, TcpListener, Shutdown};
use std::io::{Read, Write};
use std::thread;
use std::sync::{Arc};
use std::time::{SystemTime};
use std::time;
use std::num::Wrapping;
use std::fs::File;

const DEBUG: bool = constants::DEBUG;
const BUF_SIZE: usize = constants::BUF_SIZE;


pub fn ti_client( settings_file : String ) {

	let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name( &settings_file.as_str() )).unwrap()
        .merge(config::Environment::with_prefix("APP")).unwrap();

    let xor_shares_path = match settings.get_str("xor_shares_path") {
    	Ok(string) => string,
    	Err(error) => {
    		panic!("Encountered a problem while parsing xor_shares_path: {:?}", error)
    	},
    };

    let additive_shares_path = match settings.get_str("additive_shares_path") {
    	Ok(string) => string,
    	Err(error) => {
    		panic!("Encountered a problem while parsing additive_shares_path: {:?}", error)
    	},
    };


    let ti_port = match settings.get_int("ti_port") {
    	Ok(num) => num as u16,
    	Err(error) => {
    		panic!("Encountered a problem while parsing ti_port: {:?} ", error)
    	},
    };


	let mut xor_file = File::create(xor_shares_path).expect("Unable to create file");
	
	let mut additive_file = File::create(additive_shares_path).expect("Unable to create file");

	    match TcpStream::connect(format!("localhost:{}", ti_port)) {
            Ok(mut stream) => {

				stream.set_read_timeout(None).expect("set_read_timeout call failed");
                println!("TI THREAD : successfully connected to ti on port {}",
               		ti_port);

                let mut buf = [ 0u8; BUF_SIZE ];
                let mut u_buf = [ 0u8 ; 8];
				let mut v_buf = [ 0u8 ; 8];
				let mut w_buf = [ 0u8 ; 8];
				let mut got_xor_shares = false;
				let mut got_additive_shares = false;
                let tx_len = BUF_SIZE - (BUF_SIZE % 24);
	            while match stream.read_exact(&mut buf) {
                	Ok(_) => {

                		if &buf[..7] == b"end xor" {
                			println!("TI THREAD : got xor shares, receiving additive shares");
                			got_xor_shares = true;
                		}

                		else if &buf[..7] == b"end add" {
                			got_additive_shares = true;
                		}

                		if got_xor_shares && got_additive_shares {
                			println!("TI THREAD : done receiving shares");
                			return ()
                		}

                		if got_xor_shares {
                			
	                		for i in (0..tx_len-24).step_by(24) {
	                			
	                			u_buf.clone_from_slice( &buf[i..i+8]);
	        					v_buf.clone_from_slice( &buf[i+8..i+16]);
	                			w_buf.clone_from_slice( &buf[i+16..i+24]);
	                			
	                			let (u, v, w) = (
									utility::byte_array_to_u64( u_buf ),
									utility::byte_array_to_u64( v_buf ),
									utility::byte_array_to_u64( w_buf )
	                			);
	                			if u>0 || v>0 || w>0 { 
	                				write!(additive_file, "{},{},{}\n", u, v, w);

	                			}
	                		}

	                	} else {
                        
                           for i in (0..tx_len-24).step_by(24) {
                                
                                u_buf.clone_from_slice( &buf[i..i+8]);
                                v_buf.clone_from_slice( &buf[i+8..i+16]);
                                w_buf.clone_from_slice( &buf[i+16..i+24]);
                                	
	                			let (u, v, w) = (
									Wrapping(utility::byte_array_to_u64( u_buf )),
									Wrapping(utility::byte_array_to_u64( v_buf )),
									Wrapping(utility::byte_array_to_u64( w_buf ))
	                			);
	                			if u.0>0 || v.0>0 || w.0>0 { 
	                				write!(xor_file, "{},{},{}\n", u, v, w);
	                			
	                			}
	                		}
	                	}

                		true
                	},
                	Err(e) => {
                		
                		false
                	}

	                
	            } {}
            },
            Err(e) => {
                println!("TI THREAD : failed to connect: {}", e);
            }
        }
	
}

}