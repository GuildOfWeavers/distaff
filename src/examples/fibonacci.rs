use distaff::{ Program, ProgramInputs, assembly, FiniteField, F128 };
use super::{ Example, utils::parse_args };

pub fn get_example(args: &[String]) -> Example  {

    // get the length of Fibonacci sequence and proof options from the arguments
    let (n, options) = parse_args(args);
    
    // generate the program and expected results
    let program = generate_fibonacci_program(n);
    let expected_result = vec![compute_fibonacci(n)];
    println!("Generated a program to compute {}-th Fibonacci term; expected result: {}", 
        n,
        expected_result[0]);

    // initialize stack with 2 values; 1 will be at the top
    let inputs = ProgramInputs::from_public(&[1, 0]);

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

/// Generates a program to compute the `n`-th term of Fibonacci sequence
fn generate_fibonacci_program(n: usize) -> Program {

    let mut program = String::with_capacity(n * 20);

    // the program is a simple repetition of 4 stack operations:
    // the first operation moves the 2nd stack item to the top,
    // the second operation duplicates the top 2 stack items,
    // the third operation removes the top item from the stack
    // the last operation pops top 2 stack items, adds them, and pushes
    // the result back onto the stack
    for _ in 0..(n - 1) {
        program.push_str("swap dup.2 drop add ");
    }

    return Program::from_path(assembly::translate(&program).unwrap());
}

/// Computes the `n`-th term of Fibonacci sequence
fn compute_fibonacci(n: usize) -> u128 {
    let mut n1 = 0;
    let mut n2 = 1;

    for _ in 0..(n - 1) {
        let n3 = F128::add(n1, n2);
        n1 = n2;
        n2 = n3;
    }

    return n2;
}