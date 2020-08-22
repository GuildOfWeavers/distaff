use distaff::{ ProgramInputs, assembly, math::field };
use super::{ Example, utils::parse_args };

pub fn get_example(args: &[String]) -> Example  {

    // get value and proof options from the arguments
    let (value, options) = parse_args(args);

    // determine the expected result
    let expected_result: u128 = if value < 9 {
        field::mul(9, value as u128)
    }
    else {
        field::add(9, value as u128)
    };
    
    // construct the program which checks if the value provided via secret inputs is
    // less than 9; if it is, the value is multiplied by 9, otherwise, 9 is added
    // to the value; then we check if the value is odd.
    let program = assembly::compile("
    begin
        push.9
        read
        dup.2
        lt.128
        if.true
            mul
        else
            add
        end
        dup
        isodd.128
    end").unwrap();

    println!("Generated a program to test comparisons; expected result: {}", 
        expected_result);

    // put the flag as the only secret input for tape A
    let inputs = ProgramInputs::new(&[], &[value as u128], &[]);

    // a single element from the top of the stack will be the output
    let num_outputs = 2;

    return Example {
        program,
        inputs,
        options,
        expected_result: vec![expected_result & 1, expected_result],
        num_outputs
    };
}