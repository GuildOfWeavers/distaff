use distaff::{ ProgramInputs, assembly, math::field };
use super::{ Example, utils::parse_args };

pub fn get_example(args: &[String]) -> Example  {

    // get value and proof options from the arguments
    let (value, options) = parse_args(args);

    // determine the expected result
    let expected_result: u128 = if value < 10 {
        field::mul(10, value as u128)
    }
    else {
        field::add(10, value as u128)
    };
    
    // construct the program which checks if the value provided via secret inputs is
    // less than 10; if it is, the value is multiplied by 10, otherwise, 10 is added
    // to the value
    let program = assembly::compile("
        push.10
        read
        dup.2
        lt.128
        if.true
            mul
        else
            add
        endif",
        options.hash_fn()).unwrap();

    println!("Generated a program to test comparisons; expected result: {}", 
        expected_result);

    // put the flag as the only secret input for tape A
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