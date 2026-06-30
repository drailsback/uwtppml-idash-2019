pub mod logistic_regression {

//extern crate csv;

use crate::init::*;
use crate::protocol::*;
use crate::utility::*;
use std::num::Wrapping;
use std::fs::File;
use std::time::{SystemTime};
use std::io::Write;
use csv::{ReaderBuilder, Reader, ByteRecord};

pub fn lr_module( ctx: &mut init::Context ) {

	let mut add_file = File::open(ctx.additive_shares_path.clone()).unwrap();
    let mut add_rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(add_file);
	let mut xor_file = File::open(ctx.xor_shares_path.clone()).unwrap();
	let mut xor_rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(xor_file);
	let attribute_count = ctx.attribute_count;
	let instance_count  = ctx.instance_count;
	let iterations      = ctx.iterations;

	let x_matrix = ctx.x_matrix.clone();
	let y_matrix = ctx.y_matrix.clone();


	let mut weights = vec![ Wrapping(0u64) ; attribute_count ];

	for i in 0..iterations {

		ctx.corr_rand = load_wrapping_u64_tuples( &mut File::open(ctx.additive_shares_path.clone()).unwrap() );
		ctx.corr_rand_xor = load_u64_tuples( &mut  File::open(ctx.xor_shares_path.clone()).unwrap()  );

		println!("LR MODULE     : [{}] STARTING ITERATION", i);

		/* get dot products of weigts with each instance */
		let now = SystemTime::now();
		let mut z_list = vec![Wrapping(0u64) ; instance_count ];
		for k in 0..instance_count {

			z_list[k] = protocol::dot_product( &x_matrix[k], 
											   &weights, 
											   ctx, 
											   ctx.decimal_precision,
											   true,
											   false
						);
		}
		println!("LR MODULE     : [{}] dot products complete     -- work time = {} (ms)", 
		i, now.elapsed().unwrap().as_millis() );
		/* end dot product */

		/* get activation of each instance */
		let now = SystemTime::now();

		let o_list = batch_activate(&z_list, ctx);	
		println!("LR MODULE     : [{}] activactions complete     -- work time = {} (ms)", 
		i, now.elapsed().unwrap().as_millis() );
		/* end activation */

		/* test print of z, activation */
		// let z_revealed = protocol::reveal(&z_list, ctx, ctx.decimal_precision, true);
		// let o_list_revealed = protocol::reveal(&o_list, ctx, ctx.decimal_precision, true);
		// println!("LR MODULE     : [{}] activate[0]({}) = {}", i, z_revealed[0], o_list_revealed[0]);
		/* end test print */

		/* calculate sum of gradients for each instance */
		let now = SystemTime::now();
		let mut current_sum = vec![Wrapping(0u64) ; attribute_count];
		for k in 0..instance_count {
		
			let diff_list =  vec![y_matrix[k][0] - o_list[k] ; attribute_count];
			// TODO: add batch mult option to truncate
			let gradient = protocol::batch_multiply(&x_matrix[k], 
													&diff_list, 
													ctx
									 );

			for j in 0..attribute_count {
				current_sum[j] += gradient[j];
			}
		}

		for j in 0..attribute_count {
			current_sum[j] = utility::truncate_local(
									current_sum[j], 
									ctx.decimal_precision, 
									(*ctx).asymmetric_bit
								);
		}
		println!("LR MODULE     : [{}] sum of gradients complete -- work time = {} (ms)", 
			i, now.elapsed().unwrap().as_millis() );
		/* end sum of gradients */

		/* update weights */
		let now = SystemTime::now();
		let mut scaled_sum = vec![Wrapping(0u64) ; attribute_count];
		for k in 0..attribute_count {
			scaled_sum[k] = utility::truncate_local( 
									ctx.learning_rate * current_sum[k], 
									ctx.decimal_precision, 
									(*ctx).asymmetric_bit 
								);
			weights[k] += scaled_sum[k];
		}

		println!("LR MODULE     : [{}] weights updated           -- work time = {} (ms)", 
			i, now.elapsed().unwrap().as_millis() );
		/* end weight update */

	}

	// 	for k in 0..instance_count {
	// 	println!("weights[{}] = {}", k, weight_revealed[k]);
	// }
	let weight_revealed = protocol::reveal(&weights, ctx, ctx.decimal_precision, true);
	generate_weights_file(&weight_revealed, &(*ctx).output_path);
	println!("LR MODULE     : instance completed -- weights written to:\n{}.", (*ctx).output_path);


}

fn batch_activate( z_list : &Vec<Wrapping<u64>>, 
				   ctx    : &mut init::Context) -> Vec<Wrapping<u64>> {

	let frac_bitmask : u64 = (1 << (ctx.decimal_precision - 1)) - 1;
	let int_bitmask  :  u64 = (1 << (ctx.integer_precision + 1)) - 1;

	let one_half: Wrapping<u64> = Wrapping(1 << (ctx.decimal_precision - 1));

	let len            = ctx.instance_count;
	let asymmetric_bit = (*ctx).asymmetric_bit as u64;
	let inversion_mask: u64 = ( - Wrapping(asymmetric_bit) ).0;

	// println!("frac_bitmask  : 0x{:x}", frac_bitmask);
	// println!("int_bitmask   : 0x{:x}", int_bitmask);
	// println!("one half      : 0x{:x}", one_half);
	// println!("len           : {}", len);
	// println!("asymmetric_bit: {}", asymmetric_bit);
	// println!("inv mask      : 0x{:x}", inversion_mask);

	let z_decomp_list  = protocol::batch_bit_decomp(z_list, ctx); 

	let mut msb_list      = vec![ 0u64 ; len ];
	let mut not_msb_list  = vec![ 0u64 ; len ];
	let mut z_neg_list    = vec![ Wrapping(0) ; len ];

	for i in 0..len {

		msb_list[i]     = z_decomp_list[i] >> 63;
		not_msb_list[i] = (z_decomp_list[i] >> 63) ^ asymmetric_bit;
		z_neg_list[i]   = -z_list[i];
	}


	// println!("z[0]       : 0x{:x}", z_list[0]);
	// println!("z_decomp[0]: 0x{:x}", z_decomp_list[0]);
	// println!("z_neg[0]   : 0x{:x}", z_neg_list[0]);
	// println!("msb[0]     : {}", msb_list[0]);
	// println!("not_msb[0] : {}", not_msb_list[0]);

	// test 1 works up to this point

	let msb_list     = protocol::binary_vector_to_ring( &msb_list, ctx );
	let not_msb_list = protocol::binary_vector_to_ring( &not_msb_list, ctx );

	let z_unchanged = protocol::batch_multiply( &not_msb_list, &z_list, ctx );
	let z_flipped   = protocol::batch_multiply( &msb_list, &z_neg_list, ctx );
	
	let mut z_rectified = vec![ Wrapping(0u64) ; len ];

	for i in 0..len {

		z_rectified[i] = z_unchanged[i] + z_flipped[i];
	}


	let z_rectified_decomp = protocol::batch_bit_decomp(&z_rectified, ctx);

	// println!("z rectified[0]: 0x{:x}", z_rectified[0]);
	// println!("z rectified decomp[0]: 0x{:x}", z_rectified_decomp[0]);

	let mut z_frac_list = vec![ 0u64 ; len ];
	let mut z_int_list  = vec![ 0u64 ; len ];

	for i in 0..len {
		z_frac_list[i] = z_rectified_decomp[i] & frac_bitmask;
		z_int_list[i]  = 
			(z_rectified_decomp[i] >> (ctx.decimal_precision - 1))  & int_bitmask;
	}

	// println!("z frac[0]: 0x{:x}", z_frac_list[0]);
	// println!("z int[0]:  0x{:x}", z_int_list[0]);


	let mut not_or_list = vec![ 0u64 ; len ]; 
	let mut or_list     = vec![ 0u64 ; len ]; 
	
	for i in 0..len {
		not_or_list[i] = (z_int_list[i] ^ inversion_mask) & 1u64;
		z_int_list[i] = (z_int_list[i] ^ inversion_mask) >> 1;

	}

	for _i in 1..ctx.integer_precision {
		not_or_list = protocol::batch_bitwise_and(&not_or_list, &z_int_list, ctx, false);

		for j in 0..len {
			z_int_list[j] >>= 1;
		}
	}

	for i in 0..len {
		not_or_list[i] &= 1u64;
		or_list[i] = (not_or_list[i] ^ inversion_mask) & 1u64;
	}

	let or_list     = protocol::binary_vector_to_ring( &or_list, ctx );
	let not_or_list = protocol::binary_vector_to_ring( &not_or_list, ctx );
	
	// println!("is >= 1/2: 0x{:x}", or_list[0]);
	// println!("is <  1/2: 0x{:x}", not_or_list[0]);


	// convert frac to additive sharing TODO: DOES THIS ACTUALLY WORK?
	let z_frac_list = protocol::xor_share_to_additive( &z_frac_list, ctx, (ctx.decimal_precision-1) as usize );

	// println!("z frac ring: 0x{:x}", z_frac_list[0]);

	let mut const_region = vec![ Wrapping(0u64) ; len ];

	for i in 0..len {
		const_region[i] = one_half * or_list[i];
	}

	// println!("value from constant region: 0x{:x}", const_region[0]);

	let linear_region = protocol::batch_multiply(&z_frac_list, &not_or_list, ctx);

	// println!("value from linear region: 0x{:x}", linear_region[0]);

	let mut partial_result_pos = vec![ Wrapping(0) ; len ]; 
	let mut partial_result_neg = vec![ Wrapping(0) ; len ];
	for i in 0..len {
		partial_result_pos[i] = const_region[i] + linear_region[i];
		partial_result_neg[i] = - partial_result_pos[i]; 
	}

	// println!("positive partial resut: 0x{:x}", partial_result_pos[0]);
	// println!("negative partial resut: 0x{:x}", partial_result_neg[0]);

	let pos_selected = protocol::batch_multiply(&partial_result_pos, &not_msb_list, ctx);
	let neg_selected = protocol::batch_multiply(&partial_result_neg, &msb_list, ctx);

	// println!("positive selected: 0x{:x}", pos_selected[0]);
	// println!("negative selected: 0x{:x}", neg_selected[0]);


	let mut activations  = vec![ Wrapping(0u64) ; len ];

	for i in 0..len {
		activations[i] = pos_selected[i] + neg_selected[i] + (Wrapping(asymmetric_bit) * one_half); // + one_half;
	}
	//println!("activation: 0x{:x}",activations[0]);

	activations
}

fn generate_weights_file( weights: &Vec<f64>, file_path: &String ) {

	let mut file = File::create(file_path).expect("Unable to create file");
	for elem in weights.iter() {
		write!(file, "{}\n", elem);
	}
}

fn load_u64_tuples( file: &mut File ) -> Vec<(u64, u64, u64)> {

    let mut corr_rand: Vec<(u64, u64, u64)> = Vec::new();

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);
    let mut index = 0;
    for result in rdr.deserialize() {

        if index == 1000 {
            break;
        }

        let tuple: Vec<u64> = result.unwrap();
        corr_rand.push( (tuple[0] as u64, tuple[1] as u64, tuple[2] as u64) );
        index += 1;
    }

    corr_rand
}


