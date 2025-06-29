# Gemini Workspace

This file helps Gemini understand your project's context.

## About This Project

RLottery is an efficient yet fully functional basic Lotto game engine. The main ideas are to tigure out efficient "core structure" of a Lotto game engine, study efficient database structure for storing multi-tenant lottery data, find out whether Rust is a good technology choice for Lotto engine, and figure out capabilities of Google Gemini AI agent in the context of this project!

## Key Technologies

*   Language: Rust
*   Frameworks/Libraries: Tokio, Tonic, Serde, SQLx, Refinery, Tracing, Clippy
*   Build/Package Manager: Cargo
*   Test Framework: Rust built-in
*   Linter/Formatter: rustfmt

## Project Structure
Project follows standard Rust directory structure.

* src: Contains source code for the project.
* tests: Test code

## Important Commands

*   `cargo build`: Install dependencies and build
*   `./target/debug/rlottery`: Run the application.
*   `cargo test`: Run tests.
*   `cargo clippy`: Run the linter.
