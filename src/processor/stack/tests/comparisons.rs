use crate::math::{ field };
use super::{ init_stack, get_stack_state, OpCode, OpHint, TRACE_LENGTH };
use super::super::Stack;

// EQUALITY OPERATION
// ================================================================================================

#[test]
fn eq() {
    let inv_diff = field::inv(field::sub(1, 4));
    let mut stack = init_stack(&[3, 3, 4, 5], &[0, inv_diff], &[], TRACE_LENGTH);

    stack.execute(OpCode::Read, OpHint::None);
    stack.execute(OpCode::Eq, OpHint::None);
    assert_eq!(vec![1, 4, 5, 0, 0, 0, 0, 0], get_stack_state(&stack, 2));

    assert_eq!(3, stack.depth);
    assert_eq!(5, stack.max_depth);

    stack.execute(OpCode::Read, OpHint::None);
    stack.execute(OpCode::Eq, OpHint::None);
    assert_eq!(vec![0, 5, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 4));

    assert_eq!(2, stack.depth);
    assert_eq!(5, stack.max_depth);
}

#[test]
fn eq_with_hint() {
    let mut stack = init_stack(&[3, 3, 4, 5], &[], &[], TRACE_LENGTH);

    stack.execute(OpCode::Read, OpHint::EqStart);
    stack.execute(OpCode::Eq, OpHint::None);
    assert_eq!(vec![1, 4, 5, 0, 0, 0, 0, 0], get_stack_state(&stack, 2));

    assert_eq!(3, stack.depth);
    assert_eq!(5, stack.max_depth);

    stack.execute(OpCode::Read, OpHint::EqStart);
    stack.execute(OpCode::Eq, OpHint::None);
    assert_eq!(vec![0, 5, 0, 0, 0, 0, 0, 0], get_stack_state(&stack, 4));

    assert_eq!(2, stack.depth);
    assert_eq!(5, stack.max_depth);
}

// COMPARISON OPERATION
// ================================================================================================

#[test]
fn cmp_128() {

    let a: u128 = field::rand();
    let b: u128 = field::rand();
    let p127: u128 = field::exp(2, 127);
    
    // initialize the stack
    let (inputs_a, inputs_b) = build_inputs_for_cmp(a, b, 128);
    let mut stack = init_stack(&[0, 0, 0, 0, 0, a, b], &inputs_a, &inputs_b, 256);
    stack.execute(OpCode::Pad2, OpHint::None);
    stack.execute(OpCode::Push, OpHint::PushValue(p127));

    // execute CMP operations
    for i in 2..130 {
        stack.execute(OpCode::Cmp, OpHint::None);

        let state = get_stack_state(&stack, i);
        let next  = get_stack_state(&stack, i + 1);

        let gt = state[4];
        let lt = state[5];
        let not_set = field::mul(field::sub(field::ONE, gt), field::sub(field::ONE, lt));
        assert_eq!(not_set, next[3]);
    }

    // check the result
    let lt = if a < b { field::ONE }  else { field::ZERO };
    let gt = if a < b { field::ZERO } else { field::ONE  };

    let state = get_stack_state(&stack, 130);
    assert_eq!([gt, lt, b, a], state[4..8]);
}

#[test]
fn cmp_64() {

    let a: u128 = (field::rand() as u64) as u128;
    let b: u128 = (field::rand() as u64) as u128;
    let p63: u128 = field::exp(2, 63);
    
    // initialize the stack
    let (inputs_a, inputs_b) = build_inputs_for_cmp(a, b, 64);
    let mut stack = init_stack(&[0, 0, 0, 0, 0, a, b], &inputs_a, &inputs_b, 256);
    stack.execute(OpCode::Pad2, OpHint::None);
    stack.execute(OpCode::Push, OpHint::PushValue(p63));

    // execute CMP operations
    for i in 2..66 {
        stack.execute(OpCode::Cmp, OpHint::None);

        let state = get_stack_state(&stack, i);
        let next  = get_stack_state(&stack, i + 1);

        let gt = state[4];
        let lt = state[5];
        let not_set = field::mul(field::sub(field::ONE, gt), field::sub(field::ONE, lt));
        assert_eq!(not_set, next[3]);
    }

    // check the result
    let lt = if a < b { field::ONE }  else { field::ZERO };
    let gt = if a < b { field::ZERO } else { field::ONE  };

    let state = get_stack_state(&stack, 66);
    assert_eq!([gt, lt, b, a], state[4..8]);
}

// COMPARISON PROGRAMS
// ================================================================================================

#[test]
fn lt() {

    let a: u128 = field::rand();
    let b: u128 = field::rand();
    let p127: u128 = field::exp(2, 127);
    
    // initialize the stack
    let (inputs_a, inputs_b) = build_inputs_for_cmp(a, b, 128);
    let mut stack = init_stack(&[0, 0, 0, a, b, 7, 11], &inputs_a, &inputs_b, 256);
    stack.execute(OpCode::Pad2, OpHint::None);
    stack.execute(OpCode::Pad2, OpHint::None);
    stack.execute(OpCode::Push, OpHint::PushValue(p127));

    // execute CMP operations
    for _ in 3..131 { stack.execute(OpCode::Cmp, OpHint::None); }

    // execute program finale
    lt_finale(&mut stack);

    // check the result
    let state = get_stack_state(&stack, stack.current_step());
    let expected = if a < b { field::ONE }  else { field::ZERO };
    assert_eq!(vec![expected, 7, 11, 0, 0, 0, 0, 0, 0, 0, 0, 0], state);
}

