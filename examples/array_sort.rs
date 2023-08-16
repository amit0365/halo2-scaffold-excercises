use clap::Parser;
use halo2_base::gates::{GateChip, GateInstructions,RangeChip,RangeInstructions};
use halo2_base::utils::ScalarField;
use halo2_base::{AssignedValue, QuantumCell};
#[allow(unused_imports)]
use halo2_base::{
    Context,
    QuantumCell::{Constant, Existing, Witness},
};
use halo2_scaffold::scaffold::cmd::Cli;
use halo2_scaffold::scaffold::run;
use serde::{Deserialize, Serialize};
use std::env::var;
use poseidon::PoseidonChip;

const T: usize = 3;
const RATE: usize = 2;
const R_F: usize = 8;
const R_P: usize = 57;

// public inputs:
// * An array `arr` of length 1000
// * `start`, an index guaranteed to be in `[0, 1000)`
// * `end`, an index guaranteed to be in `[0, 1000)`
// * It is also known that `start <= end`

// public outputs:
// * An array `out` of length 1000 such that
//   * the first `end - start` entries of `out` are the subarray `arr[start:end]`
//   * all other entries of `out` are 0.

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    pub start: String,// field element, but easier to deserialize as a string
    pub end: String,
    pub array: Vec<u64>,
}

fn some_algorithm_in_zk<F: ScalarField>(
    ctx: &mut Context<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>,
) {
    assert!(input.array.len() == 25, "Array length must be 25");

    //let start: F = F::from_str_vartime(&input.start).expect("deserialize field element should not fail");
    let start = ctx.load_witness(F::from_str_vartime(&input.start).expect("deserialize field element should not fail"));
    let end = ctx.load_witness(F::from_str_vartime(&input.end).expect("deserialize field element should not fail"));
    let one = ctx.load_witness(F::from(1));

    const NUM_BITS: usize = 12;
    let lookup_bits: usize = var("LOOKUP_BITS").unwrap_or_else(|_| panic!("LOOKUP_BITS not set")).parse().unwrap();
    let chip: RangeChip<F> = RangeChip::default(lookup_bits);
    let gate = GateChip::<F>::default();

    // let start_index: usize = input.start.parse().expect("Failed to convert string to index");
    // let end_index: usize = input.end.parse().expect("Failed to convert string to index");
    
    let mut in_array_loaded = ctx.assign_witnesses(input.array.into_iter().map(F::from));

    let mut out_witness_array = [0 ; 30];
    for i in start.value()..end.value() {
        out_witness_array[i - start_index] = in_array_loaded[i].clone();
    }

    let mut out_array_loaded = ctx.assign_witnesses(out_witness_array.into_iter().map(F::from));

    // can use quantumcell directly 
    // let out_witness_array_fe: Vec<QuantumCell<F>>= out_witness_array.into_iter().map(|item| {
    //     let converted: F = F::from_str_vartime(&item).expect("deserialize field element should not fail");
    //     Witness(converted)
    // }).collect();

    // any alternative to clone
    //let mut in_array_filtered: Vec<QuantumCell<F>> = Vec::with_capacity(30);
    for idx in 0..25 {
        let greater_than_start = chip.is_less_than(ctx, Existing(start), Witness(F::from(idx+1)), NUM_BITS);
        let less_than_end = chip.is_less_than(ctx, Witness(F::from(idx+1)), Existing(end), NUM_BITS);
        let indicator = Existing(gate.mul(ctx, Existing(greater_than_start), Existing(less_than_end)));
        //let element = Existing(gate.select_from_idx(ctx, in_array_loaded.clone(), Witness(F::from(idx))));
        in_array_loaded[idx] = gate.mul(ctx, in_array_loaded[idx], indicator);
        //in_array_filtered.push(hadamard_product)
    }

    // (in[i][0]+in[i][1]+1)*(in[i][0]+in[i][1])+in[i][1];
    //let mut pair_out_array: Vec<QuantumCell<F>> = Vec::with_capacity(30);
    for idx in 0..25 {
        //let element = gate.select_from_idx(ctx, out_witness_array_fe.clone(), Witness(F::from(idx)));
        let first = gate.add(ctx,Witness(F::from(idx)), Existing(out_witness_array_fe[idx]));
        let first_add_1 = gate.add(ctx,Existing(one), Existing(first));
        let val = *first_add_1.value() * (*first.value()) + *element.value();
        let pairing = ctx.assign_region_last([Existing(element), Existing(first_add_1), Existing(first), Witness(val)], [0]);
        pair_out_array.push(Existing(pairing));
    };

    // (in[i][0]+in[i][1]+1)*(in[i][0]+in[i][1])+in[i][1];
    //let mut pair_in_array: Vec<QuantumCell<F>> = Vec::with_capacity(30);
    for idx in 0..25 {
        //let element = gate.select_from_idx(ctx, in_array_filtered.clone(), Witness(F::from(idx)));
        let first_sub_start = gate.sub(ctx,Witness(F::from(idx)), Existing(start));
        let first_add_element = gate.add(ctx,Existing(first_sub_start),Existing(in_array_filtered[idx]));
        let first_add_1 = gate.add(ctx, Existing(first_add_element), Existing(one));
        let val = (*first_add_1.value()) * (*first_add_element.value()) + *element.value();
        in_array_loaded[idx] = ctx.assign_region_last([Existing(element), Existing(first_add_1), Existing(first_add_element), Witness(val)], [0]);
        //pair_in_array.push(Existing(pairing));
    };

    let mut poseidon = PoseidonChip::<F, T, RATE>::new(ctx, R_F, R_P).unwrap();
    poseidon.update(&[start, end]);
    let hash = poseidon.squeeze(ctx, &gate).unwrap();
    make_public.push(hash);


    // let mut acc_in_poly: Vec<QuantumCell<F>> = Vec::with_capacity(30);
    // acc_in_poly.push(Constant(F::from(1)));

    // for idx in 0..25 {
    //     acc_in_poly.push(Existing(pairing));
    //     let prev_acc = gate.select_from_idx(ctx, acc_in_poly.clone(), Witness(F::from(idx)));
    //     let val = (*element.value() + F::from(idx)-start + F::from(1)) * (*element.value() + F::from(idx)-start) + *element.value();
    //     let new_acc = ctx.assign_region_last([Constant(F::from(0)), Witness(*prev_acc), Witness(*element.value()), Witness(val)], [0]);
    //     acc_in_poly.push(Existing(new_acc));
    // };

    //make_public.push(window);
    //println!("{:?}", window);
    //println!("out: {:?}", out_witness_array_fe[11]);
    for element in pair_out_array.iter().take(8) {
        println!("pairOut: {:?}", element);
    }
    for element in pair_in_array.iter().skip(start_index).take(8) {
        println!("pairIn: {:?}", element);
    }
    // println!("val_assigned: {:?}", out.value());
    // assert_eq!(*x.value() * x.value() + c, *out.value());
}

fn main() {
    env_logger::init();

    let args = Cli::parse();

    // run different zk commands based on the command line arguments
    run(some_algorithm_in_zk, args);
}
