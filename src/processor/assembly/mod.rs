use crate::crypto::{ hash::blake3 };
use super::{ opcodes::f128 as opcodes, get_padded_length, Program, ExecutionGraph };

mod parsers;
use parsers::*;

mod errors;
use errors::{ AssemblyError };

#[cfg(test)]
mod tests;

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
    let padded_length = get_padded_length(program.len(), *program.last().unwrap());
    program.resize(padded_length, opcodes::NOOP);

    return Ok(program);
}

pub fn compile(assembly: &str) -> Result<Program, AssemblyError> {
    
    // all programs must start with BEGIN operation
    let mut segment_ops = vec![opcodes::BEGIN];

    // break assembly string into tokens
    let tokens: Vec<&str> = assembly.split_whitespace().collect();

    for i in 0..tokens.len() {
        match tokens[i] {
            "if.true" => {
                let true_branch = parse_true_branch(&tokens, i + 1)?;
                let i = find_matching_else(&tokens, i + 1);
                let false_branch = parse_false_branch(&tokens, i + 1)?;

                let mut exe_graph = ExecutionGraph::new(segment_ops);
                exe_graph.set_next(true_branch, false_branch);
                return Ok(Program::new(exe_graph, blake3));
            },
            "else"  => return Err(AssemblyError::unmatched_else(i)),
            "endif" => return Err(AssemblyError::unmatched_endif(i)),
            token   => parse_op_token(token, &mut segment_ops, i)
        }?;
    }

    return Ok(Program::from_path(segment_ops));
}

fn parse_true_branch(tokens: &[&str], step: usize) -> Result<ExecutionGraph, AssemblyError> {

    // true branch must start with ASSERT
    let mut segment_ops = vec![opcodes::ASSERT];

    let mut ignore_ctr = 0;

    for i in step..tokens.len() {
        match tokens[i] {
            "if.true" => {
                if ignore_ctr > 0 { continue; }
        
                let true_branch = parse_true_branch(&tokens, i + 1)?;
                let i = find_matching_else(tokens, i + 1);
                let false_branch = parse_false_branch(&tokens, i + 1)?;

                let mut graph = ExecutionGraph::new(segment_ops);
                graph.set_next(true_branch, false_branch);
                return Ok(graph);
            },
            "else" => {
                ignore_ctr += 1;
            },
            "endif" => {
                ignore_ctr -= 1;
            }
            token => {
                if ignore_ctr > 0 { continue; }
                parse_op_token(token, &mut segment_ops, i)?;
            }
        }
    }

    // TODO: check for errors
    return Ok(ExecutionGraph::new(segment_ops));
}

fn parse_false_branch(tokens: &[&str], step: usize) -> Result<ExecutionGraph, AssemblyError> {

    // false branch must start with NOT ASSERT
    let mut segment_ops = vec![opcodes::NOT, opcodes::ASSERT];

    let mut ignore_ctr = 0;
    
    for i in step..tokens.len() {

        match tokens[i] {
            "if.true" => {
                if ignore_ctr > 0 { continue; }

                let true_branch = parse_true_branch(&tokens, i + 1)?;
                let i = find_matching_else(tokens, i + 1);
                let false_branch = parse_false_branch(&tokens, i + 1)?;

                let mut graph = ExecutionGraph::new(segment_ops);
                graph.set_next(true_branch, false_branch);
                return Ok(graph);
            },
            "else" => {
                ignore_ctr += 1;
            },
            "endif" => {
                if ignore_ctr > 0 {
                    ignore_ctr -= 1;
                }
            }
            token => {
                if ignore_ctr > 0 { continue; }
                parse_op_token(token, &mut segment_ops, i)?;
            }
        }
    }

    // TODO: check for errors
    return Ok(ExecutionGraph::new(segment_ops));
}

fn find_matching_else(tokens: &[&str], start: usize) -> usize {

    let mut depth = 1;

    for i in start..tokens.len() {
        match tokens[i] {
            "if.true" => {
                depth += 1;
            }
            "else" => {
                depth -= 1;
                if depth == 0 { return i; }
            }
            _ => { }
        }
    }

    // TODO: convert into error return
    panic!("matching else not found");
}

fn parse_op_token(token: &str, program: &mut Vec<u128>, step: usize) -> Result<bool, AssemblyError> {

    let op: Vec<&str> = token.split(".").collect();

    match op[0] {
        "noop"   => parse_noop(program, &op, step),
        "assert" => parse_assert(program, &op, step),

        "push"   => parse_push(program, &op, step),
        "read"   => parse_read(program, &op, step),

        "dup"    => parse_dup(program, &op, step),
        "pad"    => parse_pad(program, &op, step),
        "pick"   => parse_pick(program, &op, step),
        "drop"   => parse_drop(program, &op, step),
        "swap"   => parse_swap(program, &op, step),
        "roll"   => parse_roll(program, &op, step),

        "add"    => parse_add(program, &op, step),
        "sub"    => parse_sub(program, &op, step),
        "mul"    => parse_mul(program, &op, step),
        "div"    => parse_div(program, &op, step),
        "neg"    => parse_neg(program, &op, step),
        "inv"    => parse_inv(program, &op, step),
        "not"    => parse_not(program, &op, step),

        "eq"     => parse_eq(program, &op, step),
        "gt"     => parse_gt(program, &op, step),
        "lt"     => parse_lt(program, &op, step),
        "rc"     => parse_rc(program, &op, step),
        "cmp"    => parse_cmp(program, &op, step),
        "binacc" => parse_binacc(program, &op, step),

        "choose" => parse_choose(program, &op, step),

        "hash"   => parse_hash(program, &op, step),
        "mpath"  => parse_mpath(program, &op, step),

        _ => return Err(AssemblyError::invalid_op(&op, step))
    }?;

    return Ok(true);
}