use super::{ opcodes, Span, HashMap, ExecutionHint };

#[test]
fn span_hash() {
    // hash noop operations
    let block = Span::from_instructions(vec![
        opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
        opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
        opcodes::NOOP, opcodes::NOOP, opcodes::NOOP, opcodes::NOOP,
        opcodes::NOOP, opcodes::NOOP, opcodes::NOOP
    ]);

    let hash = block.hash([0, 0, 0, 0]);
    assert_eq!([
        157316262623033666713538203058938064692, 80197455177734037203720349496951896931,
            68421092599366047480951655047179627665, 80949210058808928588856268130226361227,
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
        85369190300427710998622643733359886806, 215794304827495802485341382487907232249,
        70401873096594455076423121418488093540, 172342600926520679431223305032484622923,
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
            24020747447884318572054407156629291452,   8692602976904114086461881490969072192,
        274663519746421350445504800760290377716, 338505076190505971725325594571505821280,
    ], hash);
}