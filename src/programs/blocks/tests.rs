use super::{ OpCode, Span, HashMap, OpHint };

#[test]
fn span_hash() {
    // hash noop operations
    let block = Span::from_instructions(vec![
        OpCode::Noop, OpCode::Noop, OpCode::Noop, OpCode::Noop,
        OpCode::Noop, OpCode::Noop, OpCode::Noop, OpCode::Noop,
        OpCode::Noop, OpCode::Noop, OpCode::Noop, OpCode::Noop,
        OpCode::Noop, OpCode::Noop, OpCode::Noop
    ]);

    let hash = block.hash([0, 0, 0, 0]);
    assert_eq!([
        283855050660402859567809346597024356257, 290430270201175202384178252750741838599,
         33642161455895506272337605785278290375, 114906032113415280284656928780040029722,
    ], hash);

    // hash noops and a push operation
    let mut hints = HashMap::new();
    hints.insert(8, OpHint::PushValue(1));
    let block = Span::new(vec![
        OpCode::Noop, OpCode::Noop, OpCode::Noop, OpCode::Noop,
        OpCode::Noop, OpCode::Noop, OpCode::Noop, OpCode::Noop,
        OpCode::Push, OpCode::Noop, OpCode::Noop, OpCode::Noop,
        OpCode::Noop, OpCode::Noop, OpCode::Noop
    ], hints);

    let hash = block.hash([0, 0, 0, 0]);
    assert_eq!([
        309939768290184920181146334415666126639, 189522128575407709345588553132211127638,
        300449513105356487315600679523377528535, 201241536410685268433124688525928056833,
    ], hash);

    // hash noops and a push operation with a different value
    let mut hints = HashMap::new();
    hints.insert(8, OpHint::PushValue(2));
    let block = Span::new(vec![
        OpCode::Noop, OpCode::Noop, OpCode::Noop, OpCode::Noop,
        OpCode::Noop, OpCode::Noop, OpCode::Noop, OpCode::Noop,
        OpCode::Push, OpCode::Noop, OpCode::Noop, OpCode::Noop,
        OpCode::Noop, OpCode::Noop, OpCode::Noop
    ], hints);

    let hash = block.hash([0, 0, 0, 0]);
    assert_eq!([
        238085520613464573032580920836572617149,  98362585914038709664139524327351111560,
        159064915881679512167348007665307977960, 152057468867502483682425300737565245134,
    ], hash);
}