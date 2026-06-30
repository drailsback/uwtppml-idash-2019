pub mod init {

extern crate csv;
extern crate config;
use crate::constants::*;
use std::sync::{Arc, Mutex, Condvar};
use std::num::Wrapping;
use std::fs::File;

const BUF_SIZE: usize = constants::BUF_SIZE;

pub struct Context {

    /* network */
	pub party_id: u8,
	pub ti_port: u16,
	pub internal_port: u16,
	pub external_port: u16,
	
    /* mpc */
    pub asymmetric_bit: u8,
    pub corr_rand: Vec<(Wrapping<u64>, Wrapping<u64>, Wrapping<u64>)>,
    pub corr_rand_xor: Vec<(u64, u64, u64)>,
    pub xor_shares_path: String,
    pub additive_shares_path: String,

    /* logistic regression */
	pub x_input_path: String,
	pub y_input_path: String,
	pub output_path: String,
    pub x_matrix: Vec<Vec<Wrapping<u64>>>,
    pub y_matrix: Vec<Vec<Wrapping<u64>>>,
	pub attribute_count: usize,
	pub instance_count: usize,
	pub iterations: u32,
    pub learning_rate_float: f64,
    pub learning_rate: Wrapping<u64>,
    pub decimal_precision: u32,
    pub integer_precision: u32,

    /* system */
    pub tx_buffer: Arc<Mutex<[u8 ; BUF_SIZE]>>,
	pub rx_buffer: Arc<Mutex<[u8 ; BUF_SIZE]>>,
	pub wake_tx: Arc<(Mutex<bool>, Condvar)>,
	pub wake_rx: Arc<(Mutex<bool>, Condvar)>,
	pub wake_op: Arc<(Mutex<bool>, Condvar)>,
	pub client_connected: Arc<Mutex<bool>>,
	pub server_connected: Arc<Mutex<bool>>,
    
}

pub struct TI_Context {

    pub ti_port: u16,
    pub additive_share_count: usize,
    pub xor_share_count: usize,
}

pub fn initialize_ti_context( settings_file : String ) -> TI_Context {

    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name( settings_file.as_str() )).unwrap()
        .merge(config::Environment::with_prefix("APP")).unwrap();

    let ti_port = match settings.get_int("ti_port") {
        Ok(num) => num as u16,
        Err(error) => {
            panic!("Encountered a problem while parsing ti_port: {:?} ", error)
        },
    };

    let additive_share_count = match settings.get_int("additive_share_count") {
        Ok(num) => num as usize,
        Err(error) => {
            panic!("Encountered a problem while parsing additive_share_count: {:?} ", error)
        },
    };


    let xor_share_count = match settings.get_int("xor_share_count") {
        Ok(num) => num as usize,
        Err(error) => {
            panic!("Encountered a problem while parsing xor_share_count: {:?} ", error)
        },
    };

    TI_Context {

        ti_port : ti_port,
        additive_share_count : additive_share_count,
        xor_share_count : xor_share_count,
    }

}

