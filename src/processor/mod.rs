use crate::programs::{ Program, ProgramInputs, ExecutionGraph, ExecutionHint, get_padded_length };
use crate::{ MIN_TRACE_LENGTH };

// RE-EXPORTS
// ================================================================================================
pub mod opcodes;

mod decoder;
use decoder::Decoder;

mod stack;
use stack::Stack;

// PUBLIC FUNCTIONS
// ================================================================================================

/// Returns register traces resulting from executing the specified program against the
/// specified inputs.
pub fn execute(program: &Program, inputs: &ProgramInputs<u128>) -> Vec<Vec<u128>>
{
    // initialize decoder and stack components
    // TODO: determine initial trace length dynamically
    let mut decoder = Decoder::new(MIN_TRACE_LENGTH);
    let mut stack = Stack::new(inputs, MIN_TRACE_LENGTH);

    // execute the program by traversing the execution graph
    traverse(program.execution_graph(), &mut decoder, &mut stack, 0);

    // merge decoder and stack register traces into a single vector
    let mut register_traces = decoder.into_register_trace();
    register_traces.append(&mut stack.into_register_traces());

    return register_traces;
}

// HELPER FUNCTIONS
// ================================================================================================

fn traverse(graph: &ExecutionGraph, decoder: &mut Decoder, stack: &mut Stack, mut step: usize) {

    let segment_ops = graph.operations();

    // execute all operations, except the last one, in the current segment of the graph
    let mut i = 0;
    while i < segment_ops.len() - 1 {
        // apply current operation to the decoder and the stack
        decoder.decode(segment_ops[i], false, step);
        stack.execute(segment_ops[i], segment_ops[i + 1], graph.get_hint(i), step);

        // if the current operation is a PUSH, update the decoder and the stack
        // and skip over to the next operation
        if segment_ops[i] == opcodes::f128::PUSH {
            step += 1;
            i += 1;
            decoder.decode(segment_ops[i], true, step);
            stack.execute(opcodes::f128::NOOP, 0, graph.get_hint(i), step);
        }

        step += 1;
        i += 1;
    }

    // if the graph doesn't end here, traverse the following branches
    if graph.has_next() {
        // first, execute the last operation in the current segment; we don't pass next_op
        // here because last operation of a segment cannot be a PUSH.
        decoder.decode(segment_ops[i], false, step);
        stack.execute(segment_ops[i], 0, graph.get_hint(i), step);
        step += 1;

        // then, based on the current value at the top of the stack, select a branch to follow
        let selector = stack.get_stack_top(step);
        match selector {
            1 => traverse(graph.true_branch(), decoder, stack, step),
            0 => traverse(graph.false_branch(), decoder, stack, step),
            _ => panic!("cannot branch on a non-binary value {} at step {}", selector, step)
        }
    }
    else {
        // if there are no more branches left, figure out how long the padded execution
        // path should be
        let path_length = get_padded_length(step, segment_ops[segment_ops.len() - 1]);
        let last_step = path_length - 1;

        if step < last_step {
            decoder.decode(segment_ops[i], false, step);
            stack.execute(segment_ops[i], 0, graph.get_hint(i), step);
            step += 1;
        }

        while step < last_step {
            decoder.decode(opcodes::f128::NOOP, false, step);
            stack.execute(opcodes::f128::NOOP, 0, ExecutionHint::None, step);
            step += 1;
        }
    }
}