# Circuit Game

![image](https://img.shields.io/badge/-TypeScript-103040.svg?logo=typescript&style=popout)
![image](https://img.shields.io/badge/-Rust-403540.svg?logo=rust&style=popout)

Circuit Game is a simulation tool for designing and testing digital circuits. It allows users to define modules, gates, and tests for their circuits and provides tools for compiling and testing these designs.

# Key Features
- DSL for describing logic circuits
- Transpile to TypeScript
- Development using both CLI and Web


# Installation

To install the project, follow these steps:

## Software required
- git
- rust, cargo
- node, npm

## Build CircuitGame CLI Tool
1. Clone  
    ```sh
    git clone https://github.com/neknaj/circuitgame
    cd circuitgame
    ```
2. Build  
    ```sh
    git pull
    cargo build --release
    # The built binary is: ./target/release/circuitgame_bin
    ```

## Build CircuitGame Web Tool

1. Clone:
    ```sh
    git clone https://github.com/neknaj/circuitgame.git
    cd circuitgame
    ```

2. Install dependencies:
    ```sh
    npm install
    ```

3. Build the project:
    ```sh
    node build.js
    ```

## Usage

This software is intended for development using both CLI and a Web Browser.  
The CLI tool has a file watcher and websocket communication.  

```sh
cargo run -- -i spec/sample.ncg -s 8080
# or: ncg -i spec/sample.ncg -s 8080
```

Click the link displayed in the console to view the results in your web browser.  

## Notation

The circuit notation grammar is defined using a BNF-like syntax.  

For a detailed BNF grammar, refer to the `lang.bnf` file in the `spec` directory.  
For the actual implementation, see the `parser.rs` file in the `src/compiler` directory.

### Sample Code

For more examples, see the `sample.ncg` file in the `spec` directory.
```ncg
using nor:2->1;

// This is a NOT gate module
module not (x)->(a) {
    a: nor <- x x;
}

// Providing tests for the not module
test not:1->1 {
    t -> f;
    f -> t;
}
```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## License

This project is licensed under the MIT License.