use std::collections::HashMap;
use crate::opcodes;
use super::{ hash_seq, hash_op, BASE_CYCLE_LENGTH };

#[cfg(test)]
mod tests;

// TYPES AND INTERFACES
// ================================================================================================

#[derive(Copy, Clone)]
pub enum ExecutionHint {
    EqStart,
    RcStart(u32),
    CmpStart(u32),
    PushValue(u128),
    None,
}

#[derive(Clone)]
pub enum ProgramBlock {
    Span(Span),
    Group(Group),
    Switch(Switch),
    Loop(Loop),
}

#[derive(Clone)]
pub struct Span {
    op_codes    : Vec<u8>,
    op_hints    : HashMap<usize, ExecutionHint>,
}

#[derive(Clone)]
pub struct Group {
    body        : Vec<ProgramBlock>,
}

#[derive(Clone)]
pub struct Switch {
    t_branch    : Vec<ProgramBlock>,
    f_branch    : Vec<ProgramBlock>,
}

#[derive(Clone)]
pub struct Loop {
    body        : Vec<ProgramBlock>,
    skip        : Vec<ProgramBlock>,
}

// PROGRAM BLOCK IMPLEMENTATION
// ================================================================================================

impl ProgramBlock {

    pub fn is_span(&self) -> bool {
        return match self {
            ProgramBlock::Span(_) => true,
            _ => false,
        };
    }

}

// SPAN IMPLEMENTATION
// ================================================================================================
impl Span {

    pub fn new(instructions: Vec<u8>, hints: HashMap<usize, ExecutionHint>) -> Span {
        let alignment = instructions.len() % BASE_CYCLE_LENGTH;
        assert!(alignment == BASE_CYCLE_LENGTH - 1,
            "invalid number of instructions: expected one less than a multiple of {}, but was {}",
            BASE_CYCLE_LENGTH, instructions.len());

        // make sure all instructions are valid
        for i in 0..instructions.len() {
            let op_code = instructions[i];
            assert!(is_valid_instruction(op_code), "invalid instruction opcode {} at step {}", op_code, i);
            if op_code == opcodes::PUSH {
                assert!(i % 8 == 0, "PUSH is not allowed on step {}, must be on step which is a multiple of 8", i);
                let hint = hints.get(&i);
                assert!(hint.is_some(), "invalid PUSH operation on step {}: operation value is missing", i);
                match hint.unwrap() {
                    ExecutionHint::PushValue(_) => (),
                    _ => panic!("invalid PUSH operation on step {}: operation value is of wrong type", i)
                }
            }
        }

        // make sure all hints are within bounds
        for &step in hints.keys() {
            assert!(step < instructions.len(), "hint out of bounds: step must be smaller than {} but is {}",
                instructions.len(), step);
        }

        return Span {
            op_codes: instructions,
            op_hints: hints
        };
    }

    pub fn new_block(instructions: Vec<u8>) -> ProgramBlock {
        return ProgramBlock::Span(Span::new(instructions, HashMap::new()));
    }

    pub fn from_instructions(instructions: Vec<u8>) -> Span {
        return Span::new(instructions, HashMap::new());
    }

    pub fn length(&self) -> usize {
        return self.op_codes.len();
    }

    pub fn starts_with(&self, instructions: &[u8]) -> bool {
        return self.op_codes.starts_with(instructions);
    }

    pub fn get_op(&self, step: usize) -> (u8, ExecutionHint) {
        return (self.op_codes[step], self.get_hint(step));
    }

    pub fn get_hint(&self, op_index: usize) -> ExecutionHint {
        return match self.op_hints.get(&op_index) {
            Some(&hint) => hint,
            None => ExecutionHint::None
        };
    }

    pub fn hash(&self, mut state: [u128; 4]) -> [u128; 4] {
        for (i, &op_code) in self.op_codes.iter().enumerate() {
            let op_value = if op_code == opcodes::PUSH {
                match self.get_hint(i) {
                    ExecutionHint::PushValue(op_value) => op_value,
                    _ => panic!("value for PUSH operation is missing")
                }
            }
            else { 0 };
            hash_op(&mut state, op_code, op_value, i)
        }
        return state;
    }
}

// GROUP IMPLEMENTATION
// ================================================================================================
impl Group {

    pub fn new(body: Vec<ProgramBlock>) -> Group {
        validate_block_list(&body, &[]);
        return Group { body };
    }

