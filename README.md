# Nulink-chain

<p align="center">
  <a href="https://www.nulink.org/"><img src="https://github.com/NuLink-network/nulink/blob/94c5538a5fdc25e7d4391f4f2e4af60b3c480fc1/logo/nulink-bg-1.png" width=40%  /></a>
</p>

<p align="center">
  <a href="https://github.com/NuLink-network"><img src="https://img.shields.io/badge/Playground-NuLink_Network-brightgreen?logo=Parity%20Substrate" /></a>
  <a href="http://nulink.org/"><img src="https://img.shields.io/badge/made%20by-NuLink%20Foundation-blue.svg?style=flat-square" /></a>
  <a href="https://github.com/NuLink-network/nulink-chain"><img src="https://img.shields.io/badge/project-Nulink_Chain-yellow.svg?style=flat-square" /></a>
</p>

The project Nulink-chain is trying to bridge the NuCypher Network from Ethereum to the Polkadot Ecosystem. The NuCypher Network is a decentralized network of nodes that perform threshold cryptography operations which are serving users with secret management and dynamic access control.

NuLink network is using a fresh FRAME-based [Substrate](https://www.substrate.io/) node, ready for hacking :rocket:

## Getting Started

### Rust Setup

Instructions for setting up working environment of the [Rust](https://www.rust-lang.org/) programming language can
be found at the
[Substrate Developer Hub](https://substrate.dev/docs/en/knowledgebase/getting-started). Follow those
steps to install [`rustup`](https://rustup.rs/) and configure the Rust toolchain to default with the
latest stable version.

### Makefile

This project uses a [Makefile](Makefile) to document helpful commands and make them easier to be executed. 
Get started by running these [`make`](https://www.gnu.org/software/make/manual/make.html)
targets:

1. `make init` - Run the [init script](scripts/init.sh) to configure the Rust toolchain for
   [WebAssembly compilation](https://substrate.dev/docs/en/knowledgebase/getting-started/#webassembly-compilation).
1. `make run` - Build and launch this project in development mode.

The init script and Makefile both specify the version of the
[Rust nightly compiler](https://substrate.dev/docs/en/knowledgebase/getting-started/#rust-nightly-toolchain)
which this project is depending on.

### Build

The `cargo run` command will perform an initial build. Use the following command to build the node
without launching it:

```sh
cargo build --release
```
or you can do `cargo build` or `cargo build --release` to build it. and you can run `cargo test` to run the tests.
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
A simple way of using NULINK-NETWORK to distribute rewards to all stakers with local assets(NLK),you can run a local node by using this command `./target/release/nulink-chain --dev --tmp --ws-external`. 
For using the [Polkadot JS UI](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer), you may need the [types](https://github.com/NuLink-network/nulink-chain/blob/main/types.json) with the UI. After that you can start to register the watcher node and make it work.

```
1. register watcher
2. synchronize the staker by watcher
3. deposit local asset for reward
4. claim reward by staker
5. create policy by user
6. revoke policy by user
```

### Register Watcher
The first step in using this pallet is to register the watcher. Once the registration is successfully done, the watcher would begin to update the staker information here. In the test mode, you can manually control the watcher to update the stake information(For example, you can register watchers with the inherent user `alice` by submitting an extrinsic with the `nuproxy.register_watcher` function.). And we also provide a automatic watcher network to retrieve the staker information from Ethereum and to update it here. The implementation of watcher nodes can be found [here](https://github.com/NuLink-network/NuLink-watcher). 

1. `origin`: the owner of the watcher, in our example it's `alice`.

![](https://github.com/NuLink-network/nulink/blob/main/img2/Register_Watcher.jpg?raw=true)

Remark: Single watcher is supported currently in this version and the watcher can only be registered once. 

### Synchronize The Staker By Watcher
After the watcher registration is completed, the watcher would submit the staker information to the Nulink Network. In the test mode, the wacher can simply submit an extrinsic with `nuproxy.update_staker_infos_and_mint` function. This function is used for updating the staker infos and calculating reward upon request. And if you are using our watcher network to do the synchronization, then you do not need to do anything. The watcher nodes would automatically sync the staker information in epoch base. The epoch is defined in the NuCypher contract in the Ethereum network(7 days currently).


1. `origin`: the watcher account(`alice`) registered by `nuproxy.register_watcher` interface.
2. `infos`: the new list of stakers in the next epoch from ethereum by watcher set.

![](https://github.com/NuLink-network/nulink/blob/main/img2/update_by_epoch.jpg?raw=true)

### Deposit Local Asset For Reward
Everyone(normally the DAO treasury) can deposit assets(Local asset[`NLK`]) to the vault by submitting an extrinsic with `nuproxy.reserve_to_vault` function. The asset in the vault is used for assigning basic rewards to the stakers.

1. `origin`: the account(`alice`) who reserves the asset to vault.
2. `amount`: the amount of the local asset(NLK).

![](https://github.com/NuLink-network/nulink/blob/main/img2/vault.jpg?raw=true)

### Claim Reward By Staker
The stakers can retrieve rewards any time by submitting an extrinsic with `nuproxy.claim_reward_by_staker`.

1. `origin`: the staker user account.
2. `amount`: the amount of the local asset(NLK).

![](https://github.com/NuLink-network/nulink/blob/main/img2/claim.jpg?raw=true)

Remark: The rewards accumulated in the current epoch can be claimed after this epoch.

### Create Policy By User
Users who want to use NuLink’s PRE service can now use `policy.create_policy` to create policy in the Nulink Network.

1. `origin`: the user account(`alice`) who can create policy.
2. `pid`: the ID of the policy which is produced by the user offline.
3. `amount`: the amount of the local asset(NLK) for rewarding the staker.
4. `period`: the time range for the staker to process the policy. it's counted by the number of block numbers.
5. `stakers`: the address of the worker who will serve the PRE service for the user.

![](https://github.com/NuLink-network/nulink/blob/main/img2/create_policy.jpg?raw=true)

### Revoke Policy By User
Users who have created a policy can use `policy.revoke_policy` to revoke the policy before it expires. The remaining fees of this policy will be returned to the policy creator. And the user can withdraw their remaining balance(NLK) with `nuproxy.claim_reward_by_user`.

1. `origin`: the user account(`alice`) who has created the policy.
2. `pid`: the ID of the policy which is produced by the user outside.

![](https://github.com/NuLink-network/nulink/blob/main/img2/revoke_policy.jpg?raw=true)