# Contributing to ComChan

We welcome contributions to the ComChan project! Before you start, please take a moment to read these guidelines.

## Getting Started

To set up your development environment, you'll need to have Rust and Cargo installed. If you don't have them already, you can install them via `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

For Arduino-related contributions, you'll need the Arduino IDE and the necessary board packages installed.

## Build, Lint, and Test

We use Cargo for managing our Rust project. Here are the common commands:

*   **Build:** `cargo build`
*   **Run:** `cargo run`
*   **Lint:** `cargo clippy`
*   **Test All:** `cargo test`
*   **Run Single Test:** `cargo test <test_name>` (e.g., `cargo test my_specific_test`)
*   **Format:** `cargo fmt`

## Code Style Guidelines

Please adhere to the following code style guidelines to maintain consistency across the project:

*   **Formatting:** Always run `cargo fmt` before submitting your changes.
*   **Imports:** Organize `use` statements for clarity. Prefer explicit imports over glob imports (`use crate::module::*`) unless necessary.
*   **Naming Conventions:**
    *   `snake_case` for function and variable names (e.g., `my_function`, `my_variable`).
    *   `PascalCase` for type names (structs, enums, traits), and modules (e.g., `MyStruct`, `MyEnum`, `MyModule`).
    *   `SCREAMING_SNAKE_CASE` for constants (e.g., `MY_CONSTANT`).
*   **Error Handling:** Prefer Rust's `Result<T, E>` and `Option<T>` enums for error handling and optional values, respectively. Utilize the `?` operator for concise error propagation.
*   **Modularity:** Organize code into logical modules and files to improve readability and maintainability.
*   **Comments:** Add comments sparingly, focusing on _why_ a piece of code exists or its purpose, rather than _what_ it does.

## Submitting Changes

1.  **Fork the repository.**
2.  **Create a new branch** for your feature or bug fix: `git checkout -b feature/your-feature-name` or `git checkout -b bugfix/your-bug-name`.
3.  **Make your changes**, ensuring they follow the code style guidelines.
4.  **Test your changes** thoroughly.
5.  **Commit your changes** with a clear and concise commit message.
6.  **Push your branch** to your forked repository.
7.  **Open a Pull Request** to the `main` branch of the original repository. Describe your changes clearly and explain their purpose.

Thank you for contributing!
