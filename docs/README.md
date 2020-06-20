# Distaff VM
Distaff VM is a simple [stack machine](https://en.wikipedia.org/wiki/Stack_machine). This means all values live on the stack and all operations work with values near the top of the stack. 

### The stack
Currently, Distaff VM stack can be up to 32 items deep (this will be increased in the future). However, the more stack space a program uses, the longer it will take to execute, and the larger the execution proof will be. So, it pays to use stack space judiciously.

Values on the stack must be elements of a [prime field](https://en.wikipedia.org/wiki/Finite_field) with modulus `340282366920938463463374557953744961537` (which can also be written as 2<sup>128</sup> - 45 * 2<sup>40</sup> + 1). This means that all valid values are in the range between `0` and `340282366920938463463374557953744961536` - this covers almost all 128-bit integers.   

All arithmetic operations (addition, multiplication) also happen in the same prime field. This means that overflow happens after a value reaches field modulus. So, for example: `340282366920938463463374557953744961536 + 1 = 0`.

Besides being field elements, values in Distaff VM are untyped. However, some operations expect binary values and will fail if you attempt to execute them using non-binary values. Binary values are values which are either `0` or `1`.

### Programs
Programs in Distaff VM are represented by a directed acyclic graph of [instructions](isa.md). You can construct this graph manually, but it is much easier to construct it by compiling [Distaff assembly](assembly.md) source code.

In fact, Distaff assembly is the preferred way of writing programs for Distaff VM, and all references and examples in these docs use assembly syntax.

### Inputs / outputs
Currently, there are 3 ways to get values onto the stack:

1. You can use `push` operations to push values onto the stack. These values become a part of the program itself, and, therefore, cannot be changed between program executions. You can think of them as constants.
2. You can initialize the stack with a set of public inputs as described [here](https://github.com/GuildOfWeavers/distaff#program-inputs). Because these inputs are public, they must be shared with a verifier for them to verify program execution.
3. You can provide unlimited number of secret inputs via input tapes `A` and `B`. Similar to public inputs, these tapes are defined as a part of [program inputs](https://github.com/GuildOfWeavers/distaff#program-inputs). To move secret inputs onto the stack, you'll need to use `read` operations.

Values remaining on the stack after a program is executed can be returned as program outputs. You can specify exactly how many values (from the top of the stack) should be returned. Currently, the number of outputs is limited to 8. A way to return a large number of values (hundreds or thousands) is not yet available, but will be provided in the future.

### Turing-completeness
Distaff VM is currently not [Turing-complete](https://en.wikipedia.org/wiki/Turing_completeness). However, conditional execution (i.e. if/else statements) are fully supported, and support for bounded (and maybe even un-bounded) loops will be added in the future.

### Memory
Currently, Distaff VM has no random access memory - all values live on the stack. However, a memory module will be added in the future to enable saving values to and reading values from RAM.

### Program hash
All Distaff programs can be reduced to a single 32-byte value, called program hash. Once a `Program` object is constructed (e.g. by compiling assembly code), you can access this hash via `Program.hash()` method. This hash value is used by a verifier when they verify program execution. This ensure that the verifier verifies execution of a specific program (e.g. a program which the prover had committed to previously). The methodology for computing program hash is described below.

#### 1. Deconstructing a program into execution paths
First, a program is deconstructed into all possible linear execution paths. For example, if a program looks like so:

```
push.3
push.5
read
if.true
    add
else
    mul
endif
```
It will have 2 possible execution paths:
```
path 1: push.3 push.5 read assert add
path 2: push.3 push.5 read not assert mul
```

#### 2. Hashing execution paths
After all possible execution paths are generated, each path is reduced to a 32-byte value using the following methodology:

1. First, the path is padded with `noop` operations. The padding ensures that:
   1. The program consists of at least 16 operations.
   2. The number of operations is a power of 2 (16, 32, 64 etc.).
2. Then, a hash function is used to sequentially hash all instructions together. This hash function is based on Rescue hash function - however, it deviates significantly from the original construction. Security implications of this deviation have not been analyzed. It is possible that this hashing scheme is insecure, and will need to be changed in the future.

The hash function works as follows:

First, a state of 4 field elements is initialized to `0`. Then, the following procedure is applied:
```
for each op_code in the path do:
    add round constants;
    apply s-box;
    apply MDS;
    state[0] = state[0] + state[2] * op_code;
    state[1] = state[1] * state[3] + op_code;
    add round constants;
    apply inverse s-box;
    apply MDS;
```
where `op_code` is the opcode of the operation being executed on the VM. As mentioned above, this is based on the Rescue hash function but with the following significant differences:
1. The opcodes of the program are injected into the state in the middle of every round.
2. The number of rounds is equal to the number of operations in the execution path.

After the above procedure has been applied for all operations of the path, the first two elements of the state are returned as the hash of the path.

#### 3. Combining path hashes into a Merkle tree
Once all execution paths have been reduced to 32-byte values, a Merkle tree is build from these value. The root of the tree is the hash of the entire program.

The hash function used for building the Merkle tree is defined at the time of program compilation and can be any standard hash function - e.g. SHA256 or Blake3.