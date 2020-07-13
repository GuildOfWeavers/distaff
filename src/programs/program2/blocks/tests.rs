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
        118844146892083470514455294601677965871, 224872343428836366240517714625610433477,
         88452912995157535200154835609809700641, 299350722679819457778461266425341779937,
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
        202376941217984595534656725660487737949, 230597638008266299255765775743759397082,
         97603442173678867660521049421511291867, 258944901987071318964394217303782133754,
    ], hash);
}