# Project Guidelines for ComChan

This document outlines the essential commands and code style guidelines for the ComChan project.

## Build/Lint/Test Commands

*   **Build:** `cargo build`
*   **Run:** `cargo run`
*   **Lint:** `cargo clippy`
*   **Test All:** `cargo test`
*   **Run Single Test:** `cargo test <test_name>` (e.g., `cargo test my_specific_test`)
*   **Format:** `cargo fmt`

## Code Style Guidelines

*   **Formatting:** Use `cargo fmt` to automatically format Rust code.
*   **Imports:** Organize `use` statements for clarity. Prefer explicit imports over glob imports (`use crate::module::*`) unless necessary.
*   **Naming Conventions:**
    *   `snake_case` for function and variable names (e.g., `my_function`, `my_variable`).
    *   `PascalCase` for type names (structs, enums, traits), and modules (e.g., `MyStruct`, `MyEnum`, `MyModule`).
    *   `SCREAMING_SNAKE_CASE` for constants (e.g., `MY_CONSTANT`).
*   **Error Handling:** Prefer Rust's `Result<T, E>` and `Option<T>` enums for error handling and optional values, respectively. Utilize the `?` operator for concise error propagation.
*   **Comments:** Add comments sparingly, focusing on _why_ a piece of code exists or its purpose, rather than _what_ it does.
*   **Modularity:** Organize code into logical modules and files to improve readability and maintainability.
=======
# ComChan CRUSH.md

This document provides conventions for the ComChan codebase.

## Build, Lint, and Test

- **Build:** `cargo build`
- **Run:** `cargo run -- [args]`
- **Test:** This project does not have an automated test suite. Manual testing is performed using the Arduino sketches in the `code_tests` directory.
- **Lint:** `cargo clippy`
- **Format:** `cargo fmt`

## Code Style

- **Formatting:** Use `rustfmt` for all code.
- **Imports:** Group imports in the following order: `std`, external crates, and internal modules.
- **Types:** Use specific integer types (`u32`, `usize`, etc.) instead of generic types like `int`. All public functions and structs should have explicit types.
- **Naming:**
    - Structs: `PascalCase`
    - Functions and variables: `snake_case`
    - Constants: `SCREAMING_SNAKE_CASE`
- **Error Handling:** Use `Result` for functions that can fail. Use the `?` operator for propagating errors. Add context to errors using `map_err`.
- **Panics:** Avoid panicking. Return a `Result` instead.
- **Comments:** Add comments to explain complex or non-obvious logic.

## Commits and Pull Requests

- **Commit Messages:** Follow the Conventional Commits specification.
- **Pull Requests:** Ensure that all checks pass before merging. Provide a clear description of the changes and link to any relevant issues.
>>>>>>> add_paper
