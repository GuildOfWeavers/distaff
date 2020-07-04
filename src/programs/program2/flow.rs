use crate::opcodes;
use super::{ Span, hash_acc, hash_seq };

// TYPES AND INTERFACES
// ================================================================================================
pub enum ProgramBlock {
    Span(Span),
    Group(Group),
    Switch(Switch),
    Loop(Loop),
}

pub struct Group {
    blocks      : Vec<ProgramBlock>,
}

pub struct Switch {
    t_branch    : Vec<ProgramBlock>,
    f_branch    : Vec<ProgramBlock>,
}

pub struct Loop {
    body        : Vec<ProgramBlock>,
    skip        : Vec<ProgramBlock>,
}

// GROUP IMPLEMENTATION
// ================================================================================================
impl Group {

    pub fn new(blocks: Vec<ProgramBlock>) -> Group {
        // TODO: first block is an instruction block
        // TODO: instruction block is not followed by an instruction block
        // TODO: number of instructions in instruction blocks must be valid

        return Group { blocks };
    }

    pub fn blocks(&self) -> &[ProgramBlock] {
        return &self.blocks;
    }

    pub fn hash(&self, state: [u128; 4]) -> [u128; 4] {
        let v0 = hash_seq(&self.blocks);
        return hash_acc(state[0], v0, 0);
    }
}

// SWITCH IMPLEMENTATION
// ================================================================================================
impl Switch {

    pub fn new(true_branch: Vec<ProgramBlock>, false_branch: Vec<ProgramBlock>) -> Switch {
        // TODO: first block is an instruction block
        // TODO: instruction block is not followed by an instruction block
        // TODO: number of instructions in instruction blocks must be valid

        return Switch {
            t_branch    : true_branch,
            f_branch    : false_branch
        };
    }

    pub fn hash(&self, state: [u128; 4]) -> [u128; 4] {
        let v0 = hash_seq(&self.t_branch);
        let v1 = hash_seq(&self.f_branch);
        return hash_acc(state[0], v0, v1);
    }
}

// LOOP IMPLEMENTATION
// ================================================================================================
impl Loop {

    pub fn new(body: Vec<ProgramBlock>) -> Loop {
        // TODO: first block is an instruction block
        // TODO: instruction block is not followed by an instruction block
        // TODO: number of instructions in instruction blocks must be valid

        let skip_block = Span::from_instructions(vec![
            opcodes::NOT,  opcodes::ASSERT, opcodes::NOOP, opcodes::NOOP,
            opcodes::NOOP, opcodes::NOOP,   opcodes::NOOP, opcodes::NOOP,
            opcodes::NOOP, opcodes::NOOP,   opcodes::NOOP, opcodes::NOOP,
            opcodes::NOOP, opcodes::NOOP,   opcodes::NOOP, opcodes::NOOP,
        ]);

        let skip = vec![ProgramBlock::Span(skip_block)];

        return Loop { body, skip };
    }

    pub fn hash(&self, state: [u128; 4]) -> [u128; 4] {
        let v0 = hash_seq(&self.body);
        let v1 = hash_seq(&self.skip);
        return hash_acc(state[0], v0, v1);
    }
}