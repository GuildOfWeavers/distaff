use std::fmt;
use std::cmp;
use crate::math::{ FiniteField };
use super::{ decoder, NUM_LD_OPS, MIN_STACK_DEPTH };

// CONSTANTS
// ================================================================================================
const NUM_STATIC_REGISTER: usize = decoder::NUM_REGISTERS;

// TYPES AND INTERFACES
// ================================================================================================
#[derive(Debug, PartialEq)]
pub struct TraceState<T>
    where T: FiniteField
{
    registers       : Vec<T>,
    state_width     : usize,
    op_flags        : [T; NUM_LD_OPS],
    op_flags_set    : bool,
}

// TRACE STATE IMPLEMENTATION
// ================================================================================================
impl <T> TraceState<T>
    where T: FiniteField
{
    pub fn new(stack_depth: usize) -> TraceState<T> {
        let state_width = NUM_STATIC_REGISTER + stack_depth;
        let num_registers = NUM_STATIC_REGISTER + cmp::max(stack_depth, MIN_STACK_DEPTH);
        
        return TraceState {
            registers   : vec![T::ZERO; num_registers],
            state_width : state_width,
            op_flags    : [T::ZERO; NUM_LD_OPS],
            op_flags_set: false
        };
    }

    pub fn from_raw_state(mut state: Vec<T>) -> TraceState<T> {
        let state_width = state.len();
        let stack_depth = state_width - NUM_STATIC_REGISTER;

        if stack_depth < MIN_STACK_DEPTH {
            state.resize(state.len() + (MIN_STACK_DEPTH - stack_depth), T::ZERO);
        }

        return TraceState {
            registers   : state,
            state_width : state_width,
            op_flags    : [T::ZERO; NUM_LD_OPS],
            op_flags_set: false
        };
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------
    pub fn get_op_code(&self) -> T {
        return self.registers[decoder::OP_CODE_IDX];
    }

    pub fn get_push_flag(&self) -> T {
        return self.registers[decoder::PUSH_FLAG_IDX];
    }

    pub fn get_op_acc(&self) -> &[T] {
        return &self.registers[decoder::OP_ACC_RANGE];
    }

    pub fn get_program_hash(&self) -> &[T] {
        return &self.registers[decoder::PROG_HASH_RANGE];
    }

    pub fn get_op_bits(&self) -> &[T] {
        return &self.registers[decoder::OP_BITS_RANGE];
    }

    pub fn get_op_bits_value(&self) -> T {
        let op_bits = self.get_op_bits();
        let mut value = op_bits[0];
        value = T::add(value, T::mul(op_bits[1], T::from( 2)));
        value = T::add(value, T::mul(op_bits[2], T::from( 4)));
        value = T::add(value, T::mul(op_bits[3], T::from( 8)));
        value = T::add(value, T::mul(op_bits[4], T::from(16)));
        return value;
    }

    pub fn get_op_flags(&self) -> [T; NUM_LD_OPS] {
        if !self.op_flags_set {
            unsafe {
                let mutable_self = &mut *(self as *const _ as *mut TraceState<T>);
                mutable_self.set_op_flags();
                mutable_self.op_flags_set = true;
            }
        }
        return self.op_flags;
    }

    pub fn get_stack(&self) -> &[T] {
        return &self.registers[NUM_STATIC_REGISTER..];
    }

    pub fn compute_stack_depth(trace_register_count: usize) -> usize {
        return trace_register_count - NUM_STATIC_REGISTER;
    }

    // RAW STATE
    // --------------------------------------------------------------------------------------------
    pub fn registers(&self) -> &[T] {
        return &self.registers[..self.state_width];
    }

    pub fn set_register(&mut self, index: usize, value: T) {
        self.registers[index] = value;
        self.op_flags_set = false;
    }

    // HELPER METHODS
    // --------------------------------------------------------------------------------------------
    fn set_op_flags(&mut self) {
        // TODO: needs to be optimized

        // initialize op_flags to 1
        let mut op_flags = [T::ONE; NUM_LD_OPS];

        // expand the bits
        let op_bits = self.get_op_bits();
        for i in 0..5 {
            
            let segment_length = usize::pow(2, (i + 1) as u32);

            let inv_bit = T::sub(T::ONE, op_bits[i]);
            for j in 0..(segment_length / 2) {
                op_flags[j] = T::mul(op_flags[j], inv_bit);
            }

            for j in (segment_length / 2)..segment_length {
                op_flags[j] = T::mul(op_flags[j], op_bits[i]);
            }

            for j in (segment_length..NUM_LD_OPS).step_by(segment_length) {
                op_flags.copy_within(0..segment_length, j);
            }
        }

        self.op_flags = op_flags;
    }
}

impl <T> fmt::Display for TraceState<T>
    where T: FiniteField
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]\t[{}]\t{:?}\t{:?}",
            self.get_op_code(), 
            self.get_push_flag(),
            self.get_op_bits(),
            self.get_stack())
    }
}