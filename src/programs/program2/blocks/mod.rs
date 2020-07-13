use std::collections::HashMap;
use crate::processor::opcodes2::{ UserOps as Opcode, OpHint };
use super::{ hash_seq, hash_op, BASE_CYCLE_LENGTH };

#[cfg(test)]
mod tests;

// TYPES AND INTERFACES
// ================================================================================================

#[derive(Clone)]
pub enum ProgramBlock {
    Span(Span),
    Group(Group),
    Switch(Switch),
    Loop(Loop),
}

#[derive(Clone)]
pub struct Span {
    op_codes    : Vec<Opcode>,
    op_hints    : HashMap<usize, OpHint>,
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

impl std::fmt::Debug for ProgramBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProgramBlock::Span(block)   => write!(f, "{:?}", block)?,
            ProgramBlock::Group(block)  => write!(f, "{:?}", block)?,
            ProgramBlock::Switch(block) => write!(f, "{:?}", block)?,
            ProgramBlock::Loop(block)   => write!(f, "{:?}", block)?,
        }
        return Ok(());
    }
}

// SPAN IMPLEMENTATION
// ================================================================================================
impl Span {

    pub fn new(instructions: Vec<Opcode>, hints: HashMap<usize, OpHint>) -> Span {
        let alignment = instructions.len() % BASE_CYCLE_LENGTH;
        assert!(alignment == BASE_CYCLE_LENGTH - 1,
            "invalid number of instructions: expected one less than a multiple of {}, but was {}",
            BASE_CYCLE_LENGTH, instructions.len());

        // make sure all instructions are valid
        for i in 0..instructions.len() {
            let op_code = instructions[i];
            if op_code == Opcode::Push {
                assert!(i % 8 == 0, "PUSH is not allowed on step {}, must be on step which is a multiple of 8", i);
                let hint = hints.get(&i);
                assert!(hint.is_some(), "invalid PUSH operation on step {}: operation value is missing", i);
                match hint.unwrap() {
                    OpHint::PushValue(_) => (),
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

    pub fn new_block(instructions: Vec<Opcode>) -> ProgramBlock {
        return ProgramBlock::Span(Span::new(instructions, HashMap::new()));
    }

    pub fn from_instructions(instructions: Vec<Opcode>) -> Span {
        return Span::new(instructions, HashMap::new());
    }

    pub fn length(&self) -> usize {
        return self.op_codes.len();
    }

    pub fn starts_with(&self, instructions: &[Opcode]) -> bool {
        return self.op_codes.starts_with(instructions);
    }

    pub fn get_op(&self, step: usize) -> (Opcode, OpHint) {
        return (self.op_codes[step], self.get_hint(step));
    }

    pub fn get_hint(&self, op_index: usize) -> OpHint {
        return match self.op_hints.get(&op_index) {
            Some(&hint) => hint,
            None => OpHint::None
        };
    }

    pub fn hash(&self, mut state: [u128; 4]) -> [u128; 4] {
        for (i, &op_code) in self.op_codes.iter().enumerate() {
            let op_value = if op_code == Opcode::Push {
                match self.get_hint(i) {
                    OpHint::PushValue(op_value) => op_value,
                    _ => panic!("value for PUSH operation is missing")
                }
            }
            else { 0 };
            hash_op(&mut state, op_code as u8, op_value, i)
        }
        return state;
    }

    pub fn merge(span1: &Span, span2: &Span) -> Span {
        // merge op codes
        let mut new_op_codes = span1.op_codes.clone();
        new_op_codes.push(Opcode::Noop);
        new_op_codes.extend_from_slice(&span2.op_codes);

        // merge hints
        let offset = span1.length() + 1;
        let mut new_hints = span1.op_hints.clone();
        for (step, &hint) in &span2.op_hints {
            new_hints.insert(step + offset, hint);
        }

        // build and return a new Span
        return Span::new(new_op_codes, new_hints);
    }
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (op_code, op_hint) = self.get_op(0);
        write!(f, "{}{}", op_code, op_hint)?;

        for i in 1..self.length() {
            let (op_code, op_hint) = self.get_op(i);
            write!(f, " {}{}", op_code, op_hint)?;
        }
        return Ok(());
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

impl std::fmt::Debug for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "block ")?;
        for block in self.body.iter() {
            write!(f, "{:?} ", block)?;
        }
        write!(f, "end")
    }
}

// SWITCH IMPLEMENTATION
// ================================================================================================
impl Switch {

    pub fn new(true_branch: Vec<ProgramBlock>, false_branch: Vec<ProgramBlock>) -> Switch {
        validate_block_list(&true_branch, &[Opcode::Assert]);
        validate_block_list(&false_branch, &[Opcode::Not, Opcode::Assert]);
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

impl std::fmt::Debug for Switch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "if ")?;
        for block in self.t_branch.iter() {
            write!(f, "{:?} ", block)?;
        }
        write!(f, "else ")?;
        for block in self.f_branch.iter() {
            write!(f, "{:?} ", block)?;
        }
        write!(f, "end")
    }
}

// LOOP IMPLEMENTATION
// ================================================================================================
impl Loop {

    pub fn new(body: Vec<ProgramBlock>) -> Loop {
        validate_block_list(&body, &[Opcode::Assert]);

        let skip_block = Span::from_instructions(vec![
            Opcode::Not,  Opcode::Assert, Opcode::Noop, Opcode::Noop,
            Opcode::Noop, Opcode::Noop,   Opcode::Noop, Opcode::Noop,
            Opcode::Noop, Opcode::Noop,   Opcode::Noop, Opcode::Noop,
            Opcode::Noop, Opcode::Noop,   Opcode::Noop
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

impl std::fmt::Debug for Loop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "while ")?;
        for block in self.body.iter() {
            write!(f, "{:?} ", block)?;
        }
        write!(f, "end")
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn validate_block_list(blocks: &Vec<ProgramBlock>, starts_with: &[Opcode]) {

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