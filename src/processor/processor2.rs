use crate::math::{ field };
use crate::programs::{ ProgramInputs };
use crate::programs::program2::{ Program, ProgramBlock, Span, Loop };
use crate::{ MIN_TRACE_LENGTH };

pub const HACC_NUM_ROUNDS: usize = 14; // TODO: move to global constants

// RE-EXPORTS
// ================================================================================================

use super::decoder::decoder2::{ Decoder };
use super::stack::stack2::{ Stack };
use super::opcodes2::{ UserOps as OpCode, OpHint };

// PUBLIC FUNCTIONS
// ================================================================================================

/// Returns register traces resulting from executing the specified procedure within the program
/// against the specified inputs.
pub fn execute(program: &Program, proc_index: usize, inputs: &ProgramInputs) -> Vec<Vec<u128>>
{
    // initialize decoder and stack components
    let mut decoder = Decoder::new(MIN_TRACE_LENGTH);
    let mut stack = Stack::new(inputs, MIN_TRACE_LENGTH);

    // get the procedure from the program and execute it
    let procedure = program.get_proc(proc_index);
    execute_blocks(procedure.body(), &mut decoder, &mut stack);
    close_block(&mut decoder, &mut stack, field::ZERO, true);

    // fill in remaining steps to make sure the length of the trace is a power of 2
    decoder.finalize_trace();
    stack.finalize_trace();

    // merge decoder and stack register traces into a single vector
    let mut register_traces = decoder.into_register_traces();
    register_traces.append(&mut stack.into_register_traces());

    return register_traces;
}

// HELPER FUNCTIONS
// ================================================================================================
fn execute_blocks(blocks: &[ProgramBlock], decoder: &mut Decoder, stack: &mut Stack)
{
    // execute first block in the sequence, which mast be a Span block
    match &blocks[0] {
        ProgramBlock::Span(block) => execute_span(block, decoder, stack, true),
        _ => panic!("first block in a sequence must be a Span block"),
    }

    // execute all other blocks in the sequence one after another
    for block in blocks.iter().skip(1) {
        match block {
            ProgramBlock::Span(block) => execute_span(block, decoder, stack, false),
            ProgramBlock::Group(block) => {
                start_block(decoder, stack);
                execute_blocks(block.body(), decoder, stack);
                close_block(decoder, stack, field::ZERO, true);
            },
            ProgramBlock::Switch(block) => {
                start_block(decoder, stack);
                let condition = stack.get_stack_top();
                match condition {
                    0 => {
                        execute_blocks(block.false_branch(), decoder, stack);
                        close_block(decoder, stack, block.true_branch_hash(), false);
                    },
                    1 => {
                        execute_blocks(block.true_branch(), decoder, stack);
                        close_block(decoder, stack, block.false_branch_hash(), true);
                    },
                    _ => panic!("cannot select a branch based on a non-binary condition {}", condition)
                };
            },
            ProgramBlock::Loop(block) => {
                let condition = stack.get_stack_top();
                match condition {
                    0 => {
                        start_block(decoder, stack);
                        execute_blocks(block.skip(), decoder, stack);
                        close_block(decoder, stack, block.body_hash(), false);
                    },
                    1 => execute_loop(block, decoder, stack),
                    _ => panic!("cannot enter loop based on a non-binary condition {}", condition)
                }
            },
        }
    }
}

/// Executes all instructions in a Span block.
fn execute_span(block: &Span, decoder: &mut Decoder, stack: &mut Stack, is_first: bool) {
    // if this is the first Span block in a sequence of blocks, it needs to be
    // pre-padded with a NOOP to make sure the first instruction in the block
    // starts executing on a step which is a multiple of 16
    if !is_first {
        decoder.decode_op(OpCode::Noop, field::ZERO);
        stack.execute(OpCode::Noop, OpHint::None);
    }

    // execute all other instructions in the block
    for i in 0..block.length() {
        let (op_code, op_hint) = block.get_op(i);
        decoder.decode_op(op_code, op_hint.value());
        stack.execute(op_code, op_hint);
    }
}

/// Starts executing a new program block.
fn start_block(decoder: &mut Decoder, stack: &mut Stack)
{
    decoder.start_block();
    stack.execute(OpCode::Noop, OpHint::None);
}

/// Closes the currently executing program block.
fn close_block(decoder: &mut Decoder, stack: &mut Stack, sibling_hash: u128, is_true_branch: bool)
{
    // a sequence of non-loop blocks always ends on a step which is one less than a
    // multiple of 16; so, we need to pad it with a NOOP to make sure block end
    // aligns with a multiple of 16
    decoder.decode_op(OpCode::Noop, field::ZERO);
    stack.execute(OpCode::Noop, OpHint::None);

    // end the block, this prepares decoder registers for merging block hash into
    // program hash
    decoder.end_block(sibling_hash, is_true_branch);
    stack.execute(OpCode::Noop, OpHint::None);

    // execute NOOPs to merge block hash into the program hash
    for _ in 0..HACC_NUM_ROUNDS {
        decoder.decode_op(OpCode::Noop, field::ZERO);
        stack.execute(OpCode::Noop, OpHint::None);
    }
}

fn execute_loop(block: &Loop, decoder: &mut Decoder, stack: &mut Stack)
{
    // mark the beginning of the loop block
    decoder.start_loop(block.body_hash());
    stack.execute(OpCode::Noop, OpHint::None);

    // execute blocks in loop body until top of the stack becomes 0
    loop {
        execute_blocks(block.body(), decoder, stack);

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

    // end the block, this prepares decoder registers for merging block hash into
    // program hash
    decoder.end_block(block.skip_hash(), true);
    stack.execute(OpCode::Noop, OpHint::None);

    // execute NOOPs to merge block hash into the program hash
    for _ in 0..HACC_NUM_ROUNDS {
        decoder.decode_op(OpCode::Noop, field::ZERO);
        stack.execute(OpCode::Noop, OpHint::None);
    }
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {

    use crate::crypto::{ hash::blake3 };
    use crate::programs::program2::assembly;
    use super::{ ProgramInputs };

    #[test]
    fn execute() {
        let program = assembly::compile("begin add push.5 mul push.7 end", blake3).unwrap();
        let inputs = ProgramInputs::from_public(&[1, 2]);

        let trace = super::execute(&program, 0, &inputs);
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