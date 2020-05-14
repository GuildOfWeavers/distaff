# AIR constraints

AIR for Distaff VM consists of 2 high-level categories of constraints: *boundary constraints* and *transition constraints*. You can find evaluation logic for these constraints in the [evaluator](/evaluator.rs) module and related sub-modules. The specifics of these constraints are described below.

## Boundary constraints
Boundary constraints enforce that specific registers contain specific values at given steps of the execution trace. In Distaff VM, all boundary constrained are applied either to the first or to the last step of the computation.

Currently, there are 3 sets of boundary constraints:

### 1. Input constraints
Input constraints enforce the state of the stack at the beginning of the execution trace and are computed using the following expression:

<p align="center">
<img src="https://render.githubusercontent.com/render/math?math=\large C_k(x)=\frac{T_k(x)-v_k}{x-1}">
</p>

where:
* *k* is the index of the stack register against which the constraint is applied,
* *v* is the value that the register must have at the beginning of the execution trace,
* *x = ω<sup>i</sup><sub>ev</sub>* for all *i* in the constraint evaluation domain (see more about domains [here](..)).

### 2. Output constraints
Output constraint are similar to input constraints but enforce the state of the stack at the end of the execution trace. They are computed using the following expression:

<p align="center">
<img src="https://render.githubusercontent.com/render/math?math=\large C_k(x)=\frac{T_k(x)-v_k}{x-\omega_{trace}^{n-1}}">
</p>

where:
* *k* is index of the stack register against which the constraint is applied,
* *v* is the value that the register must have at the end of the execution trace,
* *n* is the length of the execution trace,
* *x = ω<sup>i</sup><sub>ev</sub>* for all *i* in the constraint evaluation domain (see more about domains [here](..)).

### 3. Program hash constraints
Program hash constraints enforce the hash value to which the executed program reduces by the end of the computation. Semantically, they are the same as output constraint, they are just applied to a different set of registers and enforce a different set of boundary values.

## Transition constraints
Transition constraints enforce that computation state changed correctly between two consecutive steps (except for the last step). They are computed using the following expression:

<p align="center">
<img src="https://render.githubusercontent.com/render/math?math=\large C_k(x)=\frac{F_k(x, T_0(x), \dots, T_m(x), T_0(x \cdot \omega_{trace}), \dots, T_m(x \cdot \omega_{trace}))}{(x^n-1)/(x-\omega_{trace}^{n-1})}">
</p>

where:
* *F<sub>0</sub> ... F<sub>k</sub>* are the transition constraint evaluation functions,
* *T<sub>0</sub> ... T<sub>m</sub>* are the trace polynomials,
* *n* is the length of the execution trace,
* *x = ω<sup>i</sup><sub>ev</sub>* for all *i* in the constraint evaluation domain (see more about domains [here](..)).

Currently, there are 2 sets of transition constraints:

### 1. Decoder constraints
TODO

### 2. Stack constraints
TODO