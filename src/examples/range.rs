use distaff::{ Program, ProgramInputs, assembly, crypto::HashFunction, math::field };
use super::{ Example, utils::parse_args };

pub fn get_example(args: &[String]) -> Example  {

    // get the number of values to range check and proof options
    let (n, options) = parse_args(args);
    
    // generate random sequence of 64-bit values
    let values = generate_values(n);

    // generate the program and expected results
    let program = generate_range_check_program(n, options.hash_fn());
    let expected_result = vec![count_63_bit_values(&values)];
    println!("Generated a program to range-check {} values; expected result: {}", 
        n,
        expected_result[0]);

    // set public inputs to the initial sum (0), and pass values to the secret tape A
    let inputs = ProgramInputs::new(&[0], &values, &[]);

    // a single element from the top of the stack will be the output
    let num_outputs = 1;

    return Example {
        program,
        inputs,
        options,
        expected_result,
        num_outputs
    };
}

/// Generates a random sequence of 64-bit values.
fn generate_values(n: usize) -> Vec<u128> {
    let mut values = field::rand_vector(n);
    for i in 0..values.len() {
        values[i] = (values[i] as u64) as u128;
    }
    return values;
}

/// Generates a program to range-check a sequence of values.
fn generate_range_check_program(n: usize, hash_fn: HashFunction) -> Program {

    let mut program = String::with_capacity(n * 80);
    program.push_str("begin ");

    // repeat the cycle of the following operations:
    // 1. read a value from secret tape A
    // 2. check if it fits into 63 bits (result is 1 if true, 0 otherwise)
    // 3. add the result into the running sum
    for _ in 0..n {
        program.push_str("read rc.63 add ");
    }
    program.push_str("end");

    return assembly::compile(&program, hash_fn).unwrap();
}

/// Counts the number of values smaller than 63-bits in size.
fn count_63_bit_values(values: &[u128]) -> u128 {
    let p63: u128 = field::exp(2, 63);

    let mut result = 0;
    for &value in values.iter() {
        if value < p63 {
            result += 1;
        }
    }
    return result;
}