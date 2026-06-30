
pub mod trusted_initializer {

extern crate rand;
use rand::Rng;
use crate::constants::*;
use crate::init::*;
use crate::utility::*;
use std::thread;
use std::net::{TcpStream, TcpListener, Shutdown};
use std::io::{Read, Write};
use std::num::Wrapping;
use std::time::{SystemTime};
use std::time;

const BUF_SIZE: usize = constants::BUF_SIZE;

pub fn ti_module( settings_file: String ) {

	println!("TI MAIN : generating correlated randomness...");
	
	let ctx = init::initialize_ti_context( settings_file.clone() );

	let mut xor_shares = generate_xor_triples(ctx.xor_share_count);
	let mut additive_shares = generate_additive_triples(ctx.additive_share_count);

	let mut party0_xor_shares: Vec<(u64, u64, u64)> = Vec::new();
	let mut party1_xor_shares: Vec<(u64, u64, u64)> = Vec::new();

	let mut party0_additive_shares: Vec<(u64, u64, u64)> = Vec::new();
	let mut party1_additive_shares: Vec<(u64, u64, u64)> = Vec::new();

	let mut rng = rand::thread_rng();

	for i in 0..ctx.xor_share_count {

		let (u, v, w) = xor_shares.pop().unwrap();

		let u0: u64 = rng.gen();
		let v0: u64 = rng.gen();
		let w0: u64 = rng.gen();

		let u1 = u ^ u0;
		let v1 = v ^ v0;
		let w1 = w ^ w0;

		party0_xor_shares.push( (u0, v0, w0) );
		party1_xor_shares.push( (u1, v1, w1) );
	}

	for i in 0..ctx.additive_share_count {

		let (u, v, w) = additive_shares.pop().unwrap();

		let u0: u64 = rng.gen();
		let v0: u64 = rng.gen();
		let w0: u64 = rng.gen();

		let u1 = (Wrapping(u) - Wrapping(u0)).0;
		let v1 = (Wrapping(v) - Wrapping(v0)).0;
		let w1 = (Wrapping(w) - Wrapping(w0)).0;

		party0_additive_shares.push( (u0, v0, w0) );
		party1_additive_shares.push( (u1, v1, w1) );
	}

	println!("TI MAIN : shares generated");

    let server = thread::spawn( move || {

		let ti_port = ctx.ti_port;
		let additive_share_count = ctx.additive_share_count;
		let xor_share_count = ctx.xor_share_count;

        println!("SERVER THREAD : thread beginning");

        let listener = TcpListener::bind(format!("127.0.0.1:{}", ti_port)).unwrap();
        let mut p0_done = false;
        let mut p1_done = false;
        // accept connections and process them, spawning a new thread for each one
        let mut client_id = 0;
        println!("SERVER THREAD : listening on port {}", ti_port);
        for stream in listener.incoming() {



        	let mut party0_xor_shares = party0_xor_shares.clone();
			let mut party1_xor_shares = party1_xor_shares.clone();
			let mut party0_additive_shares = party0_additive_shares.clone();
			let mut party1_additive_shares = party1_additive_shares.clone();

            match stream {
                Ok(stream) => {
                    println!("SERVER THREAD : new connection: {}, client id: {}", stream.peer_addr().unwrap(), client_id);
                    if client_id == 0 {
                    	 let p0_thread = thread::spawn(move|| {
                        	handle_client(stream, &mut party0_xor_shares.clone(), &mut party0_additive_shares.clone()) 
                   		});

                    	p0_thread.join();
                    	p0_done = true;
                    	println!("SERVER THREAD : P0 thread joined ");
                    } else if client_id == 1 {
                    	let p1_thread = thread::spawn(move|| {
                        	handle_client(stream, &mut party1_xor_shares.clone(), &mut party1_additive_shares.clone()) 
                   		});

                   	   p1_thread.join();
                   	   p1_done = true;

                       println!("SERVER THREAD : P1 thread joined ");
                   	}              
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
            client_id += 1;
            if p0_done && p1_done {
            	break
            }

        }
        drop(listener);

    });

	server.join();
	println!("TI MAIN : tearing down socket ");
    
	return ()
	   
}

fn handle_client(mut stream: TcpStream, xor_shares: &mut Vec<(u64, u64, u64)>, additive_shares:&mut  Vec<(u64, u64, u64)>) {

	println!("SERVER THREAD: (client handler) sending xor shares... ");
	while xor_shares.len() > 0 {

		let mut buf = [0u8 ; BUF_SIZE];
		let mut counter = 0;
		while xor_shares.len() != 0 && counter < BUF_SIZE-24 {
			let (u, v, w) = xor_shares.pop().unwrap();
			buf[counter..counter+8].clone_from_slice(&utility::u64_to_byte_array(u));
			buf[counter+8..counter+16].clone_from_slice(&utility::u64_to_byte_array(v));
			buf[counter+16..counter+24].clone_from_slice(&utility::u64_to_byte_array(w));
			counter += 24;
			//println!("Sent: {} {} {}", u, v, w);
		}	
		stream.write(&buf);
	}
	stream.write(b"end xor");
	println!("SERVER THREAD: (client handler) xor shares sent, sending additive shares... ");
	while additive_shares.len() > 0 {

		let mut buf = [0u8 ; BUF_SIZE];
		let mut counter = 0;

		while additive_shares.len() != 0 && counter < BUF_SIZE-24 {
			let (u, v, w) = additive_shares.pop().unwrap();
			buf[counter..counter+8].clone_from_slice(&utility::u64_to_byte_array(u));
			buf[counter+8..counter+16].clone_from_slice(&utility::u64_to_byte_array(v));
			buf[counter+16..counter+24].clone_from_slice(&utility::u64_to_byte_array(w));
			counter += 24;
			//println!("Sent: {} {} {}", u, v, w);
		}	
		stream.write(&buf);
	}
	println!("SERVER THREAD: (client handler) additive shares sent");
	stream.write(b"end add");

	()
}


fn generate_xor_triples( share_count: usize ) -> Vec<(u64, u64, u64)> {

	let mut rng = rand::thread_rng();

	let mut xor_triples = Vec::new();

	for i in 0..share_count {

		let u: u64 = rng.gen();
		let v: u64 = rng.gen();
		let w = u & v;

		xor_triples.push( (u, v, w) );
	}

	xor_triples

}

fn generate_additive_triples( share_count: usize ) -> Vec<(u64, u64, u64)> {

	let mut rng = rand::thread_rng();

	let mut additive_triples = Vec::new();

	for i in 0..share_count {

		let u: u64 = rng.gen();
		let v: u64 = rng.gen();
		let w = (Wrapping(u) * Wrapping(v)).0;

		additive_triples.push( (u, v, w) );
	}

	additive_triples

}



}