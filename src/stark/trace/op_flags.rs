use crate::{
    math::field,
    processor::opcodes::{ FlowOps, UserOps },
    NUM_FLOW_OP_BITS, NUM_FLOW_OPS,
    NUM_USER_OP_BITS, NUM_USER_OPS
};

// TYPES AND INTERFACES
// ================================================================================================

pub struct OpFlags {
    flow_ops    : [u128; NUM_FLOW_OPS],
    user_ops    : [u128; NUM_USER_OPS],
}

// OP FLAGS IMPLEMENTATION
// ================================================================================================

impl OpFlags {

    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    pub fn new() -> OpFlags {
        return OpFlags {
            flow_ops    : [0; NUM_FLOW_OPS],
            user_ops    : [0; NUM_USER_OPS]
        };
    }

    // FLAG GETTERS
    // --------------------------------------------------------------------------------------------

    pub fn get_flow_op_flag(&self, op: FlowOps) -> u128 {
        return match op {
            FlowOps::Hacc       => self.flow_ops[0],
            FlowOps::Begin      => self.flow_ops[1],
            FlowOps::Tend       => self.flow_ops[2],
            FlowOps::Fend       => self.flow_ops[3],
            FlowOps::Loop       => self.flow_ops[4],
            FlowOps::Wrap       => self.flow_ops[5],
            FlowOps::Break      => self.flow_ops[6],
            FlowOps::Void       => self.flow_ops[7],
        };
    }

    pub fn get_user_op_flag(&self, op: UserOps) -> u128 {
        return match op {
            UserOps::Noop       => self.user_ops[0],
            UserOps::Assert     => self.user_ops[1],
            UserOps::AssertEq   => self.user_ops[2],
            UserOps::Drop       => self.user_ops[3],
            UserOps::Drop4      => self.user_ops[4],
            UserOps::Pad2       => self.user_ops[5],
            UserOps::Dup        => self.user_ops[6],
            UserOps::Dup2       => self.user_ops[7],
            UserOps::Dup4       => self.user_ops[8],
            UserOps::Read       => self.user_ops[9],
            UserOps::Read2      => self.user_ops[10],
            UserOps::Swap       => self.user_ops[11],
            UserOps::Swap2      => self.user_ops[12],
            UserOps::Swap4      => self.user_ops[13],
            UserOps::Roll4      => self.user_ops[14],
            UserOps::Roll8      => self.user_ops[15],

            UserOps::Eq         => self.user_ops[16],
            UserOps::Choose     => self.user_ops[17],
            UserOps::Choose2    => self.user_ops[18],
            UserOps::CSwap2     => self.user_ops[19],
            UserOps::Add        => self.user_ops[20],
            UserOps::Mul        => self.user_ops[21],
            UserOps::And        => self.user_ops[22],
            UserOps::Or         => self.user_ops[23],
            UserOps::Inv        => self.user_ops[24],
            UserOps::Neg        => self.user_ops[25],
            UserOps::Not        => self.user_ops[26],
            UserOps::BinAcc     => self.user_ops[27],
            UserOps::MLoad      => self.user_ops[28],
            UserOps::MStore     => self.user_ops[29],
            UserOps::Future1    => self.user_ops[30],

            UserOps::Push       => self.user_ops[31],
            UserOps::Cmp        => self.user_ops[32],
            UserOps::RescR      => self.user_ops[33],
            UserOps::MemRR      => self.user_ops[34],

            UserOps::Begin      => self.user_ops[35]
        };
    }

    // FLAG SETTER
    // --------------------------------------------------------------------------------------------

