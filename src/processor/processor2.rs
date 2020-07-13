use crate::programs::{ ProgramInputs };
use crate::programs::program2::{ Program, ProgramBlock };
use crate::{ MIN_TRACE_LENGTH };

// RE-EXPORTS
// ================================================================================================

use super::decoder::decoder2::{ Decoder };
use super::stack::{ Stack };

// PUBLIC FUNCTIONS
// ================================================================================================

/// Returns register traces resulting from executing the specified program against the
/// specified inputs.
pub fn execute(program: &Program, inputs: &ProgramInputs) -> Vec<Vec<u128>>
{
    // initialize decoder and stack components
    // TODO: determine initial trace length dynamically
    let mut decoder = Decoder::new(MIN_TRACE_LENGTH);
    let mut stack = Stack::new(inputs, MIN_TRACE_LENGTH);

    // execute the program by traversing the execution graph
    //let procedure = program.get_proc(0);
    //traverse(&ProgramBlock::Group(procedure), &mut decoder, &mut stack, 0);

    // merge decoder and stack register traces into a single vector
    let mut register_traces = Vec::new(); // TODO: decoder.into_register_trace();
    register_traces.append(&mut stack.into_register_traces());

    return register_traces;
}

// HELPER FUNCTIONS
// ================================================================================================
fn traverse(block: &ProgramBlock, decoder: &mut Decoder, stack: &mut Stack) {
    match block {
        ProgramBlock::Span(block) => {
            for i in 0..block.length() {
                let (op_code, op_hint) = block.get_op(i);
                decoder.decode_op(op_code, op_hint.value());
                // TODO: update stack
            }
        },
        ProgramBlock::Group(block) => traverse_branch(block.body(), decoder, stack, 0, true),
        ProgramBlock::Switch(block) => {
            let condition = 0u128; // TODO: get from stack
            match condition {
                0 => traverse_branch(block.false_branch(), decoder, stack, block.true_branch_hash(), false),
                1 => traverse_branch(block.true_branch(), decoder, stack, block.false_branch_hash(), true),
                _ => panic!("cannot select a branch based on a non-binary condition {}", condition)
            };
        },
        ProgramBlock::Loop(block) => {
            let condition = 0u128; // TODO: get from stack
            match condition {
                0 => traverse_branch(block.skip(), decoder, stack, block.body_hash(), false),
                1 => traverse_loop(block.body(), decoder, stack, block.body_hash(), block.skip_hash()),
                _ => panic!("cannot enter loop based on a non-binary condition {}", condition)
            };
        },
    };
}

fn traverse_branch(body: &[ProgramBlock], decoder: &mut Decoder, stack: &mut Stack, sibling_hash: u128, is_true_branch: bool) {
    decoder.start_block();
    // TODO: update stack

    for block in body {
        traverse(block, decoder, stack);
        // TODO: merge block hash?
    }

    decoder.end_block(sibling_hash, is_true_branch);
    // TODO: update stack
}

fn traverse_loop(body: &[ProgramBlock], decoder: &mut Decoder, stack: &mut Stack, body_hash: u128, skip_hash: u128) {

    decoder.start_loop(body_hash);
    // TODO: update stack

    loop {
        for block in body {
            traverse(block, decoder, stack);
            // TODO: merge block hash?
        }

        let condition = 0u128; // TODO: get from stack
        match condition {
            0 => {
                decoder.break_loop();
                // TODO: update stack
                break;
            },
            1 => {
                decoder.wrap_loop();
                // TODO: update stack
            },
            _ => panic!("cannot exit loop based on a non-binary condition {}", condition)
        };
    }

    decoder.end_block(skip_hash, true);
    // TODO: update stack
}