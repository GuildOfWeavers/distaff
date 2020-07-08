use std::collections::HashMap;
use crate::{ opcodes };
use super::{ Program, ProgramBlock, Span, Group, Switch, Loop, ExecutionHint, BASE_CYCLE_LENGTH };

mod parsers;
use parsers::*;

mod errors;
use errors::{ AssemblyError };

#[cfg(test)]
mod tests;

type HintMap = HashMap<usize, ExecutionHint>;

// ASSEMBLY COMPILER
// ================================================================================================

pub fn compile(source: &str) -> Result<Program, AssemblyError> {

    // break assembly string into tokens
    let mut tokens: Vec<&str> = source.split_whitespace().collect();

    // perform basic validation
    if tokens.len() == 0 {
        return Err(AssemblyError::empty_program());
    }
    else if tokens[0] != "begin" {
        return Err(AssemblyError::invalid_program_start(tokens[0]));
    }
    else if tokens[tokens.len() - 1] != "end" {
        return Err(AssemblyError::invalid_program_end(tokens[tokens.len() - 1]));
    }

    // replace `begin` with `block` for parsing purposes
    tokens[0] = "block";

    // parse the token stream into program body
    let mut body = Vec::new();
    let last_step = parse_branch(&mut body, &tokens, 0)?;

    // make sure all tokens were consumed
    if last_step < tokens.len() - 1 {
        return Err(AssemblyError::unconsumed_tokens(last_step));
    }

    // build and return the program
    return Ok(Program::new(body));
}

// HELPER FUNCTIONS
// ================================================================================================

/// Parses a single program block from the `token` stream, and appends this block to the `parent`
/// list of blocks.
fn parse_block(parent: &mut Vec<ProgramBlock>, tokens: &[&str], mut i: usize) -> Result<usize, AssemblyError> {

    // read the block header
    let head: Vec<&str> = tokens[i].split(".").collect();

    // based on the block header, figure out what type of a block we are dealing with
    match head[0] {
        "block" => {
            // make sure block head instruction is valid
            if head.len() > 1 {
                return Err(AssemblyError::invalid_block_head(&head, i));
            }
            // then parse the body of the block, add the new block to the parent, and return
            let mut body = Vec::new();
            i = parse_branch(&mut body, tokens, i)?;
            parent.push(Group::new_block(body));
            return Ok(i + 1);
        },
        "if" => {
            // make sure block head is valid
            if head.len() == 1 || head[1] != "true" {
                return Err(AssemblyError::invalid_block_head(&head, i));
            }

            // parse the body of the true branch
            let mut t_branch = Vec::new();
            i = parse_branch(&mut t_branch, tokens, i)?;

            // if the false branch is present, parse it as well; otherwise
            // create an empty false branch
            let mut f_branch = Vec::new();
            if tokens[i] == "else" {
                i = parse_branch(&mut f_branch, tokens, i)?;
            }
            else {
                f_branch.push(Span::new_block(vec![
                    opcodes::NOT,  opcodes::ASSERT, opcodes::NOOP, opcodes::NOOP,
                    opcodes::NOOP, opcodes::NOOP,   opcodes::NOOP, opcodes::NOOP,
                    opcodes::NOOP, opcodes::NOOP,   opcodes::NOOP, opcodes::NOOP,
                    opcodes::NOOP, opcodes::NOOP,   opcodes::NOOP, opcodes::NOOP,
                ]));
            }

            // create a Switch block, add it to the parent, and return
            parent.push(Switch::new_block(t_branch, f_branch));
            return Ok(i + 1);
        },
        "while" => {
            // make sure block head is valid
            if head.len() == 1 || head[1] != "true" {
                return Err(AssemblyError::invalid_block_head(&head, i));
            }
            // then parse the body of the block, add the new block to the parent, and return
            let mut body = Vec::new();
            i = parse_branch(&mut body, tokens, i)?;
            parent.push(Loop::new_block(body));
            return Ok(i + 1);
        },
        _ => return Err(AssemblyError::invalid_block_head(&head, i)),
    }
}

