use super::{ opcodes::f128 as opcodes };

mod parsers;
use parsers::*;

pub fn translate(assembly: &str) -> Vec<u128> {

    let mut program = vec![opcodes::BEGIN];

    for token in assembly.split_whitespace() {

        let parts: Vec<&str> = token.split(".").collect();

        match parts[0] {
            "noop"   => parse_noop(&mut program, &parts),
            "assert" => parse_assert(&mut program, &parts),

            "push"   => parse_push(&mut program, &parts),
            "read"   => parse_read(&mut program, &parts),

            "dup"    => parse_dup(&mut program, &parts),
            "pad"    => parse_pad(&mut program, &parts),
            "drop"   => parse_drop(&mut program, &parts),
            "swap"   => parse_swap(&mut program, &parts),
            "roll"   => parse_roll(&mut program, &parts),

            "add"    => parse_add(&mut program, &parts),
            "sub"    => parse_sub(&mut program, &parts),
            "mul"    => parse_mul(&mut program, &parts),
            "div"    => parse_div(&mut program, &parts),
            "neg"    => parse_neg(&mut program, &parts),
            "inv"    => parse_inv(&mut program, &parts),
            "not"    => parse_not(&mut program, &parts),

            "eq"     => parse_eq(&mut program, &parts),
            "gt"     => parse_gt(&mut program, &parts),
            "lt"     => parse_lt(&mut program, &parts),
            "rc"     => parse_rc(&mut program, &parts),

            "choose" => parse_choose(&mut program, &parts),

            "hash"   => parse_hash(&mut program, &parts),
            "mpath"  => parse_mpath(&mut program, &parts),

            _ => {
                println!("not found: {:?}", parts);
            }
        }
    }

    return program;
}
