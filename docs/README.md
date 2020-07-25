# Distaff VM
Distaff VM is a simple [stack machine](https://en.wikipedia.org/wiki/Stack_machine). This means all values live on the stack and all operations work with values near the top of the stack. 

### The stack
Currently, Distaff VM stack can be up to 32 items deep (this will be increased in the future). However, the more stack space a program uses, the longer it will take to execute, and the larger the execution proof will be. So, it pays to use stack space judiciously.

Values on the stack must be elements of a [prime field](https://en.wikipedia.org/wiki/Finite_field) with modulus `340282366920938463463374557953744961537` (which can also be written as 2<sup>128</sup> - 45 * 2<sup>40</sup> + 1). This means that all valid values are in the range between `0` and `340282366920938463463374557953744961536` - this covers almost all 128-bit integers.   

All arithmetic operations (addition, multiplication) also happen in the same prime field. This means that overflow happens after a value reaches field modulus. So, for example: `340282366920938463463374557953744961536 + 1 = 0`.

Besides being field elements, values in Distaff VM are untyped. However, some operations expect binary values and will fail if you attempt to execute them using non-binary values. Binary values are values which are either `0` or `1`.

### Programs
Programs in Distaff VM are structures as an [execution graph](programs.md) of program blocks each consisting of a sequence of VM [instructions](isa.md). You can construct this graph manually, but it is much easier to construct it by compiling [Distaff assembly](assembly.md) source code.

In fact, Distaff assembly is the preferred way of writing programs for Distaff VM, and all references and examples in these docs use assembly syntax.

### Inputs / outputs
Currently, there are 3 ways to get values onto the stack:

1. You can use `push` operations to push values onto the stack. These values become a part of the program itself, and, therefore, cannot be changed between program executions. You can think of them as constants.
2. You can initialize the stack with a set of public inputs as described [here](https://github.com/GuildOfWeavers/distaff#program-inputs). Because these inputs are public, they must be shared with a verifier for them to verify program execution.
3. You can provide unlimited number of secret inputs via input tapes `A` and `B`. Similar to public inputs, these tapes are defined as a part of [program inputs](https://github.com/GuildOfWeavers/distaff#program-inputs). To move secret inputs onto the stack, you'll need to use `read` operations.

Values remaining on the stack after a program is executed can be returned as program outputs. You can specify exactly how many values (from the top of the stack) should be returned. Currently, the number of outputs is limited to 8. A way to return a large number of values (hundreds or thousands) is not yet available, but will be provided in the future.

### Memory
Currently, Distaff VM has no random access memory - all values live on the stack. However, a memory module will be added in the future to enable saving values to and reading values from RAM.

### Program hash
All Distaff programs can be reduced to a single 32-byte value, called program hash. Once a `Program` object is constructed (e.g. by compiling assembly code), you can access this hash via `Program.hash()` method. This hash value is used by a verifier when they verify program execution. This ensure that the verifier verifies execution of a specific program (e.g. a program which the prover had committed to previously). The methodology for computing program hash is described [here](programs.md#Program-hash).