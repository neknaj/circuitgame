# Neknaj Circuit Game / Neknaj Circuit Game Language

This document provides a precise specification of Neknaj Circuit Game Language syntax to avoid common misunderstandings.

## Neknaj Circuit Game

[Web Page](https://neknaj.com/circuitgame/README)  
[Github Repository](https://github.com/neknaj/circuitgame)  
[Web Playground (CircuitGame Web Tool)](https://neknaj.github.io/circuitgame/)  

## Important Syntax Limitations

1. This is NOT a general-purpose programming language
2. There are NO:
   - Arrays or lists
   - Structs or objects
   - Variables (except gate connections)
   - Loops or conditionals
   - Functions (only modules with fixed inputs/outputs)

## Basic Structure

A circuit file consists of these elements in any order:
1. One `using nor:2->1;` declaration
2. Zero or more module definitions
3. Zero or more test definitions

### Module Definition Syntax

```ncg
module name (input1 input2)->(output1) {
    wire1: gate_name <- input1 input2;
    output1: another_gate <- wire1 input2;
}
```

OR

```ncg
func name (input1 input2)->(output1) {
    wire1: gate_name <- input1 input2;
    output1: another_gate <- wire1 input2;
}
```

Key points:
- Parentheses are required even with no inputs/outputs
- Each line inside must be a gate definition
- All identifiers must be simple names (no arrays or paths)

### Gate Definition Syntax

```ncg
output_name: module_name <- input1 input2;
```

OR (multiple outputs)

```ncg
out1 out2: module_name <- in1 in2;
```

Key points:
- Each gate must have exactly this format
- No expressions or calculations
- Names must be single identifiers
- Number of inputs/outputs must match the module definition
- The arrow `<-` can be omitted, but its inclusion is recommended.
- Must end with semicolon
- Spaces around separators are optional

### What's NOT Valid

❌ These are NOT supported:
```ncg
// No arrays
outputs[0]: nor <- inputs[0] inputs[1];

// No member access
module.output: nor <- a.input b.input;

// No nested definitions
a: (nor <- b c): nor <- d e;

// No gate parameters
mygate<T>: nor <- in1 in2;
```

## Examples of Valid Code

### Valid NOT Gate
```ncg
module not (x)->(y) {
    y: nor <- x x;
}
```

### Valid Half Adder
```ncg
module halfadder (a b)->(sum carry) {
    carry: and <- a b;
    sum: xor <- a b;
}
```

### Valid Test Pattern
```ncg
test not:1->1 {
    t -> f;
    f -> t;
}
```

## Function Modules vs Non-Function Modules

This language has two types of modules: function modules (defined with `func`) and non-function modules (defined with `module`).

### Function Modules (func)

Function modules have strict ordering requirements:
- All inputs must be used in a forward-only manner
- Values must be defined before they can be used
- Can only call other function modules
- Best for combinational logic circuits

Example:
```ncg
func and (x y)->(out) {
    a: not <- x;    // Valid: using input x
    b: not <- y;    // Valid: using input y
    out: nor <- a b;  // Valid: using previously defined wires
}
```

Invalid function module:
```ncg
func invalid (x)->(out) {
    out: nor <- x b;  // ERROR: using 'b' before definition
    b: not <- x;
}
```

### Non-Function Modules (module)

Non-function modules are more flexible:
- Can use values before they are defined
- They can participate in circular dependencies, such as those found in feedback loops.
- Can call both function and non-function modules
- Suitable for sequential circuits and feedback loops

Example:
```ncg
module sr_latch (s r)->(q nq) {
    q: nor <- r nq;   // Valid: using 'nq' before definition
    nq: nor <- s q;   // Creates a feedback loop
}
```

## Module Calling Rules

### Important Restrictions
1. Function modules (`func`) can only call other function modules
2. Non-function modules (`module`) can call both types of modules
3. Violating these rules will result in compilation errors

```ncg
// WRONG: Function module trying to use a non-function module
func wrong (x)->(y) {
    y: sr_latch <- x x;  // Error: Cannot use sr_latch (non-func) in a func module
}

// Correct usage
func correct (x)->(y) {
    y: not <- x;  // OK: 'not' is a function module
}
```

### Usage Guidelines

- Use `func` for:
  - Combinational logic (AND, OR, XOR gates)
  - Arithmetic circuits (adders, multipliers)
  - Any circuit without feedback loops

- Use `module` for:
  - Sequential logic
  - Memory elements (latches, flip-flops)
  - Circuits with feedback loops
  - Clock generators
  - Oscillators

## Common Mistakes to Avoid

1. Trying to use arrays:
   ```ncg
   // WRONG:
   bits[8]: register <- data[8];
   
   // Correct:
   bit0 bit1 bit2 bit3: register <- data0 data1 data2 data3;
   ```

2. Trying to use nested structures:
   ```ncg
   // WRONG:
   alu.add.result: adder <- a.value b.value;
   
   // Correct:
   add_result: adder <- a_value b_value;
   ```

3. Trying to use expressions:
   ```ncg
   // WRONG:
   sum: adder <- (a + b) c;
   
   // Correct:
   temp: adder <- a b;
   sum: adder <- temp c;
   ```

## Valid Identifier Names

- Must start with a letter or underscore
- Can contain letters, numbers, and underscores
- Case-sensitive
- Cannot be keywords

Examples:
- ✅ `result`
- ✅ `tmp_1`
- ✅ `carry_out`
- ❌ `0bit`
- ❌ `my.wire`
- ❌ `data[0]`

## Testing Syntax

Test patterns must follow this exact format:
```ncg
test module_name:inputs->outputs {
    t f -> t;    // inputs -> expected_outputs
    f t -> f;
}
```

Important restrictions:
- Tests can ONLY be written for function modules (`func`)
- Tests CANNOT be written for non-function modules (`module`)
- This is because non-function modules may contain feedback loops and state, making their behavior time-dependent

Example of what's allowed and what's not:
```ncg
// ALLOWED: Testing a function module
func and (x y)->(z) {
    // ...implementation...
}
test and:2->1 {
    t t -> t;
    t f -> f;
}

// NOT ALLOWED: Cannot test non-function modules
module sr_latch (s r)->(q nq) {
    // ...implementation...
}
test sr_latch:2->2 {  // ERROR: Cannot test non-function modules
    t f -> t f;
}
```

Where:
- `t`, `T`, `1`, `h`, `H` represent true
- `f`, `F`, `0`, `l`, `L` represent false
- Spaces between values are required
- Arrow `->` is required
- Each pattern must end with semicolon

## Template Code Examples

### Basic Gates
```ncg
using nor:2->1;

// NOT gate
func not (x)->(out) {
    out: nor <- x x;
}

// AND gate
func and (x y)->(out) {
    a: not <- x;
    b: not <- y;
    out: nor <- a b;
}

// OR gate
func or (x y)->(out) {
    a: nor <- x y;
    out: not <- a;
}
```

### Memory Elements
```ncg
// SR Latch (must be non-function module due to feedback)
module sr_latch (s r)->(q nq) {
    q: nor <- r nq;  // Uses nq before definition
    nq: nor <- s q;  // Creates feedback loop
}

// D Latch using SR Latch
module d_latch (d clk)->(q nq) {
    nd: not <- d;
    s: nor <- nd clk;
    r: nor <- d clk;
    q nq: sr_latch <- s r;
}
```

### Arithmetic Circuits
```ncg
// Half Adder
func half_adder (a b)->(sum carry) {
    carry: and <- a b;
    sum: xor <- a b;
}

// Full Adder using Half Adder
func full_adder (a b cin)->(sum cout) {
    s1 c1: half_adder <- a b;
    s2 c2: half_adder <- s1 cin;
    c3: or <- c1 c2;
    sum: buf <- s2;
    cout: buf <- c3;
}
```

### Sequential Circuits
```ncg
// Clock Generator (requires feedback)
module clock()->(clk) {
    clk: not <- nclk;
    nclk: not <- clk;
}

// Counter with Reset
module counter (reset)->(out) {
    next: increment <- current;
    current: d_latch <- next reset;
    out: buf <- current;
}
```

### Test Patterns
```ncg
// Basic gate test
test not:1->1 {
    t -> f;
    f -> t;
}

// Multiple input test
test and:2->1 {
    t t -> t;
    t f -> f;
    f t -> f;
    f f -> f;
}

// Multiple output test
test half_adder:2->2 {
    t t -> t t;  // sum,carry
    t f -> t f;
    f t -> t f;
    f f -> f f;
}
```

## Comments

### Line Comments

The language supports single-line comments using double forward slashes:
```ncg
// This is a comment
```

Key points about comments:
- Comments start with `//` and continue to the end of the line
- Comments are completely ignored by the compiler
- Empty lines and whitespace are also ignored
- Block comments (`/* ... */`) are NOT supported
- Hash (`#`) comments are NOT supported

### Comment Placement

Comments can be placed in the following locations:

1. At the file level:
   ```ncg
   // File description
   using nor:2->1;
   
   // Module definitions below
   module example...
   ```

2. Between module/test definitions:
   ```ncg
   module not (x)->(y) {
       y: nor <- x x;
   }
   
   // AND gate implementation
   module and (x y)->(z) {
       z: nor <- x y;
   }
   ```

3. Inside module/test bodies:
   ```ncg
   module full_adder (a b cin)->(sum cout) {
       // First compute partial sum
       s1 c1: half_adder <- a b;
       
       // Then add carry in
       s2 c2: half_adder <- s1 cin;
       
       // Combine carries
       cout: or <- c1 c2;
       sum: buf <- s2;
   }
   ```

4. At the end of lines:
   ```ncg
   module example (in)->(out) {
       a: not <- in;    // Invert input
       out: buf <- a;   // Buffer the result
   }
   ```

### What's NOT Supported

❌ These comment styles are NOT supported:
```ncg
/* Block comments using slash-star are not supported */

# Hash comments are not supported

/// Documentation comments are not special
//! Module documentation is not special

--  Different comment markers are not supported
''' Python-style comments are not supported
```

## Error Messages and Troubleshooting

The compiler performs several validation checks and may produce the following errors:

### Module Definition Errors

1. **Duplicated Module Names**
   ```
   Error: Defined module name Duplicated: module_name
   ```
   - Each module name must be unique
   - Check for modules with the same name
   - Rename one of the conflicting modules

2. **Undefined Module Usage**
   ```
   Error: Undefined module used: module_name in parent_module
   ```
   - You're trying to use a module that hasn't been defined
   - Make sure the module is defined before use
   - Check for typos in module names

### Gate Connection Errors

1. **Duplicated Identifiers**
   ```
   Error: Defined id Duplicated: Input wire_name in module_name
   Error: Defined id Duplicated: Gate-Out wire_name in module_name
   ```
   - Each wire name must be unique within a module
   - Rename duplicate wire identifiers
   - Check input parameters and gate outputs

2. **Undefined Identifiers**
   ```
   Error: Undefined id used: Gate-In wire_name in module_name
   Error: Undefined id used: Output wire_name in module_name
   ```
   - You're trying to use a wire that hasn't been defined
   - Make sure all inputs are defined
   - Check for typos in wire names

### Function Module Specific Errors

1. **Forward Reference Error**
   ```
   Error: In a function module, a value cannot be used before it is declared: wire_name in module_name
   ```
   - Function modules require strict ordering
   - Move the wire definition before its usage
   - Consider using a non-function module if feedback is needed

2. **Invalid Module Usage**
   ```
   Error: Function modules cannot call non-function modules: module_name used in func_name
   ```
   - Function modules can only use other function modules
   - Either change the called module to a function module
   - Or change the calling module to a non-function module

### Type Errors

1. **Mismatched Gate Types**
   ```
   Error: Used module with unmatched type: module_name expected 2->1 but got 3->1, in parent_module
   ```
   - Number of inputs/outputs doesn't match the module definition
   - Verify the number of inputs and outputs for the module 
   - Make sure you're providing the correct number of connections

### Dependency Errors

1. **Circular Dependencies**
   ```
   Error: Cycle detected in the graph, sorting cannot be completed.
   ```
   - Modules have circular dependencies
   - Identify the dependency cycle
   - Break the cycle

### Warning Messages

1. **Unused Modules**
   ```
   Warning: Module has no dependency: module_name
   ```
   - Module is defined but never used
   - Either remove unused modules
   - Or add them to your circuit

2. **Multiple Root Modules**
   ```
   Warning: Multiple modules are not used by other modules: module1, module2, module3
   ```
   - Multiple modules are at the top level
   - This is allowed but might indicate design issues
   - Consider if this is intentional

3. **Module has no Test**
   ```
   Warning: test No test provided for module: nor
   ```
   - The nor module is defined, but it is not used within any other defined module or tested in the test section.
   - Write test for the `nor` module.

4. **Module Test Failed**
   ```
   Warning: test Test failed: module buf input [true], expected [true] but got [false]
   ```
   - The test you wrote for the buf module didn't pass. Specifically, when you gave the buf module a true input, you expected it to output true, but it actually output false.
   - Either the buf module is not working correctly and needs to be fixed, or
   - You made a mistake when you wrote the test and should correct the test pattern.
   - The two above action must be performed until the test passed.

### Best Practices for Error Prevention

1. **Plan Module Structure**
   - Decide which modules need to be function modules vs non-function modules
   - Draw dependency diagrams before implementation
   - Keep modules small and focused

2. **Naming Conventions**
   - Use descriptive, unique names for modules
   - Use consistent naming patterns for wires
   - Ensure numeric suffixes are sequential and represent a logical ordering

3. **Type Checking**
   - Verify input/output counts before implementing
   - Document module interfaces
   - Test modules individually

4. **Dependency Management**
   - Keep dependencies unidirectional when possible
   - Use function modules for pure combinational logic
   - Designate non-function modules for sequential circuits
