use std::collections::HashMap;
use crate::opcodes;
use super::hash_op;

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

pub struct Span {
    op_codes    : Vec<u8>,
    op_hints    : HashMap<usize, ExecutionHint>,
}

// SPAN IMPLEMENTATION
// ================================================================================================
impl Span {

    pub fn new(instructions: Vec<u8>, hints: HashMap<usize, ExecutionHint>) -> Span {
        assert!(instructions.len() > 0, "instruction span must contain at least one instruction");

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

    pub fn from_instructions(instructions: Vec<u8>) -> Span {
        return Span::new(instructions, HashMap::new());
    }

    pub fn length(&self) -> usize {
        return self.op_codes.len();
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
            if op_code == opcodes::PUSH {
                match self.get_hint(i) {
                    ExecutionHint::PushValue(op_value) => hash_op(&mut state, op_code, op_value, i),
                    _ => panic!("value for PUSH operation is missing")
                }
            }
            else {
                hash_op(&mut state, op_code, 0, i);
            }
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
            opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
        ]);

        let hash = block.hash([0, 0, 0, 0]);
        assert_eq!([
              8566242090173091583124763969325438871,  84636537995850117149368373965718083921,
            222657086680323995003240724209962977581, 127833818698872133570493667966884100145,
        ], hash);

        // hash noops and a push operation
        let mut hints = HashMap::new();
        hints.insert(8, ExecutionHint::PushValue(1));
        let block = Span::new(vec![
            opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
            opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
            opcodes::PUSH, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
            opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
        ], hints);

        let hash = block.hash([0, 0, 0, 0]);
        assert_eq!([
             32513308417020917531759644844383344868, 186915976146641762336738218477661156954,
              7535251945779765790057451912525515090, 278645454202396004240502843204252781361,
        ], hash);

        // hash noops and a push operation with a different value
        let mut hints = HashMap::new();
        hints.insert(8, ExecutionHint::PushValue(2));
        let block = Span::new(vec![
            opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
            opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
            opcodes::PUSH, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
            opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
        ], hints);

        let hash = block.hash([0, 0, 0, 0]);
        assert_eq!([
            149098942058300495948126997609343088395, 188197663390522895626516604937611702437,
            270603084642029093008777372688579835346, 303512582809791735737080230816116187881,
        ], hash);
    }
}