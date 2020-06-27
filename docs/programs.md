# Programs in Distaff VM
TODO

## Execution tree
Distaff programs can be thought of as execution trees of instructions. As a program is executed, a specific path through the tree is taken. The actual representation of a program is slightly more complex. For example, a program execution tree actually consists of program blocks each with its own structure and execution semantics. At the high level, there are two types of blocks: instruction blocks and control blocks. Both are explained below.

### Instruction blocks
An instruction block is just a a sequence of instructions, where each instruction is a tuple *(op_code, op_value)*. For vast majority of instructions `op_value = 0`, but there are some instructions where it is not. For example, for a `PUSH` instruction, `op_value` is set to the value which is to be pushed onto the stack.

An instruction block imposes the following restrictions on its content:
* There must be at least one instruction in the block.
* An instruction block cannot contain any of the flow control instructions. This includes: `BEGIN`, `TEND`, `FEND`, `LOOP`, `CONTINUE`, `BREAK`, and `HACC` instructions.

### Control blocks
Control blocks are used to specify flow control logic of a program. Currently, there are 3 types of control blocks: (1) group blocks, (2) switch blocks, and (3) loop blocks. Specifics of each type of these are described below.

#### Group blocks
A group block is used to group several blocks together, and has the following structure:
```
Group {
    blocks : Vector<ProgramBlock>,
}
```
where, `ProgramBlock` can be an instruction block or a control block.

Execution semantics of a group block are as follows:
* Each block in the `blocks` vector is executed one after the other. 

A group block imposes the following restrictions on its content:
* There must be at least one block in the `blocks` vector.
* An instruction block cannot be followed by another instructions block.
* Number of instructions in an instruction block must be one less than a multiple of 16 (e.g. 15, 31, 47 etc.), unless it is the very last block in the `blocks` vector. In such a case, the number of instructions must be a multiple of 16 (e.g. 16, 32, 48 etc.).

To illustrate the last point, assume our vector of blocks looks like so:
```
[b0, b1, b2]
```
* If `b0` is an instruction block, number of instructions in `b0` must be one less than a multiple of 16.
* If `b2` is an instruction block, number of instructions in `b2` must be a multiple of 16.

#### Switch blocks
A switch block is used to describe conditional branching (i.e. *if/else* statements), and has the following structure:
```
Switch {
    true_branch  : Vector<ProgramBlock>,
    false_branch : Vector<ProgramBlock>,
}
```
Execution semantics of a switch block are as follows:
* If the top of the stack is `1`, blocks in the `true_branch` is executed one after the other;
* If the top of the stack is `0`, blocks in the `false_branch` is executed one after the other.
* If the top of the stack is neither `0` nor `1`, program execution fails.

A switch block imposes the following restrictions on its content:
* The first block in the `true_branch` vector must be an instruction block which has `ASSERT` as its first operation. This guarantees that this branch can be executed only if the top of the stack is `1`.
* The first block in the `false_branch` vector must be an instruction block which has `NOT ASSERT` as its first two instructions. This guarantees that this branch can be executed only if the top of the stack is `0`.
* Within `true_branch` and `false_branch` vectors, an instruction block cannot be followed by another instructions block.
* Number of instructions in an instruction block must be one less than a multiple of 16 (e.g. 15, 31, 47 etc.), unless it is the very last block in the `true_branch` or `false_branch` vectors. In such a case, the number of instructions must be a multiple of 16 (e.g. 16, 32, 48 etc.).

#### Loop block
A loop block is used to describe a sequence of instructions which is to be repeated zero or more times based on some condition (i.e. *while* statement). Structure of a loop block looks like so:
```
Loop {
    body : Vector<ProgramBlock>,
    skip : InstructionBlock,
}
```
where, `skip` is an instruction block containing the following sequence of instructions: `NOT ASSERT` followed by 14 `NOOP`'s.

