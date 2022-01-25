# Nulink

[![Try on playground](https://img.shields.io/badge/Playground-nulink_chain-brightgreen?logo=Parity%20Substrate)](https://playground.substrate.dev/?deploy=nulink-chain)

The project NuLink is trying to bridge the NuCypher Network to Polkadot Ecosystem. The NuCypher Network is a decentralized network of nodes that perform threshold cryptography operations serving users with secrets management and dynamic access control.

NuLink network used a fresh FRAME-based [Substrate](https://www.substrate.io/) node, ready for hacking :rocket:

## Getting Started

### Rust Setup

Setup instructions for working with the [Rust](https://www.rust-lang.org/) programming language can
be found at the
[Substrate Developer Hub](https://substrate.dev/docs/en/knowledgebase/getting-started). Follow those
steps to install [`rustup`](https://rustup.rs/) and configure the Rust toolchain to default to the
latest stable version.

### Makefile

This project uses a [Makefile](Makefile) to document helpful commands and make it easier to execute
them. Get started by running these [`make`](https://www.gnu.org/software/make/manual/make.html)
targets:

1. `make init` - Run the [init script](scripts/init.sh) to configure the Rust toolchain for
   [WebAssembly compilation](https://substrate.dev/docs/en/knowledgebase/getting-started/#webassembly-compilation).
1. `make run` - Build and launch this project in development mode.

The init script and Makefile both specify the version of the
[Rust nightly compiler](https://substrate.dev/docs/en/knowledgebase/getting-started/#rust-nightly-toolchain)
that this project depends on.

### Build

The `cargo run` command will perform an initial build. Use the following command to build the node
without launching it:

```sh
cargo build --release
```
or you and `cargo build` or `cargo build --release` to build it. and you can run `cargo test` to run the tests.
```
 cargo build 
 or 
 cargo test
```

### Embedded Docs

Once the project has been built, the following command can be used to explore all parameters and
subcommands:

```sh
./target/release/nulink-chain -h
```

## Run

The provided `cargo run` command will launch a temporary node and its state will be discarded after
you terminate the process. After the project has been built, there are other ways to launch the
node.

### Single-Node Development Chain

This command will start the single-node development chain with persistent state:

```bash
./target/release/nulink-chain --dev
```

Purge the development chain's state:

```bash
./target/release/nulink-chain purge-chain --dev
```

Start the development chain with detailed logging:

```bash
RUST_LOG=debug RUST_BACKTRACE=1 ./target/release/nulink-chain -lruntime=debug --dev
```

### Connect with Polkadot-JS Apps Front-end

Once the nulink-node is running locally, you can connect it with **Polkadot-JS Apps** front-end
to interact with your chain. [Click here](https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9944) connecting the Apps to your local nulink-node.

### Multi-Node Local Testnet

If you want to see the multi-node consensus algorithm in action, refer to
[our Start a Private Network tutorial](https://substrate.dev/docs/en/tutorials/start-a-private-network/).


## Usage
A simple way to use NULINK-NETWORK to distribute rewards to all stakers used local asset(NLK),you can run local node for use it with `./target/release/nulink-chain --dev --tmp --ws-external`, and use the [Polkadot JS UI](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer), you may need the [types](https://github.com/NuLink-network/nulink-chain/blob/main/types.json) with the UI.then we can register the watcher and make it work.

```
1. register watcher
2. deploy the staker by watcher
3. deposit local asset for reward
4. claim reward by staker
5. create policy by user
6. revoke policy by user
```

### Register Watcher
Before using the Pallet, you need to register the watcher first. After the registration is successful, the watcher can submit the registered staker information and update the staker information regularly.
ps: Only supports one watcher for the time being.
we can register watcher with the inherent user `alice` and submit an extrinsicz with `nuproxy.register_watcher` function.

1. `origin`: the owner of the watcher,on this,it's `alice`.

### Deploy The Staker By Watcher
After the watcher registration is completed, the watcehr node submits the staker information to the nulink network in every epoch.
The watcher collect the staker infos from ethereum network and update it to the nulink network,the epoch is based on the epoch in the contract in the ethereum network.

There is a simple way to update staker infos into nulink network,submit an extrinsicz with `nuproxy.update_staker_infos_and_mint` function. update the staker infos and calc reward by epoch with the called by watchers,update the staker infos from ethereum network and reward it in every epoch if it still works on next epoch in the ethereum, If it stops working, the watcher will periodically notify the nulink network and stop rewarding it.

1. `origin`: the watcher account(`alice`) registered by `nuproxy.register_watcher` interface.
2. `infos`: the new stakers in next epoch from ethereum by watcher set.

### Deposit Local Asset For Reward
Before deploy the staker infos, it must be deposit assets(Local asset[`NLK`]) to the vault for assigned rewards, submit an extrinsicz with `nuproxy.reserve_to_vault` function.

1. `origin`: the account(`alice`) who reserve the asset to vault.
2. `amount`: the amount of the local asset(NLK).

### Claim Reward By Staker
Now stakers users can receive rewards after each epoch,we can submit an extrinsicz with `nuproxy.claim_reward_by_staker` to claim it's rewards.

1. `origin`: the staker user account.
2. `amount`: the amount of the local asset(NLK).

### Create Policy By User
We can use `policy.create_policy` to create policy by user and set the key params to nulink network.

1. `origin`: the user account(`alice`) can create policy.
2. `pid`: the ID of the policy,produced by the user on outside.
3. `amount`: the amount of the local asset(NlK),used to reward for the staker.
4. `period`: Indicates the time range for the staker to process the policy,calculated by the number of blocknumbers.
5. `stakers`: the worker of the nulink network,it works for the crypto newwork.

### Revoke Policy By User
We can use `policy.revoke_policy` to revoke the policy by user before they create it. If the reward for this policy is left, it will all be returned to the creator。
Finally, the user who use the revoke policy can check their remaining balance(NLK) with `Balance::free_balance`.

1. `origin`: the user account(`alice`) who had create the policy.
2. `pid`: the ID of the policy,produced by the user on outside.
