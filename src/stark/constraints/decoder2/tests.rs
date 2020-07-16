use std::panic::catch_unwind;
use super::{ TraceState, Decoder };

// OP BIT TESTS
// ================================================================================================

#[test]
fn op_bits_are_binary() {

    let mut state1 = TraceState::new(1, 0, 1);
    let state2 = TraceState::new(1, 0, 1);

    let decoder = Decoder::new(1, 0);

    let mut result = vec![0; 12];

    state1.set_op_bits([0, 0, 0, 1, 1, 1, 1, 1, 1, 1]);
    decoder.check_op_bits(&state1, &state2, &mut result);
    assert_eq!(vec![0; 12], result);

    state1.set_op_bits([2, 0, 0, 1, 1, 1, 1, 1, 1, 1]);
    let t = catch_unwind(|| {
        let mut result = vec![0; 12];
        decoder.check_op_bits(&state1, &state2, &mut result);
        assert_eq!(vec![0; 12], result);
    });
    assert_eq!(t.is_ok(), false);
}