Execution semantics of a loop block are as follows:
* If the top of the stack is `1`, blocks in the `body` vector are executed one after the other.
  * If after executing all blocks in the `body` vector, the top of the stack is `1`, the `body` blocks are executed again. This process is repeated until the top of the stack is `0`.
* If the top of the stack is `0`, `skip` block is executed.
* If the top of the stack is neither `0` nor `1`, program execution fails.

Loop block imposes the following restrictions on its content:
* The first block in the `body` vector must be an instruction block which has `ASSERT` as its first operation. This guarantees that a loop iteration can be entered only if the top of the stack is `1`.
* Within the `body` vector, an instruction block cannot be followed by another instructions block.
* Number of instructions in an instruction block must be one less than a multiple of 16 (e.g. 15, 31, 47 etc.).

It is expected that at the end of executing all `body` block, the top of the stack will contain a binary value (i.e. `1` or `0`). However, this is not enforced at program construction time, and if the top of the stack is not binary, the program will fail at execution time.

## Example programs

### Linear program
The simplest program is a linear sequence of instructions with no branches or loops:
```
a0, a1, ..., a_i
```
where, a<sub>0</sub>, a<sub>1</sub> etc. are instructions executed one after the other. Such a program can be described by a single group block like so:

<p align="center">
    <img src="assets/prog_tree1.dio.png">
</p>
To briefly explain the diagram:

* The outer rectangle with rounded corners represents a control block. In this case, it is a group block B<sub>0</sub>.
* This group block contains a single instruction block, which is represented by the inner rectangle.

### Program with branches
Let's add some conditional logic to our program. The program below does the following:
* First, instructions a<sub>0</sub> . . . a<sub>i</sub> are executed.
* Then, if the top of the stack is `1`, instructions b<sub>0</sub> . . . b<sub>j</sub> are executed. But if the top of the stack is `0`, instructions c<sub>0</sub> . . . c<sub>k</sub> are executed.
* Finally, instructions d<sub>0</sub> . . . d<sub>n</sub> are executed.

```
a0, a1, ..., a_i
if.true
    b0, b1, ..., b_j
else
    c0, c1, ..., c_k
end
d0, d1, ..., d_n
```
A diagram for this program would look like so:

<p align="center">
    <img src="assets/prog_tree2.dio.png">
</p>

Here we have bock B<sub>0</sub> which groups 3 other blocks together. The first one is an instruction block, the second one is a switch block describing the *if/else* statement, and the last one is another group block which contains a single instruction block with instructions d<sub>0</sub> . . . d<sub>n</sub>.

### Programs with nested blocks
Let's add nested control logic to our program. The program below is the same as the program from the previous example, except the *else* clause of the *if/else* statement now also contains a loop. This loop will keep executing instructions d<sub>0</sub> . . . d<sub>n</sub> as long as, right after d<sub>n</sub> is executed, the top of the stack is `1`. Once, the top of the stack becomes `0`, instructions e<sub>0</sub> . . . e<sub>m</sub> are executed, and then execution moves on to instructions f<sub>0</sub> . . . f<sub>l</sub>.
```
a0, a1, ..., a_i
if.true
    b0, b1, ..., b_j
else
    c0, c1, ..., c_k
    while.true
        d0, d1, ..., d_n
    end
    e0, e1, ..., e_m
end
f0, f1, ..., f_l
```
A diagram for this program would look like so:

<p align="center">
    <img src="assets/prog_tree3.dio.png">
</p>

Here, we have 4 control blocks, where loop blocks B<sub>2</sub> is nested within the *else* branch of block B<sub>1</sub>.

## Program hash
All Distaff programs can be reduced to a 16-byte hash represented by a single element in a 128-bit field. The hash is designed to target 128-bit preimage and second preimage resistance, and 64-bit collision resistance.

Program hash is computed from hashes of individual program blocks in a manner similar to computing root of a Merkle tree.

For example, let's say our program consists of 3 control blocks and looks like so:

<p align="center">
    <img src="assets/prog_hash1.dio.png">
</p>

Hash of this program is computed like so:

1. First, we compute hash of instruction block *a<sub>0</sub> . . . a<sub>i</sub>*.
2. Then, we compute hashes of *b<sub>0</sub> . . . b<sub>j</sub>* and *c<sub>0</sub> . . . c<sub>k</sub>*, and combine them with the hash of block B<sub>1</sub>.
3. Then, we merge hash of block B<sub>1</sub> with hash of *a<sub>0</sub> . . . a<sub>i</sub>*, and call the resulting value *h<sub>1</sub>*.
4. Then, we compute hash of *d<sub>0</sub> . . . d<sub>n</sub>* and transform it into hash of block B<sub>2</sub>.
5. Then, we merge hash of block B<sub>2</sub> with *h<sub>1</sub>* to get the hash of block B<sub>0</sub>.
6. Finally, we merge hash of block B<sub>0</sub> with value `0` to obtain the hash of the entire program *h<sub>p</sub>*.

Graphically, this process looks like so:

<p align="center">
    <img src="assets/prog_hash2.dio.png">
</p>

As mentioned above, hashes of Distaff programs are computed in a manner similar to roots of Merkle trees. This is by design. Using this property of program hashes, we can selectively reveal any of the program blocks while keeping the rest of the program private (e.g. secret programs with public pre/post conditions).

For example, if we wanted to reveal instruction sequence *d<sub>0</sub> . . . d<sub>n</sub>*, we could do so by making intermediate hash *h<sub>1</sub>* public. Then, anyone would be able to reconstruct the root hash from these two pieces of data and be sure that *d<sub>0</sub> . . . d<sub>n</sub>* is, in fact, the last sequence of instructions executed in the program with hash *h<sub>p</sub>*.

### Hashes of control blocks
Hashes of control blocks are defined as tuples of two 128-bit elements *(v<sub>0</sub>, v<sub>1</sub>)*. These tuples are constructed as follows:

For **group blocks**, it is just a hash of `blocks` contained within the group:
* *v<sub>0</sub> = hash_seq(blocks)*
* *v<sub>1</sub> = 0*

