use std::time::{ Instant };
use crate::stark::{ TraceTable, prove };

pub mod opcodes;

const DEFAULT_EXTENSION_FACTOR: usize = 32;

pub fn execute(program: &[u64], inputs: &[u64], num_outputs: usize) -> Vec<u64> {

    // execute the program to create an execution trace
    let now = Instant::now();
    let mut trace = TraceTable::new(&program, DEFAULT_EXTENSION_FACTOR);
    let t = now.elapsed().as_millis();
    println!("Generated execution trace of {} steps in {} ms", trace.len(), t);
    
    // opy the stack state the the last step to return as output
    let last_state = trace.get_state(trace.len() - 1);
    let outputs = last_state.get_stack()[0..num_outputs].to_vec();

    // extend the execution trace
    let now = Instant::now();
    trace.interpolate();
    trace.extend();
    let t = now.elapsed().as_millis();
    println!("Extended execution trace of {} registers to {} steps in {} ms", trace.register_count(), trace.len(), t);

    // generate STARK proof
    prove(&trace, DEFAULT_EXTENSION_FACTOR);

    return outputs;
}