# Distaff assembly
Distaff assembly is a simple, low-level language for writing programs for Distaff VM. It stand just above raw Distaff VM [instructions](isa.md), and in fact, many instructions in Distaff assembly map directly to instruction of Distaff VM. However, Distaff assembly has several advantages:

* Distaff assembly supports *macro instructions*. These instructions expand into a sequence of raw Distaff VM instructions making it easier to encode common tasks.
* Distaff assembler takes care of properly aligning and padding all instructions reducing the amount of mental bookkeeping needed for writing programs.
* Distaff assembly natively supports control flow expression which the assembler automatically transforms into program execution graphs needed by Distaff VM.

### Compiling assembly code
To compile Distaff assembly source code into a program for Distaff VM, `compile()` function from the [assembly](https://github.com/GuildOfWeavers/distaff/blob/master/src/programs/assembly/mod.rs) module should be used. This function takes the following parameters:

* `source: &str` - a string containing Distaff assembly source code.
* `hash_fn: HashFunction` - a hash function to be used for building a Merkle tree of program execution paths.

The function returns `Result<Program, AssemblyError>` which will contain the compiled program if the compilation was successful, or if the source code contained errors, description of the first error will be returned.

For example:
```Rust
use distaff::{ assembly, crypto::hash::blake3 };

// the program pushes values 3 and 5 onto the stack and adds them
let program = assembly::compile("push.3 push.5 add", blake3).unwrap();
```

## Assembly programs
A Distaff assembly program is just a sequence of instructions each describing a specific operation. You can use any combination of whitespace characters to separate one instruction from another.

All currently available instructions are described below. Many instructions can be parametrized with a single parameter. The notation for specifying parameters is *operation.parameter*. For example, `push.123` describes a `push` operation which is parametrized with value `123`.

For most instructions which support parameters, the default parameter is `1`. So, `dup` is equivalent to `dup.1`, `choose` is equivalent to `choose.1` and so on.

A single instruction may take multiple VM cycles to execute. The number of cycles frequently depends on the specified parameter, and sometimes depends on other factors (e.g. place of the operation in the execution path). The tables below include this number of cycles in the last column.

### Flow control instructions

| Operation | Description                            | Cycles |
| --------- | -------------------------------------- | :----: |
| noop      | Does nothing.                          | 1      |
| assert    | Pops the top item from the stack and checks if it is equal to `1`. If it is not equal to `1` the program fails. | 1 |
| if.true   | Marks the beginning of the *true* branch in the `if.true else endif` expression. If the value at the top of the stack is `1`, the *true* branch is executed. | 1 |
| else      | Marks the beginning of the *false* branch in the `if.true else endif` expression. If the value at the top of the stack is `0`, the *false* branch is executed. | 2 |
| endif     | Marks the end of the `if.true else endif` expression.  | 0 |


### Input instructions

| Operation | Description                            | Cycles |
| --------- | -------------------------------------- | :----: |
| push.*x*  | Pushes *x* onto the stack. *x* can be any valid field element. | 2 |
| read.a    | Pushes the next value from the input tape `A` onto the stack. | 1 |
| read.ab   | Pushes the next values from input tapes `A` and `B` onto the stack. Value from input tape `A` is pushed first, followed by the value from input tape `B`. | 1 |

### Stack manipulation instructions

| Operation | Description                            | Cycles |
| --------- | -------------------------------------- | :----: |
| dup.*n*   | Pushes copies of the top *n* stack items onto the stack. *n* can be any integer between 1 and 4. | 1 - 3 |
| pad.*n*   | Pushes *n* `0`'s onto the stack; *n* can be any integer between 1 and 8. | 1 - 4 |
| pick.*n*  | Pushes a copy of the item with index *n* onto the stack. For example, assuming `S0` is the top of the stack, executing `pick.2` transforms `S0 S1 S2 S3` into `S2 S0 S1 S2 S3`. *n* can be any integer between 1 and 3. | 2 - 4 |
| drop.*n*  | Removes top *n* items from the stack; *n* can be any integer between 1 and 8. | 1 - 3 |
| swap.1    | Moves the second from the top stack item to the top of the stack (swaps top two stack items). | 1 |
| swap.2    | Moves 3rd and 4th stack items to the top of the stack. For example, assuming `S0` is the top of the stack, `S0 S1 S2 S3` becomes `S2 S3 S0 S1`. | 1 |
| swap.4    | Moves 5th through 8th stack items to the top of the stack. For example, assuming `S0` is the top of the stack, `S0 S1 S2 S3 S4 S5 S6 S7` becomes `S4 S5 S6 S7 S0 S1 S2 S3`. | 1 |
| roll.4    | Moves 4th stack item to the top of the stack. For example, assuming `S0` is the top of the stack, `S0 S1 S2 S3` becomes `S3 S0 S1 S2`. | 1 |
| roll.8    | Moves 8th stack item to the top of the stack. For example, assuming `S0` is the top of the stack, `S0 S1 S2 S3 S4 S5 S6 S7` becomes `S7 S0 S1 S2 S3 S4 S5 S6`. | 1 |

### Arithmetic and boolean instructions

| Operation | Description                            | Cycles |
| --------- | -------------------------------------- | :----: |
| add       | Pops top two items from the stack, adds them, and pushes the result back onto the stack. | 1 |
| sub       | Pops top two items from the stack, subtracts the top item from the second to the top item, and pushes the result back onto the stack.  | 2 |
| mul       | Pops top two items from the stack, multiplies them, and pushes the result back onto the stack. | 1 |
| div       | Pops top two items from the stack, divides second to the top item by the top item, and pushes the result back onto the stack. If the item at the top of the stack is `0`, this operation will fail. | 2 |
| neg       | Pops the top item from the stack, computes its additive inverse, and pushes the result back onto the stack. | 1      |
| inv       | Pops the top item from the stack, computes its multiplicative inverse, and pushes the result back onto the stack. If the value at the top of the stack is `0`, this operation will fail. | 1 |
| not       | Pops the top item from the stack, subtracts it from value `1` and pushes the result back onto the stack. In other words, `0` becomes `1`, and `1` becomes `0`. If the item at the top of the stack is not binary (i.e. not `0` or `1`), this operation will fail. | 1 |

### Comparison instructions

| Operation | Description                            | Cycles |
| --------- | -------------------------------------- | :----: |
| eq        | Pops top two items from the stack, compares them, and if their values are equal, pushes `1` onto the stack; otherwise pushes `0` onto the stack. | 1 |
| gt.*n*    | Pops top two items from the stack, compares them, and if the 1st value is greater than the 2nd value, pushes `1` onto the stack; otherwise pushes `0` onto the stack. If either of the values is greater than 2<sup>*n*</sup>, the operation will fail. *n* can be any integer between 4 and 128. | *n + 15* |
| lt.*n*    | Pops top two items from the stack, compares them, and if the 1st value is less than the 2nd value, pushes `1` onto the stack; otherwise pushes `0` onto the stack. If either of the values is greater than 2<sup>*n*</sup>, the operation will fail. *n* can be any integer between 4 and 128. | *n + 14* |
| rc.*n*    | Pops the top item from the stack, checks if it is less than 2<sup>*n*</sup>, and if it is, pushes `1` onto the stack; otherwise pushes `0` onto the stack. *n* can be any integer between 4 and 128.| *n + 6* |

### Selection instructions

| Operation | Description                            | Cycles |
| --------- | -------------------------------------- | :----: |
| choose.1  | Pops top 3 items from the stack, and pushes either the 1st or the 2nd value back onto the stack depending on whether the 3rd value is `1` or `0`. For example, assuming `S0` is the top of the stack, `S0 S1 1` becomes `S0`, while `S0 S1 0` becomes `S1`. This operation will fail if the 3rd stack item is not a binary value. | 1 |
| choose.2  | Pops top 6 items from the stack, and pushes either the 1st or the 2nd pair of values back onto the stack depending on whether the 5th value is `1` or `0`. For example, assuming `S0` is the top of the stack, `S0 S1 S2 S3 1 S5` becomes `S0 S1`, while `S0 S1 S2 S3 0 S5` becomes `S2 S3` (notice that `S5` is discarded in both cases). This operation will fail if the 5th stack item is not a binary value. | 1 |

### Cryptographic instructions

| Operation | Description                            | Cycles |
| --------- | -------------------------------------- | :----: |
| hash.*n*  | Pops top *n* items from the stack, computes their hash using Rescue hash function, and pushes the result back onto the stack. The result is always represented by 2 stack items. *n* can be any integer between 1 and 4. | ~ 16 |
| mpath.*n* | Pops top 2 items from the stack, uses them to compute a root of a Merkle authentication path for a tree of depth *n*, and pushes the result onto the stack. The result is always represented by 2 stack items. Input tapes `A` and `B` are expected to contain nodes in of the Merkle authentication path.  | ~ *32n* |