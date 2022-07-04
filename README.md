<h1 align="center">Webb Protocol ink! Contracts 🕸️ </h1>
<div align="center">
<a href="https://www.webb.tools/">
    <img alt="Webb Logo" src=".github/assets/webb-icon.svg" width="15%" height="30%" />
  </a>
  </div>
<p align="center">
    <strong>🚀 Webb's ink! Smart Contract Implementation 🚀</strong>
    <br />
    <sub> ⚠️ Beta Software ⚠️ </sub>
</p>

<div align="center" >

[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/webb-tools/protocol-ink/CI?style=flat-square)](https://github.com/webb-tools/protocol-ink/actions)
[![License Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg?style=flat-square)](https://www.apache.org/licenses/LICENSE-2.0)
[![Built with ink!](https://raw.githubusercontent.com/paritytech/ink/master/.images/badge_flat.svg)](https://github.com/paritytech/ink)
[![Twitter](https://img.shields.io/twitter/follow/webbprotocol.svg?style=flat-square&label=Twitter&color=1DA1F2)](https://twitter.com/webbprotocol)
[![Telegram](https://img.shields.io/badge/Telegram-gray?logo=telegram)](https://t.me/webbprotocol)
[![Discord](https://img.shields.io/discord/833784453251596298.svg?style=flat-square&label=Discord&logo=discord)](https://discord.gg/cv8EfJu3Tn)

</div>

<!-- TABLE OF CONTENTS -->
<h2 id="table-of-contents"> 📖 Table of Contents</h2>

<details open="open">
  <summary>Table of Contents</summary>
  <ul>
    <li><a href="#start"> Getting Started</a></li>
    <li><a href="#prerequisites">Prerequisites</a></li>
    <li><a href="#compile">Compiling</a></li>
    <li><a href="#test">Testing</a></li>
    <li><a href="#contribute">Contributing</a></li>
    <li><a href="#license">License</a></li>
  </ul>  
</details>

<h2 id="start"> Getting Started  🎉 </h2>

For additional information, please refer to the [Webb Protocol-ink! Rust Docs](https://webb-tools.github.io/protocol-ink/) 📝. Have feedback on how to improve protocol-ink? Or have a specific question to ask? Checkout the [Anchor Protocol Feedback Discussion](https://github.com/webb-tools/feedback/discussions/categories/anchor-protocol) 💬.

Looking for additional ink! documentation and deployment tools? 

[ink! Documentation Portal](https://ink.substrate.io)&nbsp;&nbsp;•&nbsp;&nbsp;
[Developer Documentation](https://paritytech.github.io/ink/ink_lang/)&nbsp;&nbsp;•&nbsp;&nbsp;
[ink! Deployment UI](https://paritytech.github.io/contracts-ui/#/instantiate)

### Project layout

```
/
  |____contracts/   # Contains all ink! smart contracts, including governed token wrapper, mixer, vanchor, and poseidon contracts
  |____scripts/     # Dedicated directory for useful scripts, currently holds the `deploy.ts` script for easy deployments
  |____tests/       # Contains all integration tests for ink! contracts (e.g. `mixer_tests.ts`, `tokenWrapper.test.ts`)
```

<h2 id="start"> Prerequisites  🎉 </h2>

This repository makes use of node.js, yarn, Rust, and requires version 16. To install node.js binaries, installers, and source tarballs, please visit https://nodejs.org/en/download/. Once node.js is installed you may proceed to install [`yarn`](https://classic.yarnpkg.com/en/docs/install):

```
npm install --global yarn
```

Great! Now your **Node** environment is ready! 🚀🚀

A prerequisite for compiling smart contracts is to have Rust and Cargo installed. We suggest using <https://rustup.rs> installer and the `rustup` tool to manage the Rust toolchain.

First install and configure `rustup`:

```bash
# Install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Configure
source ~/.cargo/env
```

Configure the Rust toolchain to default to the latest nightly version, and add the nightly wasm target:

```bash
rustup default nightly
rustup update
rustup update nightly
rustup component add rust-src --toolchain nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
```

Great! Now your Rust environment is ready! 🚀🚀

Install `dylint-link`, required to lint ink! contracts, warning you about things like using API's in a way that could lead to security issues.\

```
cargo install dylint-link --locked
```

We recommend installing [`cargo-contract`](https://github.com/paritytech/cargo-contract) as well.
It's a CLI tool which helps set up and manage WebAssembly smart contracts written with ink!:

```
cargo install cargo-contract --force
```

Use the `--force` to ensure you are updated to the most recent `cargo-contract` version.

<h2 id="compile"> Compiling 💻 </h2>

Once the development environment is set up, lets compile some contracts:

```
npx redpost compile
```

Or optionally for each contract:
```
cargo contract build
```

<h2 id="test"> Testing 🧪 </h2>

Running the test suite requires installing a fork of Parity's substrate contract node, you will need to clone and build Webb's substrate contract node.

```
# Clone the forked repo
git clone git@github.com:webb-tools/substrate-contracts-node.git

# Install contracts node
cargo install contracts-node --git https://github.com/webb-tools/substrate-contracts-node.git --force --locked
```
The `--locked` flag makes the installation use the same versions as the `Cargo.lock` in those repositories ‒ ensuring that the last known-to-work version of the dependencies are used. 

The reason for utilizing a fork is that we have made specific changes to fit our needs in the `ChainExtension`, you may see those changes in this commit [7462b](https://github.com/webb-tools/substrate-contracts-node/commit/7462b5cc97a801ea09fc9ea24c337017bb63183b).

To run a local dev node execute:

```
substrate-contracts-node --dev
```

In another terminal, install the dependencies: 

```
yarn install
```

Build all the contracts:

```
yarn build
```

Run test suite:

```
yarn test
```

<h2 id="contribute"> Contributing </h2>

Interested in contributing to the Webb Relayer Network? Thank you so much for your interest! We are always appreciative for contributions from the open-source community!

If you have a contribution in mind, please check out our [Contribution Guide](./.github/CONTRIBUTING.md) for information on how to do so. We are excited for your first contribution!

<h2 id="license"> License </h2>

Licensed under <a href="LICENSE">Apache 2.0 license</a>.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the Apache 2.0 license, shall be licensed as above, without any additional terms or conditions.