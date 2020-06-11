use super::{ opcodes::f128 as opcodes };

mod parsers;
use parsers::*;

mod errors;
use errors::{ AssemblyError };

pub fn translate(assembly: &str) -> Vec<u128> {

    let mut program = vec![opcodes::BEGIN];

    let mut step = 0;
    for token in assembly.split_whitespace() {

        let parts: Vec<&str> = token.split(".").collect();
        step += 1;

        match parts[0] {
            "noop"   => parse_noop(&mut program, &parts, step),
            "assert" => parse_assert(&mut program, &parts, step),

            "push"   => parse_push(&mut program, &parts, step),
            "read"   => parse_read(&mut program, &parts, step),

            "dup"    => parse_dup(&mut program, &parts, step),
            "pad"    => parse_pad(&mut program, &parts, step),
            "drop"   => parse_drop(&mut program, &parts, step),
            "swap"   => parse_swap(&mut program, &parts, step),
            "roll"   => parse_roll(&mut program, &parts, step),

            "add"    => parse_add(&mut program, &parts, step),
            "sub"    => parse_sub(&mut program, &parts, step),
            "mul"    => parse_mul(&mut program, &parts, step),
            "div"    => parse_div(&mut program, &parts, step),
            "neg"    => parse_neg(&mut program, &parts, step),
            "inv"    => parse_inv(&mut program, &parts, step),
            "not"    => parse_not(&mut program, &parts, step),

            "eq"     => parse_eq(&mut program, &parts, step),
            "gt"     => parse_gt(&mut program, &parts, step),
            "lt"     => parse_lt(&mut program, &parts, step),
            "rc"     => parse_rc(&mut program, &parts, step),

            "choose" => parse_choose(&mut program, &parts, step),

            "hash"   => parse_hash(&mut program, &parts, step),
            "mpath"  => parse_mpath(&mut program, &parts, step),

            _ => {
                println!("not found: {:?}", parts);
                Ok(true)
            }
        }.unwrap();
    }

    return program;
}
