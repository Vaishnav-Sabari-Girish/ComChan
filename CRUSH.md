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
