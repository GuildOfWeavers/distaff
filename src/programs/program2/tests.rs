use crate::{ opcodes };
use super::{ Program, ProgramBlock, ExecutionHint, Span, hash_op };

fn traverse(block: &ProgramBlock, hash: &mut [u128; 4], mut step: usize) -> usize {

    match block {
        ProgramBlock::Span(block)   => {
            for i in 0..block.length() {
                let (op_code, op_hint) = block.get_op(i);
                let op_value = match op_hint {
                    ExecutionHint::PushValue(value) => value,
                    _ => 0,
                };
                hash_op(hash, op_code, op_value, step);
                step += 1;
            }
        },
        ProgramBlock::Group(block)  => {
            step += 1; // BEGIN
            let mut state = [0, 0, 0, 0];
            let blocks = block.blocks();
            for i in 0..blocks.len() {
                step = traverse(&blocks[i], &mut state, step);
            }
            // TODO: check alignment
            step += 1; // TEND
            //state = [state[0], 0, hash[0], 0];
            for i in 0..14 {
                // hash_acc
            }
        },
        ProgramBlock::Switch(block) => {
            
        },
        ProgramBlock::Loop(block)   => {
            
        },
    }

    return step;
}

#[test]
fn traverse_linear_path() {
    let block = Span::from_instructions(vec![
        opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
        opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
        opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
        opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
    ]);

    let program = Program::new(vec![ProgramBlock::Span(block)]);
    let body = program.body();

    let mut hash = [0, 0, 0, 0];
    for i in 0..body.len() {
        traverse(&body[i], &mut hash, 0);
    }
}