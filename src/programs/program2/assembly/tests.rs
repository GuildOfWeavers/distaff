#[test]
fn linear_assembly() {
    let source = "begin push.1 push.2 add end";
    let program = super::compile(source);

    match &program.body()[0] {
        super::ProgramBlock::Span(block) => {
            for i in 0..block.length() {
                println!("{}: {:?}", i, block.get_op(i));
            }
        },
        _ => ()
    }

    assert_eq!(1, 2);
}