use distaff::{ ProgramInputs, assembly };
use super::{ Example, utils::parse_args };

pub fn get_example(args: &[String]) -> Example  {

    // get flag value and proof options from the arguments
    let (flag, options) = parse_args(args);

    // determine the expected result
    let expected_result: u128 = match flag {
        0 => 15,
        1 => 8,
        _ => panic!("flag must be a binary value")
    };
    
    // construct the program which either adds or multiplies two numbers
    // based on the value provided via secret inputs
    let program = assembly::compile("
        push.3
        push.5
        read
        if.true
            add
        else
            mul
        endif",
        options.hash_fn()).unwrap();

    println!("Generated a program to test conditional execution; expected result: {}", 
        expected_result);

    // put the flag as the only secret input for tape A
    let inputs = ProgramInputs::new(&[], &[flag as u128], &[]);

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