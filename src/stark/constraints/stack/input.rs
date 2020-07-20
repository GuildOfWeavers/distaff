use super::{ enforce_no_change };

/// Enforces constraints for PUSH operation. The constraints are based on the first element of the 
/// stack; the old stack is shifted right by 1 element.
pub fn enforce_push(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {

    // ensure that the rest of the stack is shifted right by 1 element
    enforce_no_change(&mut result[1..], &current[0..], &next[1..], op_flag);
}

/// Enforces constraints for READ operation. No constraints are placed on the first element of
/// the stack; the old stack is shifted right by 1 element.
pub fn enforce_read(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    enforce_no_change(&mut result[1..], &current[0..], &next[1..], op_flag);
}

/// Enforces constraints for READ2 operation. No constraints are placed on the first two elements
/// of the stack; the old stack is shifted right by 2 element.
pub fn enforce_read2(result: &mut [u128], current: &[u128], next: &[u128], op_flag: u128) {
    enforce_no_change(&mut result[2..], &current[0..], &next[2..], op_flag);
}