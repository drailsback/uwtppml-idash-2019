extern crate idash2019_rust;
use idash2019_rust::computing_party::*;
use idash2019_rust::trusted_initializer::*;
use idash2019_rust::ti_receiver::*;
use std::env;
//use std::process;

fn main() {
	

  let args: Vec<String> = env::args().collect();
  let settings_file = args[1].clone();

  println!("{}", settings_file);

	let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name( &settings_file.as_str() )).unwrap()
        .merge(config::Environment::with_prefix("APP")).unwrap();


    match settings.get_bool("ti") {
    	Ok(is_ti) => {
    		if is_ti {
    			trusted_initializer::ti_module( args[1].clone());
    		} else {
          ti_receiver::ti_client( settings_file );
    			computing_party::party_module( args[1].clone() ); 		
    		}
    	},
    	Err(error) => {
    		panic!("Encountered a problem while parsing Settings.toml: {:?}", error)
    	},
    };

    println!("MAIN MODULE: exiting");
}
