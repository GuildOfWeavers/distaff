#[test]
fn linear_assembly() {
    let source = "begin push.1 push.2 add end";
    let program = super::compile(source).unwrap();

    match &program.body()[0] {
        super::ProgramBlock::Span(block) => {
            for i in 0..block.length() {
                println!("{}: {:?}", i, block.get_op(i));
            }
        },
        _ => ()
    }

    assert_eq!(1, 1);
}

#[test]
fn branching_assembly() {
    let source = "
    begin
        push.3
        push.5
        read
        if.true
            add
        else
            mul
        end
    end";
    let program = super::compile(source);

    assert_eq!(1, 2);
}