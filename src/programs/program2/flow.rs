use crate::opcodes;
use super::{ Span, hash_seq };

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