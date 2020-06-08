use crate::math::{ F128, FiniteField };
use super::{ init_stack, get_stack_state, get_aux_state, TRACE_LENGTH };
use super::super::StackTrace;

// EQUALITY OPERATION
// ================================================================================================

#[test]
fn eq() {
    let mut stack = init_stack(&[3, 3, 4, 5], &[], &[], TRACE_LENGTH);

    stack.eq(0);
    assert_eq!(vec![1, 0], get_aux_state(&stack, 0));
    assert_eq!(vec![1, 4, 5, 0, 0, 0, 0, 0], get_stack_state(&stack, 1));

    assert_eq!(3, stack.depth);
    assert_eq!(4, stack.max_depth);

    stack.eq(1);
    let inv_diff = F128::inv(F128::sub(1, 4));
    assert_eq!(vec![inv_diff, 0], get_aux_state(&stack, 1));
    assert_eq!(vec![0, 5, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 2));

    assert_eq!(2, stack.depth);
    assert_eq!(4, stack.max_depth);
}

// COMPARISON OPERATION
// ================================================================================================

#[test]
fn cmp_128() {

    let a: u128 = F128::rand();
    let b: u128 = F128::rand();
    let p127: u128 = F128::exp(2, 127);
    
    // initialize the stack
    let (inputs_a, inputs_b) = build_inputs_for_cmp(a, b, 128);
    let mut stack = init_stack(&[0, 0, 0, 0, 0, 0, a, b], &inputs_a, &inputs_b, 256);
    stack.push(0, p127);

    // execute CMP operations
    for i in 1..129 {
        stack.cmp(i);

        let state = get_stack_state(&stack, i);

        let gt = state[3];
        let lt = state[4];
        let not_set = F128::mul(F128::sub(F128::ONE, gt), F128::sub(F128::ONE, lt));
        assert_eq!(vec![not_set, F128::ZERO], get_aux_state(&stack, i));
    }

    // check the result
    let lt = if a < b { F128::ONE }  else { F128::ZERO };
    let gt = if a < b { F128::ZERO } else { F128::ONE  };

    let state = get_stack_state(&stack, 129);
    assert_eq!([gt, lt, b, a], state[3..7]);
}

#[test]
fn cmp_64() {

    let a: u128 = (F128::rand() as u64) as u128;
    let b: u128 = (F128::rand() as u64) as u128;
    let p63: u128 = F128::exp(2, 63);
    
    // initialize the stack
    let (inputs_a, inputs_b) = build_inputs_for_cmp(a, b, 64);
    let mut stack = init_stack(&[0, 0, 0, 0, 0, 0, a, b], &inputs_a, &inputs_b, 256);
    stack.push(0, p63);

    // execute CMP operations
    for i in 1..65 {
        stack.cmp(i);

        let state = get_stack_state(&stack, i);

        let gt = state[3];
        let lt = state[4];
        let not_set = F128::mul(F128::sub(F128::ONE, gt), F128::sub(F128::ONE, lt));
        assert_eq!(vec![not_set, F128::ZERO], get_aux_state(&stack, i));
    }

    // check the result
    let lt = if a < b { F128::ONE }  else { F128::ZERO };
    let gt = if a < b { F128::ZERO } else { F128::ONE  };

    let state = get_stack_state(&stack, 65);
    assert_eq!([gt, lt, b, a], state[3..7]);
}

// COMPARISON PROGRAMS
// ================================================================================================

#[test]
fn lt() {

    let a: u128 = F128::rand();
    let b: u128 = F128::rand();
    let p127: u128 = F128::exp(2, 127);
    
    // initialize the stack
    let (inputs_a, inputs_b) = build_inputs_for_cmp(a, b, 128);
    let mut stack = init_stack(&[0, 0, 0, 0, a, b, 7, 11], &inputs_a, &inputs_b, 256);
    stack.pad2(0);
    stack.push(1, p127);

    // execute CMP operations
    for i in 2..130 { stack.cmp(i); }

    // execute program finale
    let step = lt_finale(&mut stack, 130);

    // check the result
    let state = get_stack_state(&stack, step);
    let expected = if a < b { F128::ONE }  else { F128::ZERO };
    assert_eq!(vec![expected, 7, 11, 0, 0, 0, 0, 0, 0, 0, 0], state);
}

#[test]
fn gt() {

    let a: u128 = F128::rand();
    let b: u128 = F128::rand();
    let p127: u128 = F128::exp(2, 127);
    
    // initialize the stack
    let (inputs_a, inputs_b) = build_inputs_for_cmp(a, b, 128);
    let mut stack = init_stack(&[0, 0, 0, 0, a, b, 7, 11], &inputs_a, &inputs_b, 256);
    stack.pad2(0);
    stack.push(1, p127);

    // execute CMP operations
    for i in 2..130 { stack.cmp(i); }

    // execute program finale
    let step = gt_finale(&mut stack, 130);

    // check the result
    let state = get_stack_state(&stack, step);
    let expected = if a > b { F128::ONE }  else { F128::ZERO };
    assert_eq!(vec![expected, 7, 11, 0, 0, 0, 0, 0, 0, 0, 0], state);
}

// HELPER FUNCTIONS
// ================================================================================================
fn build_inputs_for_cmp(a: u128, b: u128, size: usize) -> (Vec<u128>, Vec<u128>) {

    let mut inputs_a = Vec::new();
    let mut inputs_b = Vec::new();
    for i in 0..size {
        inputs_a.push((a >> i) & 1);
        inputs_b.push((b >> i) & 1);
    }
    inputs_a.reverse();
    inputs_b.reverse();

    return (inputs_a, inputs_b);
}

fn lt_finale(stack: &mut StackTrace<u128>, step: usize) -> usize {
    stack.drop(step + 0);
    stack.swap4(step + 1);
    stack.roll4(step + 2);
    stack.eq(step + 3);
    stack.assert(step + 4);
    stack.eq(step + 5);
    stack.assert(step + 6);
    stack.drop(step + 7);
    stack.drop(step + 8);
    stack.drop(step + 9);
    return step + 10;
}

fn gt_finale(stack: &mut StackTrace<u128>, step: usize) -> usize {
    stack.drop(step + 0);
    stack.swap4(step + 1);
    stack.roll4(step + 2);
    stack.eq(step + 3);
    stack.assert(step + 4);
    stack.eq(step + 5);
    stack.assert(step + 6);
    stack.drop(step + 7);
    stack.drop(step + 8);
    stack.swap(step + 9);
    stack.drop(step + 10);
    return step + 11;
}