#[test]
fn gt() {

    let a: u128 = field::rand();
    let b: u128 = field::rand();
    let p127: u128 = field::exp(2, 127);
    
    // initialize the stack
    let (inputs_a, inputs_b) = build_inputs_for_cmp(a, b, 128);
    let mut stack = init_stack(&[0, 0, 0, a, b, 7, 11], &inputs_a, &inputs_b, 256);
    stack.execute(OpCode::Pad2, OpHint::None);
    stack.execute(OpCode::Pad2, OpHint::None);
    stack.execute(OpCode::Push, OpHint::PushValue(p127));

    // execute CMP operations
    for _ in 3..131 { stack.execute(OpCode::Cmp, OpHint::None); }

    // execute program finale
    gt_finale(&mut stack);

    // check the result
    let state = get_stack_state(&stack, stack.current_step());
    let expected = if a > b { field::ONE }  else { field::ZERO };
    assert_eq!(vec![expected, 7, 11, 0, 0, 0, 0, 0, 0, 0, 0, 0], state);
}

// BINARY DECOMPOSITION
// ================================================================================================

#[test]
fn binacc_128() {

    let x: u128 = field::rand();
        
    // initialize the stack
    let mut inputs_a = Vec::new();
    for i in 0..128 { inputs_a.push((x >> (127 - i)) & 1); }
    inputs_a.reverse();

    let mut stack = init_stack(
        &[0, 0, 1, 0, x, 7, 11],
        &inputs_a,
        &[],
        256);

    // execute binary aggregation operations
    for _ in 0..128 { stack.execute(OpCode::BinAcc, OpHint::None); }

    // check the result
    stack.execute(OpCode::Drop, OpHint::None);
    stack.execute(OpCode::Drop, OpHint::None);
    stack.execute(OpCode::Drop, OpHint::None);
    let state = get_stack_state(&stack, 131);
    assert_eq!(vec![x, x, 7, 11, 0, 0, 0, 0], state);
}

#[test]
fn binacc_64() {

    let x: u128 = (field::rand() as u64) as u128;

    // initialize the stack
    let mut inputs_a = Vec::new();
    for i in 0..64 { inputs_a.push((x >> (63 - i)) & 1); }
    inputs_a.reverse();

    let mut stack = init_stack(
        &[0, 0, 1, 0, x, 7, 11],
        &inputs_a,
        &[],
        256);

    // execute binary aggregation operations
    for _ in 0..64 { stack.execute(OpCode::BinAcc, OpHint::None); }

    // check the result
    stack.execute(OpCode::Drop, OpHint::None);
    stack.execute(OpCode::Drop, OpHint::None);
    stack.execute(OpCode::Drop, OpHint::None);
    let state = get_stack_state(&stack, 67);
    assert_eq!(vec![x, x, 7, 11, 0, 0, 0, 0], state);
}

#[test]
fn isodd_128() {

    let x: u128 = field::rand();
    let is_odd = x & 1;
    
    // initialize the stack
    let mut inputs_a = Vec::new();
    for i in 0..128 { inputs_a.push((x >> (127 - i)) & 1); }
    inputs_a.reverse();

    let mut stack = init_stack(
        &[0, 0, 1, 0, x, 7, 11],
        &inputs_a,
        &[],
        256);

    // read the first bit and make sure it is saved at the end of the stack
    stack.execute(OpCode::BinAcc, OpHint::None);
    stack.execute(OpCode::Swap2, OpHint::None);
    stack.execute(OpCode::Roll4, OpHint::None);
    stack.execute(OpCode::Dup, OpHint::None);

    // execute remaining binary aggregation operations
    for _ in 0..127 { stack.execute(OpCode::BinAcc, OpHint::None); }

    // check the result
    stack.execute(OpCode::Drop, OpHint::None);
    stack.execute(OpCode::Drop, OpHint::None);
    stack.execute(OpCode::Swap, OpHint::None);
    stack.execute(OpCode::Roll4, OpHint::None);
    stack.execute(OpCode::AssertEq, OpHint::None);
    stack.execute(OpCode::Drop, OpHint::None);
    let state = get_stack_state(&stack, 137);
    assert_eq!(vec![is_odd, 7, 11, 0, 0, 0, 0, 0], state);
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

fn lt_finale(stack: &mut Stack) {
    stack.execute(OpCode::Drop4, OpHint::None);
    stack.execute(OpCode::Pad2, OpHint::None);
    stack.execute(OpCode::Swap4, OpHint::None);
    stack.execute(OpCode::Roll4, OpHint::None);
    stack.execute(OpCode::AssertEq, OpHint::None);
    stack.execute(OpCode::AssertEq, OpHint::None);
    stack.execute(OpCode::Dup, OpHint::None);
    stack.execute(OpCode::Drop4, OpHint::None);
}

fn gt_finale(stack: &mut Stack) {
    stack.execute(OpCode::Drop4, OpHint::None);
    stack.execute(OpCode::Pad2, OpHint::None);
    stack.execute(OpCode::Swap4, OpHint::None);
    stack.execute(OpCode::Roll4, OpHint::None);
    stack.execute(OpCode::AssertEq, OpHint::None);
    stack.execute(OpCode::AssertEq, OpHint::None);
    stack.execute(OpCode::Roll4, OpHint::None);
    stack.execute(OpCode::Dup, OpHint::None);
    stack.execute(OpCode::Drop4, OpHint::None);
}