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

pub fn compile(source: &str) -> Program {

    // break assembly string into tokens
    let mut tokens: Vec<&str> = source.split_whitespace().collect();

    assert!(tokens[0] == "begin", "TODO: not begin");
    tokens[0] = "block";

    let mut body = Vec::new();
    parse_branch(&mut body, &tokens, 0);

    return Program::new(body);
}

fn parse_block(parent: &mut Vec<ProgramBlock>, tokens: &[&str], mut i: usize) -> usize {

    let head: Vec<&str> = tokens[i].split(".").collect();

    match head[0] {
        "block" => {
            let mut body = Vec::new();
            i = parse_branch(&mut body, tokens, i);
            parent.push(Group::new_block(body));
            return i + 1;
        },
        "if" => {
            let mut t_branch = Vec::new();
            i = parse_branch(&mut t_branch, tokens, i);

            let mut f_branch = Vec::new();
            if tokens[i] == "else" {
                i = parse_branch(&mut f_branch, tokens, i);
            }
            else {
                f_branch.push(Span::new_block(vec![
                    opcodes::NOT,  opcodes::ASSERT, opcodes::NOOP, opcodes::NOOP,
                    opcodes::NOOP, opcodes::NOOP,   opcodes::NOOP, opcodes::NOOP,
                    opcodes::NOOP, opcodes::NOOP,   opcodes::NOOP, opcodes::NOOP,
                    opcodes::NOOP, opcodes::NOOP,   opcodes::NOOP, opcodes::NOOP,
                ]));
            }

            parent.push(Switch::new_block(t_branch, f_branch));
            return i + 1;
        },
        "while" => {
            let mut body = Vec::new();
            i = parse_branch(&mut body, tokens, i);
            parent.push(Loop::new_block(body));
            return i + 1;
        },
        _ => panic!("TODO: invalid block head")
    }
}

fn parse_branch(body: &mut Vec<ProgramBlock>, tokens: &[&str], mut i: usize) -> usize {

    // get the first instruction of the branch and advance instruction counter
    let first_op: Vec<&str> = tokens[i].split(".").collect();
    let mut op_codes: Vec<u8> = init_branch(first_op[0]);
    let mut op_hints: HintMap = HashMap::new();
    i += 1;

    // iterate over tokens and parse them one by one until the next branch is encountered
    while i < tokens.len() {
        let op: Vec<&str> = tokens[i].split(".").collect();

        i = match op[0] {
            "block" | "if" | "while" => {
                add_span(body, &mut op_codes, &mut op_hints);
                parse_block(body, tokens, i)
            },
            "else" => {
                assert!(first_op[0] == "if", "TODO");
                add_span(body, &mut op_codes, &mut op_hints);
                return i;
            },
            "end" => {
                add_span(body, &mut op_codes, &mut op_hints);
                return i;
            },
            _ => parse_op_token(op, &mut op_codes, &mut op_hints, i).unwrap()
        }
    }

    panic!("TODO: parse_branch end");
}

fn init_branch(head: &str) -> Vec<u8> {
    return match head {
        "block" => vec![],
        "if" | "while" => vec![opcodes::ASSERT],
        "else" => vec![opcodes::NOT, opcodes::ASSERT],
        _ => panic!("TODO: init branch")
    };
}

fn add_span(blocks: &mut Vec<ProgramBlock>, op_codes: &mut Vec<u8>, op_hints: &mut HintMap) {

    let mut span_op_codes = op_codes.clone();
    
    let pad_length = BASE_CYCLE_LENGTH - (span_op_codes.len() % BASE_CYCLE_LENGTH) - 1;
    span_op_codes.resize(span_op_codes.len() + pad_length, opcodes::NOOP);

    blocks.push(ProgramBlock::Span(Span::new(span_op_codes, op_hints.clone())));

    op_codes.clear();
    op_hints.clear();
}

fn parse_op_token(op: Vec<&str>, op_codes: &mut Vec<u8>, op_hints: &mut HintMap, step: usize) -> Result<usize, AssemblyError> {

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

    return Ok(step + 1);
}