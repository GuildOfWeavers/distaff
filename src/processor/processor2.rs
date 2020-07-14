use crate::math::{ field };
use crate::programs::{ ProgramInputs };
use crate::programs::program2::{ Program, ProgramBlock };
use crate::{ MIN_TRACE_LENGTH };

// RE-EXPORTS
// ================================================================================================

use super::decoder::decoder2::{ Decoder };
use super::stack::stack2::{ Stack };
use super::opcodes2::{ UserOps as OpCode, OpHint };

// PUBLIC FUNCTIONS
// ================================================================================================

/// Returns register traces resulting from executing the specified program against the
/// specified inputs.
pub fn execute(program: &Program, inputs: &ProgramInputs) -> Vec<Vec<u128>>
{
    // initialize decoder and stack components
    let mut decoder = Decoder::new(MIN_TRACE_LENGTH);
    let mut stack = Stack::new(inputs, MIN_TRACE_LENGTH);

    // execute the program by traversing the execution graph
    let procedure = program.get_proc(0);
    traverse_branch(procedure.body(), &mut decoder, &mut stack, field::ZERO, true, true);

    decoder.finalize_trace();
    stack.finalize_trace();

    // merge decoder and stack register traces into a single vector
    let mut register_traces = decoder.into_register_traces();
    register_traces.append(&mut stack.into_register_traces());

    return register_traces;
}

// HELPER FUNCTIONS
// ================================================================================================
fn traverse(block: &ProgramBlock, decoder: &mut Decoder, stack: &mut Stack)
{
    match block {
        ProgramBlock::Span(block) => {
            for i in 0..block.length() {
                let (op_code, op_hint) = block.get_op(i);
                decoder.decode_op(op_code, op_hint.value());
                stack.execute(op_code, op_hint);
            }
        },
        ProgramBlock::Group(block) => traverse_branch(block.body(), decoder, stack, field::ZERO, true, false),
        ProgramBlock::Switch(block) => {
            let condition = stack.get_stack_top();
            match condition {
                0 => traverse_branch(block.false_branch(), decoder, stack, block.true_branch_hash(), false, false),
                1 => traverse_branch(block.true_branch(), decoder, stack, block.false_branch_hash(), true, false),
                _ => panic!("cannot select a branch based on a non-binary condition {}", condition)
            };
        },
        ProgramBlock::Loop(block) => {
            let condition = stack.get_stack_top();
            match condition {
                0 => traverse_branch(block.skip(), decoder, stack, block.body_hash(), false, false),
                1 => traverse_loop(block.body(), decoder, stack, block.body_hash(), block.skip_hash()),
                _ => panic!("cannot enter loop based on a non-binary condition {}", condition)
            };
        },
    };
}

fn traverse_branch(body: &[ProgramBlock], decoder: &mut Decoder, stack: &mut Stack, sibling_hash: u128, is_true_branch: bool, skip_begin: bool) {
    if !skip_begin {
        decoder.start_block();
        stack.execute(OpCode::Noop, OpHint::None);
    }
    
    // traverse the first block, which must be a Span block
    traverse(&body[0], decoder, stack);

    for block in body.iter().skip(1) {
        if block.is_span() {
            decoder.decode_op(OpCode::Noop, field::ZERO);
            stack.execute(OpCode::Noop, OpHint::None);
        }
        traverse(block, decoder, stack);
    }

    decoder.decode_op(OpCode::Noop, field::ZERO);
    stack.execute(OpCode::Noop, OpHint::None);

    decoder.end_block(sibling_hash, is_true_branch);
    stack.execute(OpCode::Noop, OpHint::None);

    for _ in 0..14 {
        decoder.decode_op(OpCode::Noop, field::ZERO);
        stack.execute(OpCode::Noop, OpHint::None);
    }
}

fn traverse_loop(body: &[ProgramBlock], decoder: &mut Decoder, stack: &mut Stack, body_hash: u128, skip_hash: u128) {

    decoder.start_loop(body_hash);
    stack.execute(OpCode::Noop, OpHint::None);

    loop {
        for block in body {
            if block.is_span() {
                decoder.decode_op(OpCode::Noop, field::ZERO);
                stack.execute(OpCode::Noop, OpHint::None);
            }
            traverse(block, decoder, stack);
        }

        let condition = stack.get_stack_top();
        match condition {
            0 => {
                decoder.break_loop();
                stack.execute(OpCode::Noop, OpHint::None);
                break;
            },
            1 => {
                decoder.wrap_loop();
                stack.execute(OpCode::Noop, OpHint::None);
            },
            _ => panic!("cannot exit loop based on a non-binary condition {}", condition)
        };
    }

    decoder.end_block(skip_hash, true);
    stack.execute(OpCode::Noop, OpHint::None);

    for _ in 0..14 {
        decoder.decode_op(OpCode::Noop, field::ZERO);
        stack.execute(OpCode::Noop, OpHint::None);
    }
}

#[cfg(test)]
mod tests {

    use crate::crypto::{ hash::blake3 };
    use crate::programs::program2::assembly;
    use super::{ ProgramInputs };

    #[test]
    fn execute() {
        let program = assembly::compile("begin add push.5 mul push.7 end", blake3).unwrap();
        let inputs = ProgramInputs::from_public(&[1, 2]);

        let trace = super::execute(&program, &inputs);
        print_trace(&trace);

        assert_eq!(1, 2);
    }

    fn print_trace(trace: &Vec<Vec<u128>>) {

        let width = trace.len();
        let length = trace[0].len();

        let ctx_stack_depth = 1;
        let ctx_stack_end = 14 + ctx_stack_depth;
        let loop_stack_depth = 0;
        let loop_stack_end = ctx_stack_end + loop_stack_depth;

        for i in 0..length {
            let mut state = vec![];
            for j in 0..4 {
                state.push(trace[j][i] >> 64);
            }
            for j in 4..width {
                state.push(trace[j][i]);
            }
            
            println!("{}:\t{:>16X?} {:?} {:?} {:?} {:X?} {:X?} {:?}", i,
                &state[0..4], &state[4..7],
                &state[7..12], &state[12..14],
                &state[14..ctx_stack_end], &state[ctx_stack_end..loop_stack_end],
                &state[loop_stack_end..]
            );
        }
    }
}