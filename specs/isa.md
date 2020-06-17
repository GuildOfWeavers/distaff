# Distaff VM instruction set



### Flow control operations

| Instruction | Opcode   | Description                             |
| ----------- | :------: | --------------------------------------- |
| NOOP        | 00000000 | Does nothing. |
| BEGIN       | 11111111 | Marks the beginning of a program. Every program must start with the `BEGIN` operation. |
| ASSERT      | 00010000 | Pops the top item from the stack and checks if it is equal to `1`. If it is not equal to `1` the program fails. |

### Input operations

| Instruction | Opcode   | Description                            |
| ----------- | :------: | -------------------------------------- |
| PUSH        | 00001000 | Pushes the value of the next opcode onto the stack. The value can be any field element. |
| READ        | 00001001 | Pushes the next value from the input tape `A` onto the stack. |
| READ2       | 00001010 | Pushes the next values from input tapes `A` and `B` onto the stack. Value from input tape `A` is pushed first, followed by the value from input tape `B`. |

### Stack manipulation operations

| Instruction | Opcode   | Description                            |
| ----------- | :------: | -------------------------------------- |
| DUP         | 00001011 | Pushes a copy of the top stack item onto the stack (duplicates the top stack item). |
| DUP2        | 00001100 | Pushes copies of the top two stack items onto the stack. |
| DUP4        | 00001101 | Pushes copies of the top four stack items onto the stack. |
| PAD2        | 00001110 | Pushes two `0` values onto the stack. Equivalent to `PUSH 0 DUP`. |
| DROP        | 00010000 | Removes the top item from the stack. |
| DROP4       | 00010001 | Removes top four items from the stack. |
| SWAP        | 00011010 | Moves the second from the top stack item to the top of the stack (swaps top two stack items). |
| SWAP2       | 00011011 | Moves 3rd and 4th stack items to the top of the stack. For example, assuming `S0` is the top of the stack, `S0 S1 S2 S3` becomes `S2 S3 S0 S1`. |
| SWAP4       | 00011100 | Moves 5th through 8th stack items to the top of the stack. For example, assuming `S0` is the top of the stack, `S0 S1 S2 S3 S4 S5 S6 S7` becomes `S4 S5 S6 S7 S0 S1 S2 S3`. |
| ROLL4       | 00011101 | Moves 4th stack item to the top of the stack. For example, assuming `S0` is the top of the stack, `S0 S1 S2 S3` becomes `S3 S0 S1 S2`.  |
| ROLL8       | 00011110 | Moves 8th stack item to the top of the stack. For example, assuming `S0` is the top of the stack, `S0 S1 S2 S3 S4 S5 S6 S7` becomes `S7 S0 S1 S2 S3 S4 S5 S6`. |

### Arithmetic and boolean operations

| Instruction | Opcode   | Description                            |
| ----------- | :------: | -------------------------------------- |
| ADD         | 00011001 | Pops top two items from the stack, adds them, and pushes the result back onto the stack. |
| MUL         | 00011010 | Pops top two items from the stack, multiplies them, and pushes the result back onto the stack. |
| INV         | 00000011 | Pops the top item from the stack, computes its multiplicative inverse, and pushes the result back onto the stack. This can be used to emulate division with a sequence of two operations: `INV MUL`. If the value at the top of the stack is `0`, the operation will fail.
| NEG         | 00000100 | Pops the top item from the stack, computes its additive inverse, and pushes the result back onto the stack. This can be used to emulate subtraction with a sequence of two operations: `NEG ADD` |
| NOT         | 00000101 | Pops the top item from the stack, subtracts it from value `1` and pushes the result back onto the stack. In other words, `0` becomes `1`, and `1` becomes `0`. This is equivalent to `PUSH 1 SWAP NEG ADD` but also enforces that the top stack item is a binary value. |

### Comparison operations