/// Builds a body of a program block by parsing tokens from the stream and transforming
/// them into program blocks.
fn parse_branch(body: &mut Vec<ProgramBlock>, tokens: &[&str], mut i: usize) -> Result<usize, AssemblyError> {

    // determine starting instructions of the branch based on branch head
    let head: Vec<&str> = tokens[i].split(".").collect();
    let mut op_codes: Vec<u8> = match head[0] {
        "block" => vec![],
        "if"    => vec![opcodes::ASSERT],
        "else"  => vec![opcodes::NOT, opcodes::ASSERT],
        "while" => vec![opcodes::ASSERT],
        _ => return Err(AssemblyError::invalid_block_head(&head, i)),
    };
    let mut op_hints: HintMap = HashMap::new();

    // save first step to check for empty branches
    let first_step = i;
    i += 1;

    // iterate over tokens and parse them one by one until the end of the block is reached;
    // if a new block is encountered, parse it recursively
    while i < tokens.len() {
        let op: Vec<&str> = tokens[i].split(".").collect();
        i = match op[0] {
            "block" | "if" | "while" => {
                add_span(body, &mut op_codes, &mut op_hints);
                parse_block(body, tokens, i)?
            },
            "else" => {
                if head[0] != "if" {
                    return Err(AssemblyError::dangling_else(i));
                }
                else if i - first_step < 2 {
                    return Err(AssemblyError::empty_block(&head, first_step));
                }
                add_span(body, &mut op_codes, &mut op_hints);
                return Ok(i);
            },
            "end" => {
                if i - first_step < 2 {
                    return Err(AssemblyError::empty_block(&head, first_step));
                }
                add_span(body, &mut op_codes, &mut op_hints);
                return Ok(i);
            },
            _ => parse_op_token(op, &mut op_codes, &mut op_hints, i)?
        };
    }

    // if all tokens were consumed by block end was not found, return an error
    return match head[0] {
        "block" => Err(AssemblyError::unmatched_block(first_step)),
        "if"    => Err(AssemblyError::unmatched_if(first_step)),
        "else"  => Err(AssemblyError::unmatched_else(first_step)),
        "while" => Err(AssemblyError::unmatched_while(first_step)),
        _ => Err(AssemblyError::invalid_block_head(&head, first_step)),
    };
}

/// Adds a new Span block to a program block body based on currently parsed instructions.
fn add_span(body: &mut Vec<ProgramBlock>, op_codes: &mut Vec<u8>, op_hints: &mut HintMap) {

    // if there were no instructions in the current span, don't do anything
    if op_codes.len() == 0 { return };

    // pad the instructions to make ensure 16-cycle alignment
    let mut span_op_codes = op_codes.clone();
    let pad_length = BASE_CYCLE_LENGTH - (span_op_codes.len() % BASE_CYCLE_LENGTH) - 1;
    span_op_codes.resize(span_op_codes.len() + pad_length, opcodes::NOOP);

    // add a new Span block to the body
    body.push(ProgramBlock::Span(Span::new(span_op_codes, op_hints.clone())));

    // clear op_codes and op_hints for the next Span block
    op_codes.clear();
    op_hints.clear();
}

/// Transforms an assembly instruction into a sequence of one or more VM instructions.
fn parse_op_token(op: Vec<&str>, op_codes: &mut Vec<u8>, op_hints: &mut HintMap, step: usize) -> Result<usize, AssemblyError> {

    // based on the instruction, invoke the correct parser for the operation
    match op[0] {
        "noop"   => parse_noop(op_codes, &op, step),
        "assert" => parse_assert(op_codes, &op, step),

        "push"   => parse_push(op_codes, op_hints, &op, step),
        "read"   => parse_read(op_codes, &op, step),

        "dup"    => parse_dup(op_codes, &op, step),
        "pad"    => parse_pad(op_codes, &op, step),
        "pick"   => parse_pick(op_codes, &op, step),
        "drop"   => parse_drop(op_codes, &op, step),
        "swap"   => parse_swap(op_codes, &op, step),
        "roll"   => parse_roll(op_codes, &op, step),

        "add"    => parse_add(op_codes, &op, step),
        "sub"    => parse_sub(op_codes, &op, step),
        "mul"    => parse_mul(op_codes, &op, step),
        "div"    => parse_div(op_codes, &op, step),
        "neg"    => parse_neg(op_codes, &op, step),
        "inv"    => parse_inv(op_codes, &op, step),
        "not"    => parse_not(op_codes, &op, step),
        "and"    => parse_and(op_codes, &op, step),
        "or"     => parse_or(op_codes, &op, step),

        "eq"     => parse_eq(op_codes, op_hints, &op, step),
        "gt"     => parse_gt(op_codes, op_hints, &op, step),
        "lt"     => parse_lt(op_codes, op_hints, &op, step),
        "rc"     => parse_rc(op_codes, op_hints, &op, step),
        "isodd"  => parse_isodd(op_codes, op_hints, &op, step),

        "choose" => parse_choose(op_codes, &op, step),

        "hash"   => parse_hash(op_codes, &op, step),
        "mpath"  => parse_mpath(op_codes, &op, step),

        _ => return Err(AssemblyError::invalid_op(&op, step))
    }?;

    // advance instruction pointer to the next step
    return Ok(step + 1);
}