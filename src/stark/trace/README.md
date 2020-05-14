# Execution trace
A program's execution trace is a two-dimensional matrix which holds the result of executing the program on Distaff VM.

The state of the VM can be thought of as a set of registers. Every program instruction takes the current state as an input and produces a new state as the output. Thus, columns in the matrix correspond to state registers, and rows record how the values in these registers change from one step of the program to the next. Specifically:

<p align="center">
Row<sub>i</sub> = (r<sub>i, 0</sub>, r<sub>i, 1</sub>, . . . , r<sub>i, k - 1</sub>)
</p>

where:
* *Row<sub>0</sub>* . . . Row<sub>n-1</sub> are the rows in the trace table of length *n*.
* *r<sub>0</sub> ... r<sub>k-1</sub>* are the VM registers, and *r<sub>i, k</sub>* represent value of register *k* at row *i*.

## Trace polynomials
We can also think of each register trace as an evaluation of some polynomial such that:

<p align="center">
T<sub>k</sub>(x<sub>i</sub>) = r<sub>i, k</sub>
</p>

where:
* *x<sub>i</sub> = ω<sup>i</sup><sub>trace</sub>* for all *i* in the trace domain (see more about domains [here](..)).

And so, a trace table can be thought of as a set of polynomials *T<sub>0</sub>(x) . . . T<sub>k-1</sub>(x)*, which when evaluated over the trace domain produce values of all registers at all steps of computation.

## Trace registers
Currently, there are 2 sets of trace registers:

### 1. Decoder registers

TODO: describe decoder registers

### 2. Stack registers

TODO: describe stack registers