| Instruction | Opcode   | Description                            |
| ----------- | :------: | -------------------------------------- |
| EQ          | 00010101 | Pops top two items from the stack, compares them, and if their values are equal, pushes `1` back onto the stack; otherwise pushes `0` back onto the stack. |
| CMP         | 00000001 | Pops top 7 items from the top of the stack, performs a single round of binary comparison, and pushes the result back onto the stack. This operation can be used as a building block for *less then* and *greater than* operations (see [here](#Checking-inequality)). |
| BINACC      | 00000010 | Pops top 2 items from the top of the stack, performs a single round of binary aggregation, and pushes the result back onto the stack. This operation can be used as a building block for range check operations (see [here](#Checking-binary-decomposition)). |

### Selection operations

| Instruction | Opcode   | Description                            |
| ----------- | :------: | -------------------------------------- |
| CHOOSE      | 00010110 | Pops 3 items from the top of the stack, and pushes either the 1st or the 2nd value back onto the stack depending on whether the 3rd value is `1` or `0`. For example, assuming `S0` is the top of the stack, `S0 S1 1` becomes `S0`, while `S0 S1 0` becomes `S1`. This operation will fail if the 3rd stack item is not a binary value. |
| CHOOSE2     | 00010111 | Pops 6 items from the top of the stack, and pushes either the 1st or the 2nd pair of values back onto the stack depending on whether the 5th value is `1` or `0`. For example, assuming `S0` is the top of the stack, `S0 S1 S2 S3 1 S5` becomes `S0 S1`, while `S0 S1 S2 S3 0 S5` becomes `S2 S3` (notice that `S5` is discarded in both cases). This operation will fail if the 5th stack item is not a binary value. |

### Cryptographic operations

| Instruction | Opcode   | Description                            |
| ----------- | :------: | -------------------------------------- |
| HASHR       | 00011000 | Pops top 6 items from the stack, computes a single round of a modified [Rescue](https://eprint.iacr.org/2019/426) hash function over these values, and pushes the results back onto the stack. This operation can be used to hash up to two 256-bit values. However, to achieve 120 bits of security, the `HASHR` operation must be applied at least 10 times in a row (see [here](#Hashing-in-Distaff-VM)).  |

## Value comparison in Distaff VM
There are 3 operations in Distaff VM which can be used to compare values: `EQ`, `CMP`, and `BINACC`. Using these operations you can check whether 2 values a equal, whether one value is greater or less than the other, and whether a value can be represented with a given number of bits.

`EQ` operation is by far the simplest one out of the three. If the two values on the top of the stack are equal, it pushes `1` onto the stack. If they are not equal, it pushes `0` onto the stack. Both values are removed in the process.

The other two operations are more complex and are described in detail below.

### Checking inequality
Using repeated execution of `CMP` operation you can determine if one value is greater or less than another value. Executing this operation consumes a single input from each of the input tapes. It also assumes that you've positioned items on the stack in an appropriate order. If items on the stack are not positioned correctly, the result of the operation will be undefined.

Supposed we wanted to compare 2 values: `a` and `b` (both are 128-bit field elements). To accomplish this, we'd need to position elements on the stack like so:

```
[p, 0, 0, 0, 0, 0, 0, a, b]
```
where `p` = 2<sup>n - 1</sup> for some `n` <= 128 such that 2<sup>n</sup> > `a`, `b`. For example, if `a` and `b` are unconstrained field elements, `p` should be set to 2<sup>127</sup>. Or, if `a` and `b` are know to be 64-bit numbers, `p` should be set to 2<sup>63</sup>.

Once the stack has been arranged in this way, we'll need to execute `CMP` operation `n` times in a row. As mentioned above, each execution of the operation consumes inputs from tapes `A` and `B`. The tapes must be populated with binary representations of values `a` and `b` respectively (in [big-endian](https://en.wikipedia.org/wiki/Endianness) order). For example, if `a = 5` and `b = 8`, input tape `A` should be `[0, 1, 0, 1]`, and input tape `B` should be `[1, 0, 0, 0]`.

After we execute `CMP` operation `n` number of times, the stack will have the following form:
```
[x, x, x, gt, lt, b_acc, a_acc, a, b]
```
where:
* `x` values are intermediate results of executing `CMP` operations and should be discarded.
* `gt` value will be `1` if `a` > `b`, and `0` otherwise.
* `lt` value will be `1` if `a` < `b`, and `0` otherwise.
* `a_acc` will be equal to the result of aggregating value `a` from its binary representation.
* `b_acc` will be equal to the result of aggregating value `b` from its binary representation.

To make sure that the comparison is valid, we need to check that `a` == `a_acc` and `b` == `b_acc`. If these checks pass, then both numbers can be represented by `n`-bit values. This, in turn, means that the comparison is valid. The instruction sequences below can be executed after the last `CMP` operation in the sequence to perform these comparisons and remove un-needed values from the stack:

```
// performs the comparisons and leaves only the lt value on the stack
DROP SWAP4 ROLL4 EQ ASSERT EQ ASSERT DROP DROP DROP

// performs the comparisons and leaves only the gt value on the stack
DROP SWAP4 ROLL4 EQ ASSERT EQ ASSERT DROP DROP SWAP DROP
```

Overall, the number of operations needed to compare 2 values is proportional to the size of the values. Specifically:

* Comparing two unconstrained field elements requires ~ 140 operations,
* Comparing two 64-bit values requires ~ 75 operations,
* Comparing two 32-bit values requires ~ 45 operations.

### Checking binary decomposition
Sometimes it may be useful to check whether a value fits into a certain number of bits. This can be accomplished with `CMP` operations, but `BINACC` operation provides a simpler way to do this.

Similar to `CMP` operation, `BINACC` operation needs to be executed `n` times in a row if we want to make sure that a value can be represented with `n` bits.

Each execution of the operation consumes a single input from tape `A`. The tape must be populated with binary representation of value `a` in [big-endian](https://en.wikipedia.org/wiki/Endianness) order. For example, if `a = 5`, input tape `A` should be `[0, 1, 0, 1]`.

Also similar to `CMP` operation, `BINACC` operation expect items on the stack to be arranged in a certain order. If the items are not arranged as shown below, the result of the operation is undefined:

```
[p, 0, a]
```
where:
  * `a` is the value we want to range-check,
  * `p` is equal to 2<sup>n - 1</sup>.

Once items on the stack have been arranged as described above, we execute `BINACC` instruction `n` times. This will leave the stack in the following form:
```
[x, a_acc, a]
```
where:
* `x` value is an intermediate results of executing `BINACC` operations and should be discarded.
* `a_acc` will be equal the result of aggregating value `a` from its binary representation.

To make sure that `a` can fit into `n` bits we need to check that `a_acc = a`. This can be done using the following sequence of operations:
```
DROP EQ ASSERT
```
The above sequence discards the first item, then checks the equality of the remaining two items (`1` is placed on the stack if their values are equal), and then asserts that the top of the stack is `1`.

Overall, the number of operations needed to determine whether a value can be represented by `n` bits is `n + 3`. Specifically:

* Checking if a value can be represented with 64 bits requires 67 operations,
* Checking if a value can be represented with 32 bits requires 35 operations.

## Hashing in Distaff VM
To compute hashes in Distaff VM you can use `HASHR` operation. This operation works with the top 6 items of the stack, and depending on what you want to do, you should position the values on the stack in specific orders.

Generally, we want to hash values that are 256 bits long. And since all values in Distaff VM are about 128 bits, we'll need 2 elements to represent each 256-bit value. By convention, values to be hashed are placed in the inner-most positions on the stack, and the result of hashing is also located in the inner-most positions.

For example, suppose we wanted to compute `hash(x)`. First, we'd represent `x` by a pair of elements `(x0, x1)`, and then we'd position these elements on the stack like so:
```
[0, 0, 0, 0, x1, x0]
```
In other words, the first 4 items of the stack should be set to `0`'s, and the following 2 items should be set to the elements representing the value we want to hash.

If we wanted to compute a hash of two values `hash(x, y)`, represented by elements `(x0, x1)` and `(y0, y1)` respectively, we'd position them on the stack like so:
```
[0, 0, y1, y0, x1, x0]
```
In both cases, after the hashing is complete, the result will be located in the 5th and 6th positions of the stack (the result is also represented by two 128-bit elements).

### Hashing programs

Hashing requires multiple invocations of `HASHR` operation, and there are a few things to be aware of:
1. To achieve adequate security (e.g. 120-bits), `HASHR` operation must be executed at least 10 times in a row. This is because each `HASHR` operation computes a single round of the hash function, and at least 10 rounds are required to achieve adequate security.
2. `HASHR` operation uses a schedule of constants which repeat every 16 steps. So, to make sure you get consistent results, the first `HASHR` operation in every sequence must happen on the step which is a multiple of 16 (e.g. 16, 32, 48 etc.). To ensure this alignment, you can always use `NOOP` operations to pad your programs.
3. The top two stack items are reserved for internal operations of the hash function. You need to make sure they are set to `0`'s before you start hashing values. This also means, that you can hash at most two 256-bit values at a time.

You can think of sequences of `HASHR` operations as of "mini-programs". Below is an example of a program which reads two 256-bit values from input tape `A` and computes their hash:
```
BEGIN NOOP  NOOP  NOOP  NOOP  NOOP  NOOP  NOOP
NOOP  NOOP  NOOP  READ  READ  READ  READ  PAD2
HASHR HASHR HASHR HASHR HASHR HASHR HASHR HASHR
HASHR HASHR DROP4
```
A quick explanation of what's happening here:
1. First, we pad the beginning of the program with `NOOP`'s so that the first `HASHR` operation happens on the 16th step.
2. Then, we read 4 values from the input tape `A`. These 4 values represent our two 256-bit values. We also push two `0`'s onto the stack by executing `PAD2` operation.
3. Then, we execute `HASHR` operation 10 times. Notice again that the first `HASHR` operation is executed on the 16th step.
4. The result of hashing is now in the 5th and 6th positions of the stack. So, we remove top 4 times from the stack (using `DROP4` operation) to move the result to the top of the stack.

You can also check an example of a more sophisticated program which uses `HASHR` operation to verify a Merkle authentication path [here](https://github.com/GuildOfWeavers/distaff/blob/master/src/examples/merkle.rs).

### Hash function
As mentioned previously, Distaff VM uses a modified version of [Rescue](https://eprint.iacr.org/2019/426) hash function. This modification adds half-rounds to the beginning and to the end of the standard Rescue hash function to make the arithmetization of the function fully foldable. High-level pseudo-code for the modified version looks like so:
```
for 10 iterations do:
    add round constants;
    apply s-box;
    apply MDS;
    add round constants;
    apply inverse s-box;
    apply MDS;
```
This modification should not impact security properties of the function, but it is worth noting that it has not been studied to the same extent as the standard Rescue hash function.

Another thing to note: current implementation of the hash function uses S-Box of power 3, but this will likely be changes to S-Box of power 5 in the future.