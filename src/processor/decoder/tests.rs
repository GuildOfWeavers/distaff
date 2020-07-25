/*
TODO: re-enable
use super::{ UserOps };

#[test]
fn start_block() {
    let mut decoder = super::Decoder::new(16);
    for _ in 0..15 { decoder.decode_op(UserOps::Noop, 0); }
    decoder.start_block();
    for _ in 0..16 { decoder.decode_op(UserOps::Noop, 0); }
    decoder.end_block(0, true);
    for _ in 0..14 { decoder.decode_op(UserOps::Noop, 0); }
    decoder.finalize_trace();

    for i in 0..decoder.trace_length() {
        decoder.print_state(i);
    }
    
    assert_eq!(1, 2);
}

#[test]
fn start_loop() {
    let mut decoder = super::Decoder::new(16);
    for _ in 0..15 { decoder.decode_op(UserOps::Noop, 0); }
    decoder.start_loop(34133582271386177291348118006257970896);
    for _ in 0..15 { decoder.decode_op(UserOps::Noop, 0); }
    decoder.wrap_loop();
    for _ in 0..15 { decoder.decode_op(UserOps::Noop, 0); }
    decoder.break_loop();
    decoder.end_block(0, true);
    for _ in 0..14 { decoder.decode_op(UserOps::Noop, 0); }
    decoder.finalize_trace();

    for i in 0..decoder.trace_length() {
        decoder.print_state(i);
    }
    
    assert_eq!(1, 2);
}
*/