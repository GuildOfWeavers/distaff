use super::{ Opcode, Span, HashMap, OpHint };

#[test]
fn span_hash() {
    // hash noop operations
    let block = Span::from_instructions(vec![
        Opcode::Noop, Opcode::Noop, Opcode::Noop, Opcode::Noop,
        Opcode::Noop, Opcode::Noop, Opcode::Noop, Opcode::Noop,
        Opcode::Noop, Opcode::Noop, Opcode::Noop, Opcode::Noop,
        Opcode::Noop, Opcode::Noop, Opcode::Noop
    ]);

    let hash = block.hash([0, 0, 0, 0]);
    assert_eq!([
         34133582271386177291348118006257970896, 181876936253440791628125120546105003666,
        103931834153883155212634799354704742091, 117276145941972134574935124009307864736,
    ], hash);

    // hash noops and a push operation
    let mut hints = HashMap::new();
    hints.insert(8, OpHint::PushValue(1));
    let block = Span::new(vec![
        Opcode::Noop, Opcode::Noop, Opcode::Noop, Opcode::Noop,
        Opcode::Noop, Opcode::Noop, Opcode::Noop, Opcode::Noop,
        Opcode::Push, Opcode::Noop, Opcode::Noop, Opcode::Noop,
        Opcode::Noop, Opcode::Noop, Opcode::Noop
    ], hints);

    let hash = block.hash([0, 0, 0, 0]);
    assert_eq!([
        285304052146347300738563822185773120909, 267156608767006924578933793636440855065,
        168420360560007457644421316929053951236, 171942682764962620224706929742143306579,
    ], hash);

    // hash noops and a push operation with a different value
    let mut hints = HashMap::new();
    hints.insert(8, OpHint::PushValue(2));
    let block = Span::new(vec![
        Opcode::Noop, Opcode::Noop, Opcode::Noop, Opcode::Noop,
        Opcode::Noop, Opcode::Noop, Opcode::Noop, Opcode::Noop,
        Opcode::Push, Opcode::Noop, Opcode::Noop, Opcode::Noop,
        Opcode::Noop, Opcode::Noop, Opcode::Noop
    ], hints);

    let hash = block.hash([0, 0, 0, 0]);
    assert_eq!([
        208867221102125115481571280738406231418, 142356471514298489601201642029998007616,
        231190229887433479301832549801661607716, 59456029957482587265259622684292274960,
    ], hash);
}