use std::collections::HashMap;
use crate::opcodes;
use super::{ hash_op, ProgramBlock, BASE_CYCLE_LENGTH };

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
pub struct Span {
    op_codes    : Vec<u8>,
    op_hints    : HashMap<usize, ExecutionHint>,
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

// HELPER FUNCTIONS
// ================================================================================================
fn is_valid_instruction(op_code: u8) -> bool {
    // TODO: implement
    return true;
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {

    use super::{ opcodes, Span, HashMap, ExecutionHint };

    #[test]
    fn hashing() {
        // hash noop operations
        let block = Span::from_instructions(vec![
            opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
            opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
            opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
            opcodes::NOOP, opcodes::NOOP, opcodes::NOOP
        ]);

        let hash = block.hash([0, 0, 0, 0]);
        assert_eq!([
             52076011459971410147741803070378730890, 261452704326515948132660305632795635258,
            285762266873668793859003219115592205922, 212039139700064235831954673028848881811,
        ], hash);

        // hash noops and a push operation
        let mut hints = HashMap::new();
        hints.insert(8, ExecutionHint::PushValue(1));
        let block = Span::new(vec![
            opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
            opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
            opcodes::PUSH, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
            opcodes::NOOP, opcodes::NOOP, opcodes::NOOP
        ], hints);

        let hash = block.hash([0, 0, 0, 0]);
        assert_eq!([
            312507932535527141437397503257237214949, 280793603756331827274203760015152973193,
            306066058282300678308026054798662934360, 230434793681125211251604762490912238982,
        ], hash);

        // hash noops and a push operation with a different value
        let mut hints = HashMap::new();
        hints.insert(8, ExecutionHint::PushValue(2));
        let block = Span::new(vec![
            opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
            opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
            opcodes::PUSH, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
            opcodes::NOOP, opcodes::NOOP, opcodes::NOOP
        ], hints);

        let hash = block.hash([0, 0, 0, 0]);
        assert_eq!([
             86672797833154161666693060860109540914, 248770728298471452232553024868136109193,
            336242130300957214807078973789700958518, 232316973172051396384789365363965735328,
        ], hash);
    }
}