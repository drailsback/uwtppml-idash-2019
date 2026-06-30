pub mod computing_party {

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

const DEBUG: bool = constants::DEBUG;
const BUF_SIZE: usize = constants::BUF_SIZE;

pub fn party_module( settings_file : String ) {

	println!("MAIN THREAD   : initializing runtime context and parsing inputs");
	let now = SystemTime::now();

	let mut ctx = init::initialize_runtime_context( settings_file );
	
	println!("MAIN THREAD   : input files parsed   -- work time = {} (ms)", 
		now.elapsed().unwrap().as_millis() );

	println!("MAIN THREAD   : runtime context initialized successfully --");
	println!("\tparty_id        : {}", ctx.party_id);
	println!("\tti_port         : 127.0.0.1{}", ctx.ti_port);
	println!("\tinternal_port   : 127.0.0.1:{}", ctx.internal_port);
	println!("\texternal_port   : 127.0.0.1:{}", ctx.external_port);

	let internal_port = ctx.internal_port;
	let external_port = ctx.external_port; 

	println!("Main thread: waiting forever");
	

	// let xor_corr_rand = Arc::clone(&ctx.xor_corr_rand);
	// let corr_rand = Arc::clone(&ctx.corr_rand);
	// thread::spawn(move|| {
	// 	ti_receiver::ti_client;
	// });

	// Create server thread -- give copy of rx_buffer
    let rx_buffer        = Arc::clone(&ctx.rx_buffer); 
    let wake_op          = ctx.wake_op.clone();   
    let wake_rx          = ctx.wake_rx.clone();    
    let server_connected = Arc::clone(&ctx.server_connected);

    thread::spawn(move|| {

    	let mut rx_count = 0u64;
        let listener = TcpListener::bind(format!("0.0.0.0:{}", internal_port)).unwrap();
        println!("SERVER THREAD : listening on port {}", internal_port);
          
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    
                   println!("SERVER THREAD : New connection: {}", stream.peer_addr().unwrap());
	               {
	               		let mut server_connected = server_connected.lock().unwrap();
	               		*server_connected = true;
	               		if DEBUG {println!("SERVER THREAD : connected = {}", *server_connected);}
	               }
                	loop {

						if DEBUG {println!("SERVER THREAD : waiting for rx request");}	               	
	            		{ // sleep until woken for rx request by main 
			               	let &(ref lock, ref cvar) = &*wake_rx;
			               	let mut receive = lock.lock().unwrap();
			                while !*receive {		              
			         			receive = cvar.wait(receive).unwrap();
			         		}
			         		*receive = false;
			         	}
	        			let mut rx_buffer = rx_buffer.lock().unwrap(); 
						if DEBUG {println!("SERVER THREAD : Attempting to receive");}
						match stream.read_exact(&mut *rx_buffer) {
							Ok(_size) => {
								// if size != BUF_SIZE {
					   //  			println!("SERVER THREAD : message received ({} bytes)", size);
					   //  		}
					   			rx_count += 1;
					   			let mut confirm_buf = utility::u64_to_byte_array(rx_count);
					   			match stream.write(&mut confirm_buf) {
					   				Ok(_) => {

					   				},
					   				Err(_) => {
					   					println!("SERVER THREAD: Failed to send confirmation");
					   				}
					   			}
					        	// if DEBUG {println!("SERVER THREAD : RECEIVED:\n{:02x}", (*rx_buffer).iter().format(""));}
					        	{ // wake main thread to read rx buf					        	
						            let &(ref lock, ref cvar) = &*wake_op;
									let mut read_ok = lock.lock().unwrap();
									*read_ok = true;
									cvar.notify_one();
				            	}
					            if DEBUG {println!("SERVER THREAD : notified main to read rx_buffer");}
					             
				       		},
				        	Err(_) => {
					            println!("SERVER THREAD : an error occurred, terminating connection with {}", 
					            	stream.peer_addr().unwrap());
					            stream.shutdown(Shutdown::Both).unwrap();
					            
					        }   
			        	}
			   		}

			    },
                Err(e) => {
                    println!("Error: {}", e)
                } 
        	}    
       }
    });

    // Create client thread -- give copy of tx_buffer
    let tx_buffer        = Arc::clone(&ctx.tx_buffer);
    let wake_tx          = ctx.wake_tx.clone();
    let client_connected = Arc::clone(&ctx.client_connected);
    
    thread::spawn(move|| {
    	let mut tx_count = 0u64;
		loop {
		    match TcpStream::connect(format!("localhost:{}", external_port))
		     {
		            Ok(mut stream) => {

						stream.set_read_timeout(None).expect("set_read_timeout call failed");
		                println!("CLIENT THREAD : successfully connected to server on port {}",
		               		external_port);
		                {
		               		let mut client_connected = client_connected.lock().unwrap();
		               		*client_connected = true;
		               		if DEBUG {println!("CLIENT THREAD : connected = {}", *client_connected);}
		                }
		            	loop {

		            		if DEBUG {println!("CLIENT THREAD : waiting for tx request");}			               	
		            		{ //  sleep until woken for tx request by main 
				               	let &(ref lock, ref cvar) = &*wake_tx;
				               	let mut transmit = lock.lock().unwrap();
				                while !*transmit {		              
				         			transmit = cvar.wait(transmit).unwrap();
				         		}
				         		*transmit = false;
				         	}
			         		if DEBUG {println!("CLIENT THREAD : tx request received");}
			         		{
			         			let mut confirm_buf = [ 0u8 ; 8 ];
			         			let tx_buffer = tx_buffer.lock().unwrap();
			         			let bytes = stream.write(&*tx_buffer).unwrap();
			         			// if bytes != BUF_SIZE {
			         			// 	println!("CLIENT THREAD : wrote {} bytes", bytes);
			         			// }
			         			tx_count += 1;
			         			stream.flush().expect("CLIENT THREAD : couldn't flush");
			         			match stream.read_exact(&mut confirm_buf) {
			         				Ok(_) => {
			         					assert_eq!(utility::byte_array_to_u64(confirm_buf), tx_count);
			         				},
			         				Err(_) => {
			         					println!("CLIENT THREAD: bad confirmation");
			         				}
			         			} 
			         			//if DEBUG {println!("CLIENT THREAD : SENT:\n{:02x}", (*tx_buffer).iter().format("")); }
			         		}
		                    if DEBUG {println!("CLIENT THREAD : passed message to server");}             
		                }
		            },
		            Err(e) => {
		                println!("CLIENT THREAD : Failed to connect: {}", e);
		            }
		    }
		} 
	});

	println!("MAIN THREAD   : checking connection status");		
	loop {
		{
			let server_connected = ctx.server_connected.lock().unwrap();
			let client_connected = ctx.client_connected.lock().unwrap();

			if *server_connected && *client_connected {
				println!("MAIN THREAD   : connection status -- client={}, server={}", *client_connected, *server_connected);
				break;
			}
		}	
		thread::sleep(time::Duration::from_millis(1));
	}

	// call main protocol

   	println!("MAIN THREAD   : Runtime count starting...");
   	let start = SystemTime::now();

   	logistic_regression::lr_module(&mut ctx);

	println!("MAIN THREAD   : complete -- runtime = {} (ms)", 
		start.elapsed().unwrap().as_millis() );
}


}