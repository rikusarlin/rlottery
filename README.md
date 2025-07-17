# RLottery
## Introduction
RLottery is an efficient yet fully functional basic Lotto game engine. The main ideas are as follows:
1. Figure out efficient "core structure" of a Lotto game engine
2. Study efficient database structure for storing multi-tenant lottery data
3. Find out whether Rust is a good technology choice for Lotto engine
4. Figure out capabilities of (remote) AI agents in the context of this project

For functional and technical requirements, see [REQUIREMENTS.md](REQUIREMENTS.md)

## Running RLottery locally
Pull Postgres 17 image:
```
docker pull postgres:17
```
Start container:
```
docker run --name rlottery-postgres-17 -e POSTGRES_PASSWORD=password123 -e POSTGRES_USER=rlottery -e POSTGRES_DB=rlottery -p 5432:5432 -d postgres:17
```
Build project:
```
cargo build
```
Run project:
```
./target/debug/rlottery
```

## Testing
Currently we only have integration tests. Tests should work when run in parallel, too, but in case 
they don't you can try the following to run the tests one at a time: 

```
RUST_TEST_THREADS=1 cargo test
```
