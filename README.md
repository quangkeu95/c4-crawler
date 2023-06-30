# Contests Tracker
Fetch and compile all the public ongoing and upcoming contests on 
- [Code4rena](https://code4rena.com)
- [Sherlock](https://audits.sherlock.xyz/contests)
- [Immunefi](https://immunefi.com/explore/)
- [Blackhat](https://app.hats.finance/bug-bounties)

All the contests is stored at `contests` directory.

# Run
Run the main process:
```bash
cargo run
```

## TODO
- [x] Make crawler run concurrently to crawl contests faster.
- [ ] Make `forge build` process running concurrently. 
- [x] Add more testings (WIP).
- [ ] Clear the artifacts after build if needed.
- [x] Able to detect contest repo is using `hardhat` or `foundry`.
- [x] Support build for Hardhat.
- [ ] Support build for Truffle.
- [ ] Parse `foundry.toml` more efficient. `foundry` repo already did this so will dive deeper when i have time.
- [ ] Solc compile without cloning the repo. The idea is to scape all the source files with imported dependencies being resolved, thus 
making the compile process easier.