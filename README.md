# RLottery
## Introduction
RLottery is an efficient yet fully functional basic Lotto game engine. The main ideas are as follows:
1. Figure out efficient "core structure" of a Lotto game engine
2. Study efficient database structure for storing multi-tenant lottery data
3. Find out whether Rust is a good technology choice for Lotto engine
4. Figure out capabilities of (remote) AI agents in the context of this project
## Requirements
### Functional requirements
General
- The solution can be configured to run one lottery game for one lottery operataor at a time
  - We can run multiple instances of the engine for different lottery operators. Lottery operator id needs always to be specified
- No personal information is stored in the database. A technical user id can ba stored.
- Supports "Lottolike" games only. Lottolike games have constant stake.
- Arbitrary number of draw levels can be configuered and named
  - Typically draw levels are named "primary", "secondary", "tertiary" and so on
  - For example, we can have 6 primary numbers between 1 and 40 and 1 secondary number between 1 and 40
  - Draw levels can be dependent on previous levels, or be independent
    - In previous example the 1 secondary number is dependent on the 6 primary numbers (i.e they are drawn from the same number space)
    - In some other game we might have 14 primary numbers between 1 and 49, 1 secondary number between 1  and 4 independent of previous level, and tertiary numbers between 1 and 2 independent of previous levels
- Game types need to be configured and named, within limits given by draw levels
  - In Lottolike games
    - normal wagers contain primary draw level number of selections
    - system wagers contain more than primary draw level number of selections
- Support both random drawing (within the lottery engine) and external drawing (tombola or some other mechanism)

Wagering
- Support normal and system game types
- Support quick picks (i.e lottery engine picks the remaining numbers up to number according to game type)
- Support multiple normal boards in one wager
- Draws in which the wager will participate need to be explicitly specified
  - The engine can be configured to allow a participate in in a specified number of draws. We can for example configure that players are allowed to participate in 1, 2, 3, 4, 5, 6, 7 or 14 draws.
  - It can be configured whether first draw can be skipped or not

Drawing
- A configured number of draws can be open at any given time
  - The number needs to be one bigger than the maximum number of draws in which the wager can participate
  - When a draw is created, it is not yet open, it is in "Created" state. That is, we can technically have draws that exist but are not yet open.
- The engines supports calendar based scheduling and inteval based scheduling
  - Calendar based example: a draw every Tuesday 21.30 and every Friday 22.00
- Draw has following states
  - Created: A draw has been technically created, but is not yet open
  - Open: A draw is open for wagering
  - Closed: A draw is no longer open for wagering
  - Drawn: Numbers have been drawn (or received)
  - Winset calculated: Winset has been calculated, i.e winning boards are known (but not yet necessarily win sums)
  - Winset confirmed: Winset has been confirmed, i.e winning boards and win sums are known
  - Finalized: Draw has been finalized - typically all account related activities such as win payments are done
  - Cancelled: Draw has been cancelled, and all stakes have been returned to users
- Draws have following timestamps
 - Created timestamp: The time when the draw was created
 - Modified timestamp: The time when the draw was last modified
 - Open timestamp: The time when the draw is open for wagering
 - Closed timestamp: The time when the draw is closed, i.e is no longer open for wagering
 - Drawn timestamp: The time when numbers have
 - Winset calculated timestamp: The time when winning boards have been calculated
 - Winset confirmed timestamp: The time when winning boards and their win sums have been confirmed
- Closed state duration can be configured
- Games rules and Win class defintions dictate whether we can
  - Move from Closed to Drawn (if game is externally drawn, we need to wait for these, if randomly drawn the engine can do the drawing)
  - Move from Winset claculated to Winset confirmed (if we have external win classes, we need to wait for draw's winclass related data, and confirmation)

Win class definition
- A win class can be factor based, constant, percentage based, or externally set
  - Factor: Winners get their stake returned by a factor
  - Constant: Winners get a constant win
  - Percentage based: Winners get a percentage of draw turnoven
  - External: Win class total win sum is set externally
    - The win sum is shared by all winners of the win class
    - This covers a lot of cases such as percentage based win classes (where win sum is based on draw turnover)
- Win classes can be min and/or max capped
  - Min capping: Win class total win sum is at least a defined amount
  - Max capping: Win class total win sum is at maximum a defined amount

Winset calculation
- Need to support arbitrary number of wagers in draw (i.e don't load everything into memory during calculation)
- Game engine initially calculates winset, but does not yet calculate win sums
  - If game rules have "exteranal" win classes, we need to wait for draw's winclass related information, and confirmation before calculating win sums
  - If there are no no "external" win classses, the engine can proceed to 

### Technical requirements:
- Use Rust
- Use Postgres database. Create all necessary indices. One database must support multiple lottery operators simultaneously.
  - Maximum database capacity if 8 vCPU and 32 Gigbytes of RAM
- Use UUIDv7 for user, wager, win and board win id. 
- Use integer for draw ID
- Select suitable ID for boards and selections, etc.
- The solution needs to be scalable to following scale
  - Writing wagers at least 1000 requests per second
  - Reading wagers at least 1000 requests per second at the same time that wagers are written
  - Support 100 million wagers per draw, with winset calculation in less than 1 minute
  - You can use multiple virtual server (Kubernetes pods or whatever) to achieve this performance, but you are only allowed to use one database server
- Support scheduled tasks
  - Each day, delete closed draws and wagers that are older than 30 days
- Use  tool for handling database migrations (such as Flyway or Liquibase)
- Do not use ORM, just implement required database helper functions
- Use gRPC in APIs
- Enable transactional "extension points" for all draw and wager state changes so that necessary operations (such as wallet operations or logging) can be added later. Implement default exension point implementations that do audit logging to database. Audit logs should contain at least the JSON representation of the changed object.
- Use Xoshiro512** as pseudorandom number generator
- Implement unit tests for core operations, and integration tests that test the system from API perspective
