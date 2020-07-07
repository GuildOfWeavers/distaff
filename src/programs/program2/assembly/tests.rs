#[test]
fn linear_assembly() {
    let source = "begin push.1 push.2 add end";
    let program = super::compile(source).unwrap();

    let expected = "begin \
        push(1) noop noop noop noop noop noop noop \
        push(2) add noop noop noop noop noop end";

    assert_eq!(expected, format!("{:?}", program));
}

#[test]
fn branching_assembly() {
    let source = "
    begin
        push.3
        push.5
        read
        if.true
            add dup mul
        else
            mul dup add
        end
    end";
    let program = super::compile(source).unwrap();

    let expected = "begin \
        push(3) noop noop noop noop noop noop noop \
        push(5) read noop noop noop noop noop if \
        assert add dup mul noop noop noop noop \
        noop noop noop noop noop noop noop else \
        not assert mul dup add noop noop noop \
        noop noop noop noop noop noop noop end \
        end";

    assert_eq!(expected, format!("{:?}", program));
}