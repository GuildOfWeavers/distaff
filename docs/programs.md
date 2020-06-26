# Programs in Distaff VM
TODO

## Execution tree
Distaff programs can be thought of as execution trees of instructions. As a program is executed, a specific path through the tree is taken. The actual representation of a program is slightly more complex. For example, a program execution tree actually consists of instruction blocks each with its own structure and execution semantics. At the high level, there are two types of blocks: control blocks and code blocks. Both are explained below.

### Control blocks
Control blocks are used to specify flow control logic of a program. Currently, there are 3 types of control blocks: (1) group blocks, (2) switch blocks, and (3) loop blocks. Control blocks can contain one or more code blocks, and also have an optional `next` pointer which can be used to specify the next control block in a program. Specifics of each type of  control blocks are described below.

#### Group blocks
A group block is used to group sequences of instructions together. Besides the optional `next` pointer, a group block must contain a single code block. A data structure for a group block may look like so:
```
Group {
    content : CodeBlock,
    next?   : ControlBlock,
}
```
Execution semantics of a group block are as follows:
* First, `content` of the block is executed.
* Then, if `next` pointer is not null, execution moves to the next block.

#### Switch blocks
A switch block is used to describe conditional branching (i.e. *if/else* statements). Besides the optional `next` pointer, a switch block must contain code blocks for *true* and *false* branches of execution. A data structure for a switch block may look like so:
```
Switch {
    true_branch  : CodeBlock,
    false_branch : CodeBlock,
    next?        : ControlBlock,
}
```
Execution semantics of a switch block are as follows:
* If the top of the stack is `1`, `true_branch` is executed; if the top of the stack is `0`, `false_branch` is executed.
* Then, if `next` pointer is not null, execution moves to the next block.

Switch block imposes the following restrictions on its content:
* `true_branch` must start with an `ASSERT` instruction. This guarantees that this branch can be executed only if the top of the stack is `1`.
* `false_branch` must start with `NOT ASSERT` instruction sequence. This guarantees that this branch can be executed only if the top of the stack is `0`.

#### Loop block
A loop block is used to describe a sequence of instructions which is to be repeated zero or more times based on some condition (i.e. *while* statement). Besides the optional `next` pointer, a loop block must contain a code block describing the contents of the loop. A data structure for a loop block may look like so:
```
Loop {
    content : CodeBlock,
    next?   : ControlBlock,
}
```
Execution semantics of a loop block are as follows:
* If the top of the stack is `1`, `content` block is executed.
  * If after executing `content` block, the top of the stack is `1`, `content` block is executed again. This process is repeated until the top of the stack is `0`.
  * Then, if `next` pointer is not null, execution moves to the next block.
* If the top of the stack is `0`, `NOT ASSERT` sequence of instructions is executed.
  * Then, if `next` pointer is not null, execution moves to the next block.

Loop block imposes the following restrictions on its content:
* `content` must start with an `ASSERT` instruction. This guarantees that a loop iteration can be entered only if the top of the stack is `1`.

It is expected that at the end of executing `content` block, the top of the stack will contain a binary value (i.e. `1` or `0`). However, this is not enforced at program construction time, and if the top of the stack is not binary, the program will fail at execution time.

### Code blocks
A code block consists of a sequence of instructions to be executed and an optional `next` pointer which can be used to specify the next control block in a program.  A data structure for a code block may look like so:
```
CodeBlock {
    operations : Vector<u128>,
    next?      : ControlBlock,
}
```
Execution semantics of a code block are as follows:
* All `operations` of the block are executed first.
* Then, if `next` pointer is not null, execution moves to the next block.

Code block imposes the following restrictions on its content:
* `operations` vector must have a length which is a multiple of 16, and must contain at least 16 instructions.

Having an optional `next` pointer enables nesting of control blocks. For example, a control block can contain a code block, which, in turn, points to another control block and so on. Concrete examples of this are explored in the following section.

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
* The `next` pointer of this block is null. This is indicated by `∅` in the top right corner of the rectangle.
* The inner rectangle represents a code block with instructions a<sub>0</sub> . . . a<sub>i</sub>, and `next` pointer set to `∅`.

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

Here, we have 3 control blocks, one describing the initial sequence of instructions, another one describing the *if/else* statement, and the last block describing the final sequence of instructions. The blocks are linked via `next` pointers like so:

* The `next` pointer of the code block within B<sub>0</sub> block points to B<sub>1</sub> block.
* The `next` pointer of the B<sub>1</sub> block points to B<sub>2</sub> block directly.

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

Here, we have 5 control blocks, where blocks B<sub>2</sub> and B<sub>3</sub> are nested within the *else* branch of block B<sub>1</sub>.

## Program hash
All Distaff programs can be reduced to a 16-byte hash represented by a single element in a 128-bit field. The hash is designed to target 128-bit preimage and second preimage resistance, and 64-bit collision resistance.

