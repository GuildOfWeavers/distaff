
mod opcodes;
use opcodes::{ FlowOps, UserOps };

const SPONGE_WIDTH: usize = 4;
const NUM_CF_OP_BITS: usize = 3;
const NUM_LD_OP_BITS: usize = 5;
const NUM_HD_OP_BITS: usize = 2;


pub struct Decoder {

    op_acc      : [Vec<u128>; SPONGE_WIDTH],
    sponge      : [u128; SPONGE_WIDTH],

    cf_op_bits  : [Vec<u128>; NUM_CF_OP_BITS],
    ld_op_bits  : [Vec<u128>; NUM_LD_OP_BITS],
    hd_op_bits  : [Vec<u128>; NUM_HD_OP_BITS],

    ctx_stack   : Vec<Vec<u128>>,
    loop_stack  : Vec<Vec<u128>>,
}

impl Decoder {

    pub fn new(init_trace_length: usize) -> Decoder {

        // initialize instruction accumulator
        let op_acc = [
            vec![0; init_trace_length], vec![0; init_trace_length],
            vec![0; init_trace_length], vec![0; init_trace_length]
        ];
        let sponge = [0; SPONGE_WIDTH];

        // initialize op_bits registers
        let cf_op_bits = [
            vec![0; init_trace_length], vec![0; init_trace_length], vec![0; init_trace_length]
        ];
        let ld_op_bits = [
            vec![0; init_trace_length], vec![0; init_trace_length], vec![0; init_trace_length],
            vec![0; init_trace_length], vec![0; init_trace_length]
        ];
        let hd_op_bits = [
            vec![0; init_trace_length], vec![0; init_trace_length]
        ];

        // initialize stacks
        let ctx_stack = vec![vec![0; init_trace_length]];
        let loop_stack = Vec::new();
        
        // TODO: initialize first step

        return Decoder {
            op_acc, sponge, cf_op_bits, ld_op_bits, hd_op_bits, ctx_stack, loop_stack,
        };
    }

    pub fn start_block(&mut self) {
        
        // TODO: make sure happens on one less than multiple of 16
        self.set_op_bits(FlowOps::Begin, UserOps::Noop, 0);
        self.save_context(self.sponge[0]);
        self.set_sponge([0, 0, 0, 0]);
    }

    pub fn end_block(&mut self, sibling_hash: u128, true_branch: bool) {
        // TODO: make sure this happens on a step which is a multiple of 16

        let h = self.pop_context();
        if true_branch {
            self.set_op_bits(FlowOps::Tend, UserOps::Noop, 0);
            self.set_sponge([h, self.sponge[0], sibling_hash, 0]);
        }
        else {
            self.set_op_bits(FlowOps::Fend, UserOps::Noop, 0);
            self.set_sponge([h, sibling_hash, self.sponge[0], 0]);
        }
    }

    pub fn start_loop(&mut self, loop_image: u128) {
        // TODO: make sure happens on one less than multiple of 16
        self.set_op_bits(FlowOps::Loop, UserOps::Noop, 0);
        self.save_context(self.sponge[0]);
        self.save_loop_image(loop_image);
        self.set_sponge([0, 0, 0, 0]);
    }

    pub fn wrap_loop(&mut self) {
        // TODO: make sure happens on one less than multiple of 16
        self.set_op_bits(FlowOps::Wrap, UserOps::Noop, 0);
        let loop_image = self.peek_loop_image();
        assert!(loop_image == self.sponge[0], "TODO");
        self.set_sponge([0, 0, 0, 0]);
    }

    pub fn break_loop(&mut self) {
        // TODO: make sure happens on one less than multiple of 16
        self.set_op_bits(FlowOps::Break, UserOps::Noop, 0);
        let loop_image = self.pop_loop_image();
        assert!(loop_image == self.sponge[0], "TODO");
    }

    pub fn decode_op(&mut self, op_code: UserOps, op_value: u128) {
        // TODO: if op_value != 0, make sure this happens on a step which is a multiple of 8
        self.set_op_bits(FlowOps::Hacc, op_code, 0);
        self.apply_hacc_round(op_code, op_value)
    }

    pub fn noop(&mut self) {
        self.set_op_bits(FlowOps::Void, UserOps::Noop, 0);
        // TODO: copy contexts
        // TODO: copy sponge
    }

    // HELPER METHODS
    // --------------------------------------------------------------------------------------------

    fn set_op_bits(&mut self, flow_op: FlowOps, user_op: UserOps, step: usize) {
        let flow_op = flow_op as u8;
        for i in 0..NUM_CF_OP_BITS {
            self.cf_op_bits[i][step] = ((flow_op >> i) & 1) as u128;
        }

        let user_op = user_op as u8;
        for i in 0..NUM_LD_OP_BITS {
            self.ld_op_bits[i][step] = ((user_op >> i) & 1) as u128;
        }

        for i in 0..NUM_HD_OP_BITS {
            self.hd_op_bits[i][step] = ((user_op >> (i + NUM_LD_OP_BITS)) & 1) as u128;
        }
    }

    fn save_context(&mut self, parent_hash: u128) {
        // TODO: implement
    }

    fn pop_context(&mut self) -> u128 {
        // TODO: implement
        return 0;
    }

    fn save_loop_image(&mut self, loop_image: u128) {
        // TODO: implement
    }

    fn peek_loop_image(&self) -> u128 {
        // TODO: implement
        return 0;
    }

    fn pop_loop_image(&mut self) -> u128 {
        // TODO: implement
        return 0;
    }

    fn set_sponge(&mut self, state: [u128; SPONGE_WIDTH]) {
        let step = 0; // TODO
        self.op_acc[0][step + 1] = state[0];
        self.op_acc[1][step + 1] = state[1];
        self.op_acc[2][step + 1] = state[2];
        self.op_acc[3][step + 1] = state[3];
        self.sponge = state;
    }

    fn apply_hacc_round(&mut self, op_code: UserOps, op_value: u128) {
        // TODO: implement
    }
}