fn load_wrapping_u64_tuples( file: &mut File ) -> Vec<(Wrapping<u64>, Wrapping<u64>, Wrapping<u64>)> {

    let mut corr_rand: Vec<(Wrapping<u64>, Wrapping<u64>, Wrapping<u64>)> = Vec::new();

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);
    let mut index = 0;
    for result in rdr.deserialize() {

        if index == 50000 {
            break;
        }

        let tuple: Vec<u64> = result.unwrap();
        corr_rand.push( (Wrapping(tuple[0] as u64), Wrapping(tuple[1] as u64), Wrapping(tuple[2] as u64)) );
        index += 1;
    }

    corr_rand

}

}

		// z_list[0] = - Wrapping(if (*ctx).asymmetric_bit == 1 {1u64 << ctx.decimal_precision} else {0u64});
		// z_list[1] = - Wrapping(if (*ctx).asymmetric_bit == 1 {1u64 << ctx.decimal_precision-1 } else {0u64});
		// z_list[2] = - Wrapping(if (*ctx).asymmetric_bit == 1 {1u64 << ctx.decimal_precision-2 } else {0u64});
		// z_list[3] = - Wrapping(if (*ctx).asymmetric_bit == 1 {0u64} else {0u64});
		// z_list[4] = Wrapping(if (*ctx).asymmetric_bit == 1 {1u64 << ctx.decimal_precision-2 } else {0u64});
		// z_list[5] = Wrapping(if (*ctx).asymmetric_bit == 1 {1u64 << ctx.decimal_precision-1 } else {0u64});
		// z_list[6] = Wrapping(if (*ctx).asymmetric_bit == 1 {1u64 << ctx.decimal_precision} else {0u64});


			// let x_list_revealed = protocol::reveal(&x_matrix[k], ctx, ctx.decimal_precision, true);
			// if i==1 && k==0 {
			// 	for j in 0..attribute_count {
			// 		println!("x[{}] = {}, weights[{}] = {}", j, x_list_revealed[j], j, weight_revealed[j]);
			// 	}
			// }


	// println!("LR MODULE     : logistic regression context initialized successfully --");
	// println!("\titerations           : {}", ctx.iterations);
	// println!("\tlearning rate        : {}", ctx.learning_rate_float);
	// println!("\tlearning rate (ring) : {}", ctx.learning_rate);
	// println!("\tattribute_count      : {}", ctx.attribute_count);
	// println!("\tinstance_count       : {}", ctx.instance_count);
	// println!("\tdecimal precision    : {}", ctx.decimal_precision);
	// println!("\tinteger precision    : {}", ctx.integer_precision);
	// println!("\tx_input_path         : {}", ctx.x_input_path);
	// println!("\ty_input_path         : {}", ctx.y_input_path);
	// println!("\toutput_path          : {}", ctx.output_path);
	// 