    pub fn update(&mut self, flow_op_bits: &[u128; NUM_FLOW_OP_BITS], user_op_bits: &[u128; NUM_USER_OP_BITS]) {

        // 1 ----- compute control flow op flags --------------------------------------------------

        let not_0 = binary_not(flow_op_bits[0]);
        let not_1 = binary_not(flow_op_bits[1]);
        self.flow_ops[0] = field::mul(not_0, not_1);
        self.flow_ops[1] = field::mul(flow_op_bits[0], not_1);
        self.flow_ops[2] = field::mul(not_0, flow_op_bits[1]);
        self.flow_ops[3] = field::mul(flow_op_bits[0], flow_op_bits[1]);
        self.flow_ops.copy_within(0..4, 4);

        let not_2 = binary_not(flow_op_bits[2]);
        for i in 0..4 { self.flow_ops[i] = field::mul(self.flow_ops[i], not_2); }
        for i in 4..8 { self.flow_ops[i] = field::mul(self.flow_ops[i], flow_op_bits[2]); }

        // 2 ----- compute user op flags ----------------------------------------------------------

        let not_0 = binary_not(user_op_bits[0]);
        let not_1 = binary_not(user_op_bits[1]);
        self.user_ops[0] = field::mul(not_0, not_1);
        self.user_ops[1] = field::mul(user_op_bits[0], not_1);
        self.user_ops[2] = field::mul(not_0, user_op_bits[1]);
        self.user_ops[3] = field::mul(user_op_bits[0], user_op_bits[1]);
        self.user_ops.copy_within(0..4, 4);

        let not_2 = binary_not(user_op_bits[2]);
        for i in 0..4 { self.user_ops[i] = field::mul(self.user_ops[i], not_2); }
        for i in 4..8 { self.user_ops[i] = field::mul(self.user_ops[i], user_op_bits[2]); }
        self.user_ops.copy_within(0..8, 8);

        let not_3 = binary_not(user_op_bits[3]);
        for i in 0..8  { self.user_ops[i] = field::mul(self.user_ops[i], not_3); }
        for i in 8..16 { self.user_ops[i] = field::mul(self.user_ops[i], user_op_bits[3]); }

        self.user_ops.copy_within(0..15, 16);

        // all op_bits are 1's: for begin op only
        let class_1 = field::mul(user_op_bits[4], user_op_bits[5]);
        self.user_ops[35] = field::mul(self.user_ops[15], class_1); // begin

        // class 2 ops: can be degree 5 or smaller
        let not_4 = binary_not(user_op_bits[4]);
        let class_2 = field::mul(not_4, user_op_bits[5]);
        self.user_ops[31] = field::mul(user_op_bits[0], class_2);   // push
        self.user_ops[32] = field::mul(user_op_bits[1], class_2);   // cmp
        self.user_ops[33] = field::mul(user_op_bits[2], class_2);   // rescr
        self.user_ops[34] = field::mul(user_op_bits[3], class_2);   // memrr
        
        // class 3 ops: can be degree 2 or smaller
        let not_5 = binary_not(user_op_bits[5]);
        let class_3 = field::mul(user_op_bits[4], not_5);
        for i in 16..31 { self.user_ops[i] = field::mul(self.user_ops[i], class_3); }

        // class 4 ops: must be degree 1
        let class_4 = field::mul(not_4, not_5);
        for i in 0..16 { self.user_ops[i] = field::mul(self.user_ops[i], class_4); }
    }
}

impl PartialEq for OpFlags {
    fn eq(&self, other: &Self) -> bool {
        return self.flow_ops == other.flow_ops
            && self.user_ops.to_vec() == other.user_ops.to_vec();
    }
}

// HELPER FUNCTIONS
// ================================================================================================
#[inline(always)]
fn binary_not(v: u128) -> u128 {
    return field::sub(field::ONE, v);
}

// TESTS
// ================================================================================================
#[cfg(test)]
mod tests {

    use std::convert::TryFrom;
    use super::{ NUM_FLOW_OP_BITS, NUM_USER_OP_BITS };
    
    #[test]
    fn flow_ops() {
        let mut flags = super::OpFlags::new();
        for i in 0..8 {
            match super::UserOps::try_from(i) {
                Ok(_) => {
                    let mut flow_op_bits = [0; NUM_FLOW_OP_BITS];
                    for j in 0..NUM_FLOW_OP_BITS {
                        flow_op_bits[j] = ((i >> j) & 1) as u128;
                    }

                    flags.update(&flow_op_bits, &[0, 0, 0, 0, 0, 0]);

                    let mut flag_count = 0;
                    for flag in flags.flow_ops.to_vec() {
                        if flag == 1 { flag_count += 1; }
                    }

                    // one and only one flag can be set for each valid operation
                    assert_eq!(flag_count, 1);
                },
                Err(_) => ()
            }
        }
    }

    #[test]
    fn user_ops() {
        let mut flags = super::OpFlags::new();
        for i in 0..64 {
            match super::UserOps::try_from(i) {
                Ok(_) => {
                    let mut user_op_bits = [0; NUM_USER_OP_BITS];
                    for j in 0..NUM_USER_OP_BITS {
                        user_op_bits[j] = ((i >> j) & 1) as u128;
                    }

                    flags.update(&[0, 0, 0], &user_op_bits);

                    let mut flag_count = 0;
                    for flag in flags.user_ops.to_vec() {
                        if flag == 1 { flag_count += 1; }
                    }

                    // one and only one flag can be set for each valid operation
                    assert_eq!(flag_count, 1);
                },
                Err(_) => ()
            }
        }
    }

}