where, *[hash_seq](#hash_seq-procedure)* is the procedure used to hash a sequence of program blocks into a single 128-bit value.

For **switch blocks**, it is hashes of both branches:
* *v<sub>0</sub> = hash_seq(true_branch)*
* *v<sub>1</sub> = hash_seq(false_branch)*

For **loop blocks**, it is a hash of the loop's `body` and the `skip` block:
* *v<sub>0</sub> = hash_seq(body)*
* *v<sub>1</sub> = hash_seq(skip)*

A side note: *hash_seq(body)* is called a loop image as it binds each iteration of the loop to a specific hash.

### hash_ops procedure
The purpose of *hash_ops* procedure is to hash a sequence of instructions. The procedure takes a state of four 128-bit elements as an input, merges each instruction into the state, and returns the updates state as the output.

The pseudo-code for this procedure looks like so:
```
with inputs: state and instruction:
for each instruction do:
    state = add_round_constants(state);
    state = apply_sbox(state);
    state = apply_mds(state);
    state[0] = state[0] + instruction.op_code;
    state[1] = state[1] + instruction.op_value;
    state = add_round_constants(state);
    state = apply_inverse_sbox(state);
    state = apply_mds(state);
return state;
```
where:
  * `state` is an array of four 128-bit elements.
  * `instructions` is a vector of instruction tuples.

The body of the above loop is similar to the round of [Rescue](https://eprint.iacr.org/2019/426) hash function. But there are significant differences. Specifically, injecting values in the middle of every round invalidates security proof of sponge construction. Thus, it is possible that this approach is insecure, and will need to be modified in the future.

### hash_acc procedure
The purpose of *hash_acc* procedure is to merge hash of a control block, which is a tuple of two 128-bit values, with a single 128-bit value. The output of the *hash_acc* procedure is a single 128-bit element.

Denoting *(v<sub>0</sub>, v<sub>1</sub>)* to be the hash of the control block, and *h* to be the 128-bit value, high-level pseudo-code for *hash_acc* procedure looks like so:
```
with inputs: v0, v1, h:
let state = [v0, v1, h, 0];
for 14 rounds do:
    state = add_round_constants(state);
    state = apply_sbox(state);
    state = apply_mds(state);
    state = add_round_constants(state);
    state = apply_inverse_sbox(state);
    state = apply_mds(state);
return state[0];
```
The above is a modified version of [Rescue](https://eprint.iacr.org/2019/426) hash function. This modification adds half-rounds to the beginning and to the end of the standard Rescue hash function to make the arithmetization of the function fully foldable. This should not impact security properties of the function, but it is worth noting that it has not been studied to the same extent as the standard Rescue hash function.

### hash_seq procedure
The purpose of *hash_seq* procedure is to hash a sequence of program blocks into a single 128-bit value. The procedure works as follows:

1. First, we initialize a `state` of four 128-bit elements to `0`'s.
2. Then, we consume blocks from the sequence one by one, hash them, and merge their hashes into the `state`. Hashing and merging is performed as follows:
    1. Instruction blocks are hashed using [hash_ops](#hash_ops-procedure) procedure described above, and the return value of the *hash_ops* procedure becomes the new `state`.
    2. Control blocks are hashed according to the [rules](#Hashes-of-control-blocks) described above. The resulting hash is merged into the state using [hash_acc](#Hash_acc-procedure) procedure.
2. Finally, we return the first element of the state as the hash value of the sequence.

For example, let's say we want to hash a sequence of blocks `[b0, b1, b2]`, where `b0` and `b2` are instruction blocks, and `b1` is a control block. Hashing of this sequence will work as follows:

1. First we initialize the state as `s0 = [0, 0, 0, 0]`.
2. Then, we merge `b0` into the state as `s1 = hash_ops(s0, b0)`;
3. Then, we compute the hash of `b1` as `(v0, v1) = hash(b1)`, and merge it into the state as `s2 = hash_acc(s1, v0, v1)`.
4. Then, we merge `b0` into the state as `s3 = hash_ops(s2, b2)`.
5. Finally, we return `s3[0]` as the hash of the sequence.

There is one extra thing to note: when a control block is followed by an instruction block, we do not truncate the result of *hash_acc* procedure to a single value. That is, in the above example, `s2` will be the full state of 4 field elements, rather than a single field element.

## Hash computations in the VM
Distaff VM computes program hash as the program is executed on the VM. Several components of the VM are used in hash computations. These components are:

* **sponge state** which holds running hash of the currently executing program block; sponge state takes up 4 registers.
* **context stack** which holds hashes of parent blocks to the currently executing program block; context stack takes up between 1 and 16 registers (depending on the level of nesting in the program).

General intuition for hashing process is as follows:

1. At the start of every program block, we push current hash of its outer block onto the `context stack`, and set hash of the new block to `0` (this is done be resetting the `sponge state`).
2. Then, as we read instructions contained in the block, we merge each instruction into the block's hash.
    1. If we encounter a new block, we process it recursively starting at step 1.
3. Once the end of the block is reached, we pop hash of the parent block from the `context stack` and merge our block hash into it.

Each of these states is explained in detail below.

### Initiating a new program block
A new program block is started by the `BEGIN` operation. The operation does the following:

1. Pushes hash of the current block onto the `context stack`;
2. Sets all registers of `sponge state` to `0`.

A diagram of `BEGIN` operation is shown below. By convention, result of hashing is in the first register of `sponge state`. So, `a0` is the hash of of the current block right before `BEGIN` operation is executed.
```
╒═══ sponge ═══╕  ╒══ context ═══╕
[s0, s1, s2, s3], [              ]
                🡣
[ 0,  0,  0,  0], [s0            ]
```

### Accumulating block hash
As instructions are executed in the VM, each instruction is merged into the `sponge state` using a modified version of Rescue round.

### Merging block hash into parent hash
A block can be terminate by one of two operations: `TEND` or `FEND`. These operations behave similarly. Specifically:

* Both operations contain a payload which they move into the `sponge state`.
* Both operations pop hash of the parent block from the `context stack` and move it into the `sponge state`.
* Both operations propagate hash of the current block into the `sponge state` at the next step.

However, these operations arrange `sponge state` slightly differently, and are intended to terminate specific branches of execution.

For example, recall that hash of a switch block is defined as tuple *(v0, v1)*, where:
* *v<sub>0</sub> = hash(true_branch)*
* *v<sub>1</sub> = hash(false_branch)*

When the VM executes *true_branch* of a switch block, the block must be terminated with `TEND(v1)` operation. A diagram of this operation looks like so:
```
╒═══ sponge ═══╕  ╒══ context ═══╕
[s0, s1, s2, s3], [c0            ]
                🡣
[s0, v1, c0,  0], [              ]
```
Note that by the time `TEND` instruction is reached, the first register of the sponge will contain hash of the *true_branch*. Thus, `s0 = v0`, and the result of executing `TEND(v1)` will be sponge state set to `[v0, v1, c0, 0]`, where `c0` is the hash of the parent block.

On the other hand, when the VM executes *false_branch* of a switch block, the block must terminate with `FEND(v0)` operation. A diagram of this operation looks like so:
```
╒═══ sponge ═══╕  ╒══ context ═══╕
[s0, s1, s2, s3], [c0            ]
                🡣
[v0, s0, c0,  0], [              ]
```
Note also that by the time `FEND` instruction is reached, the first register of the sponge will contain hash of the *false_branch*. Thus, `s0 = v1`, and the result of executing `FEND(v0)` will be sponge state set to `[v0, v1, c0, 0]`.

The crucially important thing here is that by the time we exit the block, `sponge state` is the same, regardless of which branch was taken.

Similar methodology applies to other blocks as well:

* To exit a group block, we should execute `TEND(0)` operation.
* To exit a loop block, we should use `TEND(v1)` if the body of the loop was executed at least once, and `FEND(v0)`, if the body of the loop was never entered.

After `sponge state` has been arranged as described above, `HACC` operation is executed 14 times to perform [hash_acc](#hash_acc-procedure) procedure. After this, program execution can resume with the following instruction or the next code block in the sequence.

#### A note on operation alignment
`TEND` and `FEND` operations can be executed only on steps which are multiples of 16 (e.g. 16, 32, 48 etc.). Trying to execute them on other steps will result in program failure.

Since `TEND`/`FEND` instruction is followed by 14 `HACC` instruction, we have a cycle of 16 instructions with the last slot empty. This is convenient because we can fill it with a `BEGIN` instruction in cases when one program block is immediately followed by another. Specifically, the inter-block sequence of instructions could look like so:
```
... TEND(v1) HACC HACC HACC HACC HACC HACC HACC
    HACC     HACC HACC HACC HACC HACC HACC BEGIN ...
```

### Loops
Ability to execute unbounded loops requires additional structures. Specifically, we need a `loop stack` to holds images of loop bodies for currently active loops. Loop stack takes up between 0 and 8 registers to support nested loops up to 8 levels deep.

Loop execution proceeds as follows:

First, we check if the top of the stack is `1` or `0`. If it is `0`, we don't need to enter the loop, and instead we execute the following sequence of operations (padding with `NOOP`s is skipped for brevity):
```
BEGIN NOT ASSERT FEND(v0)
```
Recall that hash of a loop block is defined as a tuple *(v<sub>0</sub>, v<sub>1</sub>)*, where:
* *v<sub>0</sub> = hash(body)*
* *v<sub>1</sub> = hash(skip)*, where *skip* is `NOT ASSERT` sequence of instructions.

So, executing `FEND(v0)` operation sets `sponge state` to the following: `[v0, v1, h0, 0]`, where `h0` is the hash of the parent block. Then, we executed 14 `HACC` operations and we are done.

If, however, the top of the stack is `1`, we do need to enter the loop. We do this by executing a `LOOP` operation. This operation is similar to the `BEGIN` operation but it also contains a payload. This payload is set to the loop's image (hash of loop body).

Executing `LOOP(i0)` operation (where `i0` is the loop's image) does the following:

1. Pushes the operation payload `i0` onto the `loop stack`;
2. Pushes hash of the current block onto the `context stack`;
3. Sets all registers of `sponge state` to `0`.

A diagram of `LOOP(i0)` operation is shown below:
```
╒═══ sponge ═══╕  ╒══ context ═══╕ ╒═ loop stack ═╕
[s0, s1, s2, s3], [              ] [              ]
                         🡣
[ 0,  0,  0,  0], [s0            ] [i0            ]
```

After the `LOOP` operation, the body of the loop is executed similar to any other block.

If after executing the loop's body, the top of the stack is `1`, we execute a `CONTINUE` operation. `CONTINUE` operation does the following:

1. Checks whether the first register of the `sponge` is equal to the value at the top of the `loop stack` (i.e. `s0 = i0`). If it is not, the program fails.
2. Check whether the value on the top of the stack is `1`. If it is not, the program fails.
3. Resets the `sponge state`.

A diagram of `CONTINUE` operation is as follows:
```
╒═══ sponge ═══╕  ╒══ context ═══╕ ╒═ loop stack ═╕
[s0, s1, s2, s3], [h0            ] [              ]
                         🡣
[ 0,  0,  0,  0], [h0            ] [i0            ]
```
To summarize: the effect of `CONTINUE` operation is to make sure that the sequence of instructions executed in the last iteration of the loop was indeed loop's body, and also to prepare the sponge for the next iteration of the loop.

After the we execute the `CONTINUE` operation, loop body can be executed again.

If after executing the loop's body, the top of the stack is `0`, we execute `BREAK` operation. `BREAK` operation does the following:

1. Checks whether the first register of the `sponge` is equal to the value at the top of the `loop stack` (i.e. `s0 = i0`). If it is not, the program fails.
2. Check whether the value on the top of the stack is `0`. If it is not, the program fails.
3. Removes the top value from the `loop stack`.

A diagram of `BREAK` operation is as follows:
```
╒═══ sponge ═══╕  ╒══ context ═══╕ ╒═ loop stack ═╕
[s0, s1, s2, s3], [h0            ] [i0            ]
                         🡣
[s0, s1, s2, s3], [h0            ] [              ]
```
After the `BREAK` operation is executed we execute `TEND(v1)` operation. This sets the `sponge state` to `[v0, v1, h0, 0]`. We then execute 14 `HACC` operations.

Again, it is important to note that regardless of whether we enter the loop or not, `sponge state` ends up set to  `[v0, v1, h0, 0]` before we start executing `HACC` operations.

#### Loop execution example
To illustrate execution of a loop on a concrete example, let's say we have the following loop:
```
while.true
  a0, a1, ... a14
end
```
Let's also say that for a given input, the top of the stack is `1`, and it changes to `0` after we execute *a<sub>0</sub>, a<sub>1</sub>, . . ., a<sub>14</sub>* instructions twice. Then, the sequence of instructions executed on the VM to complete this loop will be:
```
LOOP
  a0  a1  a2  a3  a4  a5  a6 a7
  a8  a9 a10 a11 a12 a13 a14 CONTINUE
  a0  a1  a2  a3  a4  a5  a6 a7
  a8  a9 a10 a11 a12 a13 a14 BREAK
TEND
```