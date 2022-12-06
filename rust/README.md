# Crafting Interpreters (Rust Edition)

This project follows the awesome [Crafting Interpreters Book](https://craftinginterpreters.com/),
which implements a tree-walk interprer in Java for the imaginary Lox programming
language. In the spirit of "why not", I'm implementing this in Rust.

This project tries to follow the code in the book as closely as possible. One key difference
is that I didn't use the visitor pattern or auto-generate AST classes because pattern matching in
Rust is more than sufficient.

**Warning**: In the spirit of making bad decisions,
I'm using this project to learn Rust for the first time. Really bad
Rust code lies ahead! You've been warned!