    pub fn new_block(body: Vec<ProgramBlock>) -> ProgramBlock {
        return ProgramBlock::Group(Group::new(body));
    }

    pub fn body(&self) -> &[ProgramBlock] {
        return &self.body;
    }

    pub fn body_hash(&self) -> u128 {
        return hash_seq(&self.body, false);
    }

    pub fn get_hash(&self) -> (u128, u128) {
        let v0 = self.body_hash();
        return (v0, 0);
    }
}

// SWITCH IMPLEMENTATION
// ================================================================================================
impl Switch {

    pub fn new(true_branch: Vec<ProgramBlock>, false_branch: Vec<ProgramBlock>) -> Switch {
        validate_block_list(&true_branch, &[opcodes::ASSERT]);
        validate_block_list(&false_branch, &[opcodes::NOT, opcodes::ASSERT]);
        return Switch {
            t_branch    : true_branch,
            f_branch    : false_branch
        };
    }

    pub fn new_block(true_branch: Vec<ProgramBlock>, false_branch: Vec<ProgramBlock>) -> ProgramBlock {
        return ProgramBlock::Switch(Switch::new(true_branch, false_branch));
    }

    pub fn true_branch(&self) -> &[ProgramBlock] {
        return &self.t_branch;
    }

    pub fn true_branch_hash(&self) -> u128 {
        return hash_seq(&self.t_branch, false);
    }

    pub fn false_branch(&self) -> &[ProgramBlock] {
        return &self.f_branch;
    }

    pub fn false_branch_hash(&self) -> u128 {
        return hash_seq(&self.f_branch, false);
    }

    pub fn get_hash(&self) -> (u128, u128) {
        let v0 = self.true_branch_hash();
        let v1 = self.false_branch_hash();
        return (v0, v1);
    }
}

// LOOP IMPLEMENTATION
// ================================================================================================
impl Loop {

    pub fn new(body: Vec<ProgramBlock>) -> Loop {
        validate_block_list(&body, &[opcodes::ASSERT]);

        let skip_block = Span::from_instructions(vec![
            opcodes::NOT,  opcodes::ASSERT, opcodes::NOOP, opcodes::NOOP,
            opcodes::NOOP, opcodes::NOOP,   opcodes::NOOP, opcodes::NOOP,
            opcodes::NOOP, opcodes::NOOP,   opcodes::NOOP, opcodes::NOOP,
            opcodes::NOOP, opcodes::NOOP,   opcodes::NOOP
        ]);

        let skip = vec![ProgramBlock::Span(skip_block)];

        return Loop { body, skip };
    }

    pub fn new_block(body: Vec<ProgramBlock>) -> ProgramBlock {
        return ProgramBlock::Loop(Loop::new(body));
    }

    pub fn body(&self) -> &[ProgramBlock] {
        return &self.body;
    }

    pub fn body_hash(&self) -> u128 {
        return hash_seq(&self.body, true);
    }

    pub fn skip(&self) -> &[ProgramBlock] {
        return &self.skip;
    }

    pub fn skip_hash(&self) -> u128 {
        return hash_seq(&self.skip, false);
    }

    pub fn get_hash(&self) -> (u128, u128) {
        let v0 = self.body_hash();
        let v1 = self.skip_hash();
        return (v0, v1);
    }
}

// HELPER FUNCTIONS
// ================================================================================================

fn is_valid_instruction(op_code: u8) -> bool {
    // TODO: implement
    return true;
}

fn validate_block_list(blocks: &Vec<ProgramBlock>, starts_with: &[u8]) {

    assert!(blocks.len() > 0, "a sequence of blocks must contain at least one block");
    
    // first block must be a span block
    match &blocks[0] {
        ProgramBlock::Span(block) => {
            // if the block must start with a specific sequence of instructions, make sure it does
            if starts_with.len() > 0 {
                assert!(block.starts_with(starts_with),
                    "the first block does not start with a valid sequence of instructions");
            }
        },
        _ => panic!("a sequence of blocks must start with a Span block"),
    };

    // span block cannot be followed by another span block
    let mut was_span = true;
    for i in 1..blocks.len() {
        match &blocks[i] {
            ProgramBlock::Span(_) => {
                assert!(was_span == false, "a Span block cannot be followed by another Span block");
            },
            _ => was_span = false,
        }
    }
}