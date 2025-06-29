# RLottery Work Plan

This document outlines the work plan for implementing the RLottery engine.

## 1. Project Setup
- Initialize a new Rust project.
- Set up the basic project structure with modules for different components (e.g., `core`, `db`, `api`, `config`).
- Add initial dependencies to `Cargo.toml` (e.g., `tokio`, `tonic`, `postgres`, `serde`, `uuid`, `rand`).
- Create a `.gitignore` file.

## 2. Database Schema and Migrations
- Design the database schema based on the requirements in `README.md`.
- Create tables for:
    - `lottery_operators`
    - `games`
    - `draws`
    - `draw_levels`
    - `wagers`
    - `boards`
    - `selections`
    - `win_classes`
    - `winnings`
    - `audit_logs`
- Choose and set up a database migration tool (e.g., `sqlx-cli`, `refinery`).
- Create the initial database migration scripts.

## 3. Core Data Structures and Logic
- Implement the core data structures in Rust (e.g., `Draw`, `Wager`, `Board`, `Selection`, `WinClass`).
- Implement the business logic for creating and validating these structures.
- Implement the Xoshiro512** pseudorandom number generator for quick picks and random draws.

## 4. Configuration Module
- Implement a module to load and manage lottery operator and game configurations from a file (e.g., TOML, YAML).
- This module will provide the necessary configuration to other parts of the application.

## 5. Wagering API
- Define the gRPC service for wagering in a `.proto` file.
- Implement the gRPC server for the wagering API.
- Implement the following RPCs:
    - `PlaceWager`: For placing normal and system wagers.
    - `GetWager`: For retrieving wager information.
- Implement support for quick picks.

## 6. Draw Management
- Implement the logic for managing draw states (Created, Open, Closed, Drawn, etc.).
- Implement the scheduling logic for both calendar-based and interval-based draws.
- Implement a state machine to manage draw state transitions.

## 7. Drawing Logic
- Implement the logic for drawing winning numbers using the Xoshiro512** PRNG.
- Implement the logic for receiving and processing externally drawn numbers.

## 8. Win Class Definition and Calculation
- Implement the logic for defining and configuring different types of win classes (factor, constant, percentage, external).
- Implement the logic for calculating win sums based on the win class definitions.

## 9. Winset Calculation
- Implement the efficient winset calculation logic.
- This should be able to handle a large number of wagers without loading everything into memory.
- The calculation should be performed in a background job.

## 10. Extension Points
- Implement the transactional extension points for all draw and wager state changes.
- Implement the default extension point implementations that do audit logging to the database.

## 11. Scheduled Tasks
- Implement the scheduled task for deleting closed draws and wagers that are older than 30 days.
- Use a library like `tokio-cron-scheduler` for this.

## 12. Testing
- Implement unit tests for all core operations.
- Implement integration tests that test the system from the API perspective.
- Use a test database for integration tests.

## 13. API Documentation
- Document the gRPC API in the `.proto` file.
- Provide examples of how to use the API.

## 14. Deployment
- Create a `Dockerfile` to containerize the application.
- Create `docker-compose.yml` for easy local development and testing.
- Provide scripts for building and running the application.
