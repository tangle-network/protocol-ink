<h1 align="center">Webb Protocol Ink!</h1>

<p align="center">
    <strong>🕸️  Webb Protocol Ink!  ⧫</strong>
    <br />
    <sub> ⚠️ Beta Software ⚠️ </sub>
</p>

<br />

## Dependencies
A prerequisite for compiling smart contracts is to have Rust and Cargo installed. Here's [an installation guide](https://doc.rust-lang.org/cargo/getting-started/installation.html).

We recommend installing [`cargo-contract`](https://github.com/paritytech/cargo-contract) as well.
It's a CLI tool which helps set up and manage WebAssembly smart contracts written with ink!:

```
cargo install cargo-contract --force
```

Use the `--force` to ensure you are updated to the most recent `cargo-contract` version.


## Compiling

- `npx redpost compile`

Or optionally for each contract:

- `cargo contract build`
## Testing 

- First ensure you have downloaded substrate contract node and follow the instructions [here](https://github.com/paritytech/substrate-contracts-node#download-binary)
- Then run `yarn install`.
- Then run `yarn build` to build all the contracts.
- Then run `yarn test`.

## License

<sup>
Licensed under <a href="LICENSE">GPLV3 license</a>.
</sup>

<br/>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the GPLV3 license, shall
be licensed as above, without any additional terms or conditions.
</sub>