Program hash is computed incrementally. That is, it starts out as `0`, and with every consecutive program block, program hash is updated to include hash of that block.

For example, let's say our program consists of 3 control blocks and looks like so:

<p align="center">
    <img src="assets/prog_hash1.dio.png">
</p>

Let's also define hash of block B<sub>n</sub> as *hash(B<sub>n</sub>)*, and hash of the program before block *B<sub>n</sub>* is "merged" into it as *h<sub>n</sub>*. Then, hash of the entire program is computed like so:

1. First, we set *h<sub>0</sub> = 0*.
2. Then, we compute *h<sub>1</sub> = hash_acc(h<sub>0</sub>, hash(B<sub>0</sub>))*, where *[hash_acc]((#hash_acc-procedure))* is a procedure for merging block hashes into program hash.
3. Then, we compute *h<sub>2</sub> = hash_acc(h<sub>1</sub>, hash(B<sub>1</sub>))*.
4. Finally, we compute *h<sub>3</sub> = hash_acc(h<sub>2</sub>, hash(B<sub>2</sub>))*, which is our program hash.

Graphically, this looks like so:

<p align="center">
    <img src="assets/prog_hash2.dio.png">
</p>

This is a Merkle tree (though lopsided). In fact, all program hashes are roots of Merkle trees, where the shape of the tree is defined by program structure. This is by design. Using this property of program hashes, we can selectively reveal any of the program blocks while keeping the rest of the program private (e.g. secret programs with public pre/post conditions).

### Computing hash of a program block
In the section above, we described how to compute hash of an entire program from hashes of individual program blocks. In this section, we'll describe how to compute hashes for different types of program blocks.

It is worth noting upfront that hashes of control blocks differ from hashes of code blocks in how they are computed and how they are represented. Specifically:
1. Hashes of control blocks are defined as tuples of two 128-bit elements *(v<sub>0</sub>, v<sub>1</sub>)*.
2. While hashes of code blocks are defined as a single 128-bit element *v*.

#### Hashes of control blocks
Hashes of control blocks are just hashes of underlying code blocks arranged in specific ways.

For **group blocks**, this arrangement looks like so:
* *v<sub>0</sub> = hash(content)*
* *v<sub>1</sub> = 0*

For **switch blocks**, we define the hash to be a hash of both branches simultaneously:
* *v<sub>0</sub> = hash(true_branch)*
* *v<sub>1</sub> = hash(false_branch)*

For **loop blocks**, we also define the hash to be a hash of its content and of a static *skip* block:
* *v<sub>0</sub> = hash(content)*
* *v<sub>1</sub> = hash(skip)*

where, `skip` block is defined as a code block containing `NOT ASSERT` sequence of instructions.

A side note: *hash(content)* is called a loop image as it binds each iteration of the loop to a specific hash.

#### Hashes of code blocks
Recall that a code block has the following form:
```
CodeBlock {
    operations : Vector<u128>,
    next?      : ControlBlock,
}
```
Hash of a code block is computed as follows:

1. First, we compute hash of instructions contained in `operations` vector. To do this, we use *[hash_ops](#hash_ops-procedure)* procedure. The output of this procedure is a single 128-bit element. If `next` pointer is null, we are done, and this element is returned as hash of the entire block.
2. If `next` pointer is not null, we compute hash of the control block specified by the pointer according to the rules described in the previous section. We than use [hash_acc](#hash_acc-procedure) procedure to merge this hash with the hash of `operations` we computed in the previous step.

Or described another way:
* *v = hash_ops(operations)*, when `next` is null;
* *v = hash_acc(hash_ops(operations), hash(next))*, when `next` is not null.

### hash_acc procedure
The purpose of *hash_acc* procedure is to merge hash of a control block into the running program hash. Recall that hash of a control block is represented by two 128-bit elements, while hash of the program is represented by one 128-bit element. The output of *hash_acc()* is a single 128-bit element.

Denoting *(v<sub>0</sub>, v<sub>1</sub>)* to be the hash of the control block to be merged, and *h* to be the current hash of the program, high-level pseudo-code for *hash_acc()* function looks like so:
```
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
The above is a modified version of [Rescue](https://eprint.iacr.org/2019/426) hash function. This modification adds half-rounds to the beginning and to the end of the standard Rescue hash function to make the arithmetization of the function fully foldable. It should not impact security properties of the function, but it is worth noting that it has not been studied to the same extent as the standard Rescue hash function.

### hash_ops procedure
The purpose of *hash_ops()* function is to hash a sequence of instructions into a single 128-bit element. The pseudo-code for this function is as follows:

```
let state = [0, 0, 0, 0];
for each op_code in instructions do:
    state = hash_ops_round(state, op_code);
return state[0];
```
where:
  * `state` is an array of four 128-bit field elements,
  * `instructions` is a vector of 128-bit field elements,
  * `hash_ops_round()` is a function which merges `op_code` into the state. The specifics of this function are currently TBD.

## Program hash computations
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