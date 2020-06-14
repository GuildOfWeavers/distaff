use distaff::{ Program, ProgramInputs, processor::opcodes::f128 as opcodes, FiniteField, F128 };
use super::{ Example, utils::parse_args };

pub fn get_example(args: &[String]) -> Example  {

    // get the number of values to range check and proof options
    let (n, options) = parse_args(args);
    
    // generate random sequence of 64-bit values
    let values = generate_values(n);

    // generate the program and expected results
    let program = generate_range_check_program(n);
    let expected_result = vec![count_63_bit_values(&values)];
    println!("Generated a program to range-check {} values; expected result: {}", 
        n,
        expected_result[0]);

    // transform the sequence of values into inputs for the program
    let inputs = generate_program_inputs(&values);

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
fn generate_values(n: usize) -> Vec<F128> {
    let mut values = F128::rand_vector(n);
    for i in 0..values.len() {
        values[i] = (values[i] as u64) as u128;
    }
    return values;
}

/// Generates a program to range-check a sequence of values.
fn generate_range_check_program(n: usize) -> Program {

    let mut program = vec![opcodes::BEGIN];

    // for each value in the list we do the following:
    // 1. read the value from secret input tape
    // 2. arrange the values on the stack in a way required by BINACC operation
    // 3. execute BINACC operation 63 times
    // 4. check if 63-bit representation of the value is equal to the value
    // 5. if it is, add it to the running sum of 63-bit values
    for _ in 0..n {
        program.push(opcodes::READ);
        program.push(opcodes::SWAP2);
        program.push(opcodes::DUP2);
        program.push(opcodes::SWAP4);
        program.push(opcodes::SWAP2);
        for _ in 0..63 {
            program.push(opcodes::BINACC);
        }
        program.push(opcodes::DROP);
        program.push(opcodes::EQ);
        program.push(opcodes::ADD);
    }

    return Program::from_path(program);
}

/// Generates inputs for the range-check program for the specified values.
fn generate_program_inputs(values: &[F128]) -> ProgramInputs<F128> {

    let p62: u128 = F128::exp(2, 62);

    // we need a single tape of secret inputs. For each value, we'll push
    // the value itself onto the tape, followed by lower 63-bits of the value
    // in the reverse order.
    let mut a = Vec::new();
    for &value in values.iter() {
        a.push(value);
        let mut bits = Vec::new();
        for i in 0..63 {
            bits.push((value >> i) & 1);
        }
        bits.reverse();
        a.extend_from_slice(&bits);
    }

    // we also need public inputs of this form. the first 0 is the 
    // placeholder for the number of values smaller than 64 bits
    return ProgramInputs::new(&[0, p62, 0, p62, 0], &a, &[]);
}

/// Counts the number of values smaller than 63-bits in size.
fn count_63_bit_values(values: &[F128]) -> u128 {
    let p63: u128 = F128::exp(2, 63);

    let mut result = 0;
    for &value in values.iter() {
        if value < p63 {
            result += 1;
        }
    }
    return result;
}