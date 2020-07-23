use distaff::{ math::field, ProgramInputs, assembly };
use super::{ Example, utils::parse_args };

pub fn get_example(args: &[String]) -> Example  {

    // read starting value of the sequence and proof options from the arguments
    let (value, options) = parse_args(args);

    // determine the expected result
    let expected_result: u128 = compute_collatz_steps(value as u128);
    
    // construct the program which executes an unbounded loop to compute a Collatz sequence
    // which starts with the provided value; the output of the program is the number of steps
    // needed to reach the end of the sequence
    let program = assembly::compile("
    begin
        pad read dup push.1 ne
        while.true
            swap push.1 add swap dup isodd.128
            if.true
                push.3 mul push.1 add
            else
                push.2 div
            end
            dup push.1 ne
        end
        swap
    end",
    options.hash_fn()).unwrap();

    println!("Generated a program to compute Collatz sequence; expected result: {}", 
        expected_result);

    // put the starting value as the only secret input for tape A
    let inputs = ProgramInputs::new(&[], &[value as u128], &[]);

    // a single element from the top of the stack will be the output
    let num_outputs = 1;

    return Example {
        program,
        inputs,
        options,
        expected_result: vec![expected_result],
        num_outputs
    };
}

/// Computes number of steps in a Collatz sequence which starts with the provided `value`.
fn compute_collatz_steps(mut value: u128) -> u128 {

    let mut i = 0;
    while value != 1 {
        if value & 1 == 0 {
            value = field::div(value, 2);
        }
        else {
            value = field::add(field::mul(value, 3), 1)
        }
        i += 1;
    }

    return i;
}