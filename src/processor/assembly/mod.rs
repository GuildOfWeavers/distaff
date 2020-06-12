use super::{ opcodes::f128 as opcodes, pad_program };

mod parsers;
use parsers::*;

mod errors;
use errors::{ AssemblyError };

// ASSEMBLER
// ================================================================================================

pub fn translate(assembly: &str) -> Result<Vec<u128>, AssemblyError> {

    // all programs must start with BEGIN operation
    let mut program = vec![opcodes::BEGIN];

    // break assembly string into instruction and apply an appropriate parser to each instruction
    let mut step = 0;
    for token in assembly.split_whitespace() {

        let op: Vec<&str> = token.split(".").collect();
        step += 1;

        match op[0] {
            "noop"   => parse_noop(&mut program, &op, step),
            "assert" => parse_assert(&mut program, &op, step),

            "push"   => parse_push(&mut program, &op, step),
            "read"   => parse_read(&mut program, &op, step),

            "dup"    => parse_dup(&mut program, &op, step),
            "pad"    => parse_pad(&mut program, &op, step),
            "pick"   => parse_pick(&mut program, &op, step),
            "drop"   => parse_drop(&mut program, &op, step),
            "swap"   => parse_swap(&mut program, &op, step),
            "roll"   => parse_roll(&mut program, &op, step),

            "add"    => parse_add(&mut program, &op, step),
            "sub"    => parse_sub(&mut program, &op, step),
            "mul"    => parse_mul(&mut program, &op, step),
            "div"    => parse_div(&mut program, &op, step),
            "neg"    => parse_neg(&mut program, &op, step),
            "inv"    => parse_inv(&mut program, &op, step),
            "not"    => parse_not(&mut program, &op, step),

            "eq"     => parse_eq(&mut program, &op, step),
            "gt"     => parse_gt(&mut program, &op, step),
            "lt"     => parse_lt(&mut program, &op, step),
            "rc"     => parse_rc(&mut program, &op, step),
            "cmp"    => parse_cmp(&mut program, &op, step),
            "binacc" => parse_binacc(&mut program, &op, step),

            "choose" => parse_choose(&mut program, &op, step),

            "hash"   => parse_hash(&mut program, &op, step),
            "mpath"  => parse_mpath(&mut program, &op, step),

            _ => return Err(AssemblyError::invalid_op(&op, step))
        }?;
    }

    // pad the program with the appropriate number of NOOPs
    pad_program(&mut program);

    return Ok(program);
}