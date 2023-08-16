use clap::Parser;
use halo2_base::gates::{GateChip, GateInstructions, RangeChip, RangeInstructions};
use halo2_base::halo2_proofs::halo2curves::bn256::Fr;
use halo2_base::utils::ScalarField;
use halo2_base::AssignedValue;
#[allow(unused_imports)]
use halo2_base::{
    Context,
    QuantumCell::{Constant, Existing, Witness},
};
use halo2_scaffold::scaffold::cmd::Cli;
use halo2_scaffold::scaffold::run;
use serde::{Deserialize, Serialize};
use std::env::var;
use num_bigint::BigUint;
//use halo2_scaffold::scaffold::{mock};
//use rand::rngs::OsRng;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    pub x: String, // field element, but easier to deserialize as a string
}

// public inputs:
// * A non-negative integer x, which is guaranteed to be at most 16-bits

// public outputs:
// * The non-negative integer (x / 32), where "/" represents integer division.

fn some_algorithm_in_zk<F: ScalarField>(
    ctx: &mut Context<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>,
) {
    let x = F::from_str_vartime(&input.x).expect("deserialize field element should not fail");
    // `Context` can roughly be thought of as a single-threaded execution trace of a program we want to ZK prove.
    // We do some post-processing on `Context` to optimally divide the execution trace into multiple columns in a PLONKish arithmetization
    // More advanced usage with multi-threaded witness generation is possible, but we do not explain it here

    // first we load a number `x` into as system, as a "witness"
    let a = ctx.load_witness(x);
    // by default, all numbers in the system are private
    // we can make it public like so:
    make_public.push(a);

    let b: BigUint = BigUint::from(32u32);

    let lookup_bits: usize =
    var("LOOKUP_BITS").unwrap_or_else(|_| panic!("LOOKUP_BITS not set")).parse().unwrap();

    let chip: RangeChip<F> = RangeChip::default(lookup_bits);
    let (q,r) = chip.div_mod(ctx, a, b, 16);

    make_public.push(q);

    // ==== way 2 =======
    // here is a more optimal way to compute x^2 + 72 using the lower level `assign_region` API:
    // let val = *x.value() * x.value() + c;
    // let _val_assigned =
    //     ctx.assign_region_last([Constant(c), Existing(x), Existing(x), Witness(val)], [0]);
    // the `[0]` tells us to turn on a vertical `a + b * c = d` gate at row position 0.
    // this imposes the constraint c + x * x = val

    println!("a: {:?}", a.value());
    println!("div: {:?}", q.value());
    assert_eq!(F::from(32) * q.value() + r.value(), *a.value());

}

fn main() {
    env_logger::init();

    // run mock prover
    //mock(some_algorithm_in_zk, Fr::random(OsRng));

    // uncomment below to run actual prover:
    // prove(some_algorithm_in_zk, Fr::random(OsRng), Fr::zero());

    let args = Cli::parse();

    // run different zk commands based on the command line arguments
    run(some_algorithm_in_zk, args);
}
