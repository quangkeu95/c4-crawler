# Code4rena crawler
Get all the ongoing and upcoming contests on [code4rena](https://code4rena.com/). Then find all the contracts with their name and bytecode.

# Run
Run the main process:
```bash
cargo run
```

## TODO
- Make crawler run concurrently to crawl contests faster.
- Make `forge build` process running concurrently. 
- Add more testings.
- Clear the artifacts after build if needed.
- Able to detect contest repo is using `hardhat` or `foundry`.
- Parse `foundry.toml` more efficient. `foundry` repo already did this so will dive deeper when i have time.
- Solc compile without cloning the repo. The idea is to scape all the source files with imported dependencies being resolved, thus 
making the compile process easier.