pub fn initialize_runtime_context( settings_file: String ) -> Context {

	let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name( &settings_file.as_str() )).unwrap()
        .merge(config::Environment::with_prefix("APP")).unwrap();

    // println!("{:?}",
    //          settings.try_into::<HashMap<String, String>>().unwrap());


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

    let party_id = match settings.get_int("party_id") {
    	Ok(num) => num as u8,
    	Err(error) => { 
    		panic!("Encountered a problem while parsing party_id: {:?}", error) 
    	},
    };

    let ti_port = match settings.get_int("ti_port") {
    	Ok(num) => num as u16,
    	Err(error) => {
    		panic!("Encountered a problem while parsing ti_port: {:?} ", error)
    	},
    };

    let party0_port = match settings.get_int("party0_port") {
    	Ok(num) => num as u16,
    	Err(error) => {
    		panic!("Encountered a problem while parsing party0_port: {:?} ", error)
    	},
    };
    
    let party1_port = match settings.get_int("party1_port") {
    	Ok(num) => num as u16,
    	Err(error) => {
    		panic!("Encountered a problem while parsing party1_port: {:?} ", error)
    	},
    };

    let x_input_path = match settings.get_str("x_input_path") {
    	Ok(string) => string,
    	Err(error) => {
    		panic!("Encountered a problem while parsing x_input_path: {:?}", error)
    	},
    };

   let y_input_path = match settings.get_str("y_input_path") {
    	Ok(string) => string,
    	Err(error) => {
    		panic!("Encountered a problem while parsing y_input_path: {:?}", error)
    	},
    };
 
    let output_path = match settings.get_str("output_path") {
    	Ok(string) => string,
    	Err(error) => {
    		panic!("Encountered a problem while parsring weights_output_path: {:?}", error)
    	},
    };


    let iterations = match settings.get_int("iterations") {
        Ok(num) => num as u32,
        Err(error) => { 
            panic!("Encountered a problem while parsing iterations: {:?}", error) 
        },
    };

    let learning_rate_float = match settings.get_float("learning_rate") {
        Ok(num) => num as f64,
        Err(error) => { 
            panic!("Encountered a problem while parsing learning_rate: {:?}", error) 
        },
    };

    let attribute_count = match settings.get_int("attribute_count") {
        Ok(num) => num as usize,
        Err(error) => { 
            panic!("Encountered a problem while parsing attribute_count: {:?}", error) 
        },
    };

    let instance_count = match settings.get_int("instance_count") {
        Ok(num) => num as usize,
        Err(error) => { 
            panic!("Encountered a problem while parsing instance: {:?}", error) 
        },
    };    

    let decimal_precision = match settings.get_int("decimal_precision") {
        Ok(num) => num as u32,
        Err(error) => { 
            panic!("Encountered a problem while parsing instance: {:?}", error) 
        },
    };    

    let integer_precision = match settings.get_int("integer_precision") {
        Ok(num) => num as u32,
        Err(error) => { 
            panic!("Encountered a problem while parsing instance: {:?}", error) 
        },
    };    

   let learning_rate = Wrapping( (learning_rate_float * ((2u64.pow(decimal_precision) as f64))) as u64 );


	Context {

		party_id 		    : party_id,
		ti_port			    : ti_port,
		internal_port       : if party_id == 0 {party0_port} else {party1_port},
		external_port       : if party_id == 0 {party1_port} else {party0_port},
		asymmetric_bit      : party_id,
        x_matrix            : load_u64_matrix(&x_input_path, instance_count as usize),
        y_matrix            : load_u64_matrix(&y_input_path, instance_count as usize),
        x_input_path        : x_input_path,
        y_input_path        : y_input_path,
        output_path         : output_path,
        xor_shares_path     : xor_shares_path,
        additive_shares_path: additive_shares_path,
		attribute_count     : attribute_count,
		instance_count      : instance_count,
        iterations          : iterations,
        learning_rate_float : learning_rate_float,
        learning_rate       : learning_rate,
        decimal_precision   : decimal_precision,
        integer_precision   : integer_precision,
		corr_rand           : Vec::new(), // TODO: from TI
        corr_rand_xor       : Vec::new(),
		rx_buffer           : Arc::new(Mutex::new( [0x00 ; BUF_SIZE] )),
		tx_buffer           : Arc::new(Mutex::new( [0x00 ; BUF_SIZE] )),
		wake_tx             : Arc::new((Mutex::new(false), Condvar::new())),
		wake_rx             : Arc::new((Mutex::new(false), Condvar::new())),
		wake_op             : Arc::new((Mutex::new(false), Condvar::new())),
		server_connected    : Arc::new(Mutex::new(false)),
		client_connected    : Arc::new(Mutex::new(false)),
	}

}

fn load_u64_matrix( file_path: &String, instances: usize ) -> Vec<Vec<Wrapping<u64>>> {

    let mut matrix: Vec<Vec<Wrapping<u64>>> = vec![ Vec::new() ; instances ];

    let file = File::open(file_path);
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file.unwrap());

    let mut index = 0;
    for result in rdr.deserialize() {

        if index == instances {
            break;
        }

        matrix[index] = result.unwrap();
        index += 1;
    }

    matrix
}

}