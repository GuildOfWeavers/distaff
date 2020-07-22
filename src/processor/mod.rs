use crate::{
    math::field,
    programs::{ Program, ProgramInputs, blocks::{ ProgramBlock, Span, Loop } },
    MIN_TRACE_LENGTH, HACC_NUM_ROUNDS,
};

// RE-EXPORTS
// ================================================================================================

mod decoder;
pub use decoder::{ Decoder };

mod stack;
pub use stack::{ Stack };

pub mod opcodes;
pub use opcodes::{ UserOps as OpCode, OpHint };

// PUBLIC FUNCTIONS
// ================================================================================================

/// Returns register traces resulting from executing the specified procedure within the program
/// against the specified inputs.
pub fn execute(program: &Program, proc_index: usize, inputs: &ProgramInputs) -> (Vec<Vec<u128>>, usize, usize)
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

    // save context and loop depths into variables before decoder is consumed
    let context_depth = decoder.max_ctx_stack_depth();
    let loop_depth = decoder.max_loop_stack_depth();

    // merge decoder and stack register traces into a single vector
    let mut register_traces = decoder.into_register_traces();
    register_traces.append(&mut stack.into_register_traces());

    return (register_traces, context_depth, loop_depth);
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
fn execute_span(block: &Span, decoder: &mut Decoder, stack: &mut Stack, is_first: bool)
{
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
    // a sequence of blocks always ends on a step which is one less than a multiple of 16;
    // all sequences end one operation short of multiple of 16 - so, we need to pad them
    // with a single NOOP ensure proper alignment
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

/// Executes the specified loop.
fn execute_loop(block: &Loop, decoder: &mut Decoder, stack: &mut Stack)
{
    // mark the beginning of the loop block
    decoder.start_loop(block.image());
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

    // execute the contents of the skip block to make sure the loop was exited correctly
    match &block.skip()[0] {
        ProgramBlock::Span(block) => execute_span(block, decoder, stack, true),
        _ => panic!("invalid skip block content: content must be a Span block"),
    }

    // close block
    close_block(decoder, stack, block.skip_hash(), true);
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {

    use crate::{
        crypto::hash::blake3, programs::assembly, stark::TraceState, utils::as_bytes,
    };
    use super::{ ProgramInputs };

    #[test]
    fn execute_span() {
        let program = assembly::compile("begin add push.5 mul push.7 end", blake3).unwrap();
        let inputs = ProgramInputs::from_public(&[1, 2]);

        let (trace, ctx_depth, loop_depth) = super::execute(&program, 0, &inputs);
        let trace_length = trace[0].len();

        assert_eq!(64, trace_length);
        assert_eq!(17, trace.len());
        let mut state = build_trace_state(trace.len(), ctx_depth, loop_depth) ;
        state.update_from_trace(&trace, trace_length - 1);

        assert_eq!(46, state.op_counter());
        assert_eq!(program.hash(), as_bytes(state.program_hash()));
        assert_eq!([1, 1, 1], state.cf_op_bits());
        assert_eq!([1, 1, 1, 1, 1], state.ld_op_bits());
        assert_eq!([1, 1], state.hd_op_bits());
        assert_eq!([0], state.ctx_stack());
        assert_eq!([7, 15, 0, 0, 0, 0, 0, 0], state.user_stack());
    }

    #[test]
    fn execute_block() {
        let program = assembly::compile("begin add block push.5 mul push.7 end end", blake3).unwrap();
        let inputs = ProgramInputs::from_public(&[1, 2]);

        let (trace, ctx_depth, loop_depth) = super::execute(&program, 0, &inputs);
        let trace_length = trace[0].len();

        assert_eq!(64, trace_length);
        assert_eq!(18, trace.len());

        let mut state = build_trace_state(trace.len(), ctx_depth, loop_depth) ;
        state.update_from_trace(&trace, trace_length - 1);
        
        assert_eq!(60, state.op_counter());
        assert_eq!(program.hash(), as_bytes(state.program_hash()));
        assert_eq!([1, 1, 1], state.cf_op_bits());
        assert_eq!([1, 1, 1, 1, 1], state.ld_op_bits());
        assert_eq!([1, 1], state.hd_op_bits());
        assert_eq!([0, 0], state.ctx_stack());
        assert_eq!([7, 15, 0, 0, 0, 0, 0, 0], state.user_stack());
    }

    #[test]
    fn execute_if_else() {
        let program = assembly::compile(
            "begin read if.true add push.3 else push.7 add push.8 end mul end",
            blake3).unwrap();
        
        // execute true branch
        let inputs = ProgramInputs::new(&[5, 3], &[1], &[]);
        let (trace, ctx_depth, loop_depth) = super::execute(&program, 0, &inputs);
        let trace_length = trace[0].len();

        assert_eq!(128, trace_length);
        assert_eq!(19, trace.len());

        let mut state = build_trace_state(trace.len(), ctx_depth, loop_depth) ;
        state.update_from_trace(&trace, trace_length - 1);

        assert_eq!(76, state.op_counter());
        assert_eq!(program.hash(), as_bytes(state.program_hash()));
        assert_eq!([1, 1, 1], state.cf_op_bits());
        assert_eq!([1, 1, 1, 1, 1], state.ld_op_bits());
        assert_eq!([1, 1], state.hd_op_bits());
        assert_eq!([0, 0], state.ctx_stack());
        assert_eq!([24, 0, 0, 0, 0, 0, 0, 0], state.user_stack());

        // execute false branch
        let inputs = ProgramInputs::new(&[5, 3], &[0], &[]);
        let (trace, ctx_depth, loop_depth) = super::execute(&program, 0, &inputs);
        let trace_length = trace[0].len();

        assert_eq!(128, trace_length);
        assert_eq!(19, trace.len());

        let mut state = build_trace_state(trace.len(), ctx_depth, loop_depth) ;
        state.update_from_trace(&trace, trace_length - 1);

        assert_eq!(92, state.op_counter());
        assert_eq!(program.hash(), as_bytes(state.program_hash()));
        assert_eq!([1, 1, 1], state.cf_op_bits());
        assert_eq!([1, 1, 1, 1, 1], state.ld_op_bits());
        assert_eq!([1, 1], state.hd_op_bits());
        assert_eq!([0, 0], state.ctx_stack());
        assert_eq!([96, 3, 0, 0, 0, 0, 0, 0], state.user_stack());
    }

    #[test]
    fn execute_loop() {
        let program = assembly::compile(
            "begin mul read while.true dup mul read end end",
            blake3).unwrap();

        // don't enter the loop
        let inputs = ProgramInputs::new(&[5, 3], &[0], &[]);
        let (trace, ctx_depth, loop_depth) = super::execute(&program, 0, &inputs);
        let trace_length = trace[0].len();

        assert_eq!(64, trace_length);
        assert_eq!(18, trace.len());

        let mut state = build_trace_state(trace.len(), ctx_depth, loop_depth) ;
        state.update_from_trace(&trace, trace_length - 1);

        assert_eq!(60, state.op_counter());
        assert_eq!(program.hash(), as_bytes(state.program_hash()));
        assert_eq!([1, 1, 1], state.cf_op_bits());
        assert_eq!([1, 1, 1, 1, 1], state.ld_op_bits());
        assert_eq!([1, 1], state.hd_op_bits());
        assert_eq!([0, 0], state.ctx_stack());
        assert_eq!([15, 0, 0, 0, 0, 0, 0, 0], state.user_stack());

        // execute one iteration
        let inputs = ProgramInputs::new(&[5, 3], &[1, 0], &[]);
        let (trace, ctx_depth, loop_depth) = super::execute(&program, 0, &inputs);
        let trace_length = trace[0].len();

        assert_eq!(128, trace_length);
        assert_eq!(19, trace.len());

        let mut state = build_trace_state(trace.len(), ctx_depth, loop_depth) ;
        state.update_from_trace(&trace, trace_length - 1);

        assert_eq!(75, state.op_counter());
        assert_eq!(program.hash(), as_bytes(state.program_hash()));
        assert_eq!([1, 1, 1], state.cf_op_bits());
        assert_eq!([1, 1, 1, 1, 1], state.ld_op_bits());
        assert_eq!([1, 1], state.hd_op_bits());
        assert_eq!([0, 0], state.ctx_stack());
        assert_eq!([0], state.loop_stack());
        assert_eq!([225, 0, 0, 0, 0, 0, 0, 0], state.user_stack());

        // execute five iteration
        let inputs = ProgramInputs::new(&[5, 3], &[1, 1, 1, 1, 1, 0], &[]);
        let (trace, ctx_depth, loop_depth) = super::execute(&program, 0, &inputs);
        let trace_length = trace[0].len();

        assert_eq!(256, trace_length);
        assert_eq!(19, trace.len());

        let mut state = build_trace_state(trace.len(), ctx_depth, loop_depth) ;
        state.update_from_trace(&trace, trace_length - 1);

        assert_eq!(135, state.op_counter());
        assert_eq!(program.hash(), as_bytes(state.program_hash()));
        assert_eq!([1, 1, 1], state.cf_op_bits());
        assert_eq!([1, 1, 1, 1, 1], state.ld_op_bits());
        assert_eq!([1, 1], state.hd_op_bits());
        assert_eq!([0, 0], state.ctx_stack());
        assert_eq!([0], state.loop_stack());
        assert_eq!([43143988327398919500410556793212890625, 0, 0, 0, 0, 0, 0, 0], state.user_stack());
    }

    fn build_trace_state(num_registers: usize, ctx_depth: usize, loop_depth: usize) -> TraceState {
        let decoder_width = TraceState::compute_decoder_width(ctx_depth, loop_depth);
        let stack_depth = num_registers - decoder_width;
        return TraceState::new(ctx_depth, loop_depth, stack_depth);
    }
}