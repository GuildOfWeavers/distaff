use crate::crypto::{ HashFunction };
use crate::processor::{ opcodes::f128 as opcodes };
use super::{ Program, ExecutionGraph };

mod parsers;
use parsers::*;

mod errors;
use errors::{ AssemblyError };

#[cfg(test)]
mod tests;

// ASSEMBLER
// ================================================================================================

/// Compiles provided assembly code into a program.
pub fn compile(source: &str, hash_fn: HashFunction) -> Result<Program, AssemblyError> {
    
    // all programs must start with BEGIN operation
    let mut segment_ops = vec![opcodes::BEGIN];

    // break assembly string into tokens
    let tokens: Vec<&str> = source.split_whitespace().collect();

    // iterate over tokens and parse them one by one until the first branch is encountered
    for i in 0..tokens.len() {
        match tokens[i] {
            "if.true" => {
                // when `if` token is encountered, recursively parse true and false branches,
                // combine them into an execution graph, construct a program, and return
                let true_branch = parse_branch(&tokens, i, 1)?;
                let i = find_matching_else(&tokens, i)?;
                let false_branch = parse_branch(&tokens, i, 1)?;

                let mut exe_graph = ExecutionGraph::new(segment_ops);
                exe_graph.set_next(true_branch, false_branch);
                return Ok(Program::new(exe_graph, hash_fn));
            },
            "else"  => return Err(AssemblyError::unmatched_else(i)),
            "endif" => return Err(AssemblyError::unmatched_endif(i)),
            token   => parse_op_token(token, &mut segment_ops, i)
        }?;
    }

    // if there are no branches, make sure there was at least one operation,
    // build the program, and return
    if segment_ops.len() <= 1 {
        return Err(AssemblyError::empty_program());
    }
    else {
        return Ok(Program::from_path(segment_ops));
    }
}

// HELPER FUNCTIONS
// ================================================================================================
fn parse_branch(tokens: &[&str], mut i: usize, mut depth: usize) -> Result<ExecutionGraph, AssemblyError> {

    // get the first instruction of the branch and advance instruction counter
    let first_op = tokens[i];
    i += 1;

    // true branch must start with ASSERT, which false branch must start with NOT ASSERT
    let branch_head_length: usize;
    let mut segment_ops = if first_op == "if.true" {
        branch_head_length = 1;
        vec![opcodes::ASSERT]
    }
    else {
        branch_head_length = 2;
        vec![opcodes::NOT, opcodes::ASSERT]
    };

    // iterate over tokens and parse them one by one until the next branch is encountered
    while i < tokens.len() {
        match tokens[i] {
            "if.true" => {
                // make sure the branch was not empty
                if segment_ops.len() <= branch_head_length {
                    return Err(AssemblyError::empty_branch(first_op, i));
                }

                // parse subsequent true and false branches
                let true_branch = parse_branch(&tokens, i, depth + 1)?;
                let i = find_matching_else(tokens, i)?;
                let false_branch = parse_branch(&tokens, i, depth + 1)?;

                // build the graph and return
                let mut graph = ExecutionGraph::new(segment_ops);
                graph.set_next(true_branch, false_branch);
                return Ok(graph);
            },
            "else" => {
                i = find_matching_endif(&tokens, i)?;
            },
            "endif" => {
                // make sure branches are closed correctly
                if depth == 0 {
                    return Err(AssemblyError::unmatched_endif(i));
                }
                depth -= 1;
            }
            token => {
                parse_op_token(token, &mut segment_ops, i)?;
            }
        }

        i += 1;
    }

    // if there were no further branches, make sure the branch was not empty, and return
    if segment_ops.len() <= branch_head_length {
        return Err(AssemblyError::empty_branch(first_op, i));
    }
    else {
        return Ok(ExecutionGraph::new(segment_ops));
    }
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

fn find_matching_else(tokens: &[&str], start: usize) -> Result<usize, AssemblyError> {

    let mut if_ctr = 0;
    let mut else_ctr = 0;
    let mut end_ctr = 0;

    for i in start..tokens.len() {
        match tokens[i] {
            "if.true" => {
                if_ctr += 1;
            },
            "else" => {
                else_ctr += 1;
                if else_ctr > if_ctr {
                    return Err(AssemblyError::unmatched_else(i));
                }
                else if if_ctr == else_ctr {
                    return Ok(i);
                }
            },
            "endif" => {
                end_ctr += 1;
                if end_ctr > if_ctr {
                    return Err(AssemblyError::unmatched_endif(i));
                }
                else if end_ctr > else_ctr {
                    return Err(AssemblyError::missing_else(i));
                }
            }
            _ => ()
        }
    }

    return Err(AssemblyError::dangling_if(start));
}

fn find_matching_endif(tokens: &[&str], start: usize) -> Result<usize, AssemblyError> {

    let mut if_ctr = 1;
    let mut else_ctr = 0;
    let mut end_ctr = 0;

    for i in start..tokens.len() {
        match tokens[i] {
            "if.true" => {
                if_ctr += 1;
            },
            "else" => {
                else_ctr += 1;
                if else_ctr > if_ctr {
                    return Err(AssemblyError::unmatched_else(i));
                }
            },
            "endif" => {
                end_ctr += 1;
                if end_ctr > if_ctr {
                    return Err(AssemblyError::unmatched_endif(i));
                }
                else if end_ctr > else_ctr {
                    return Err(AssemblyError::missing_else(i));
                }
                else if end_ctr == else_ctr {
                    return Ok(i);
                }
            },
            _ => ()
        }
    }

    return Err(AssemblyError::dangling_else(start));
}