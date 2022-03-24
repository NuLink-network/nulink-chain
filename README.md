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
A simple way of using NULINK-NETWORK is to run a local node by using this command `./target/release/nulink-chain --dev --tmp --ws-external`.  After you run the local node, you can use the [Polkadot JS UI](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer) to connect to your local node. 

![](https://github.com/NuLink-network/nulink/blob/main/img2/localnode.png?raw=true)





After you connect to your local node , you have to paste the config [types](https://github.com/NuLink-network/nulink-chain/blob/main/types.json) in the settings. After that you can start to register the watcher node and make all the functions work.

![](https://github.com/NuLink-network/nulink/blob/main/img2/app_config.png?raw=true)

```
1. register watcher
2. synchronize the staker by watcher
3. deposit local asset for reward
4. claim reward by staker
5. create policy by user
6. revoke policy by user
```

Also we provide a parachain for testing. You need to connect to our test network by filling the custom endpoint. The endpoint for our test parachain is wss://testnet-pok.nulink.org.  In our test network, the watcher has already been registered and it will automatically retrieve the staker information from Ethereum and update it here. The implementation of watcher node can be found [here](https://github.com/NuLink-network/NuLink-watcher).


![](https://github.com/NuLink-network/nulink/blob/main/img2/test_endpoint.png?raw=true)


### Register Watcher
The first step in using this pallet is to register the watcher. If you are connecting to our test network, then you can skip this step as we have already done it. 

And if  you are connecting to your own local node, you can use the  inherent user `alice` (also you can create an new account, but you need transfer enough token to afford the extrinsic  fee) and register it as the watcher by submitting an extrinsic with the `nuproxy.register_watcher` function. 


1. `origin`: the owner of the watcher, in our example it's `alice`.

![](https://github.com/NuLink-network/nulink/blob/main/img2/Register_Watcher.jpg?raw=true)

Remark: Single watcher is supported currently in this version and the watcher can only be registered once.  If you connect to our test parachain and try to register your own account as watcher, it will report failure and return a OnlyOneWatcher warning. 


### Synchronize The Staker By Watcher

If you are connecting to our test network, then you can also skip this step as the watcher node would automatically sync the staker information in epoch base. The epoch is defined in the NuCypher contract in the Ethereum network(7 days currently). 

If  you are connecting to your own local parachain. you can now use your watcher account to synchronize the staker information from Ethereum to your parachain by submitting the extrinsic `nuproxy.update_staker_infos_and_mint` . This function is used for updating the staker infos and calculating reward upon request.


1. `origin`: the watcher account(`alice`) registered by `nuproxy.register_watcher` interface.
2. `infos`: the new list of stakers in the next epoch from ethereum by watcher set.(workbase is the Ethereum address of the staker, coinbase is the bonding parachain address, iswork is the flag to identify if the staker is on duty or not for the upcoming period, locked balance is the amount of the staked NuCypher token, and workcount is a metric to record the number of PRE services this staker provides)

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

The staker  can check their reward balance through the state query `nuproxy.rewards`.
![](https://github.com/NuLink-network/nulink/blob/main/img2/rewards.png?raw=true)

Remark1: The rewards accumulated in the current epoch can be claimed after this epoch.

Remark2：If you are connecting to the  test network, you may need to add the following staker account in order to sign this extrinsics.
You can fill the raw seed when adding the staker account:  0xc8a9dda922026f8cb4619daacecaceaa04c731633cd61a358f5d8803ffe0fd76 or 0x8122683602e7c8ae75076d70c0ffcdec28ba15cd409b42e04001f2f2571391a7.


### Create Policy By User

When user Alice grant access to Bob, the Nucpher client would first create a policy off-chain and then publish this policy in Polkadot parachain(please refer [here](https://docs.nucypher.com/en/latest/application_development/cli_examples.html) for more details regarding creating a policy in Nucypher). Users Alice need to use `policy.create_policy` to create policy in the Nulink Network. 

1. `origin`: the user account(`alice`) who can create policy.
2. `pid`: the ID of the policy which is produced by the user off-line(Policy ID is used to identify the policy in the Ursulas network and is uniquely generated when the data owner grants access to the data receiver in the Nucypher client. It is the hash of the pubkey of the data owner, the pubkey of the  data receiver and the policy label. Please check [here](https://github.com/nucypher/nucypher/blob/31bd5a6998760385d5e36bce9a0c55e3ff161cc8/nucypher/policy/policies.py#L86) for more infos).
3. `amount`: the amount of the local asset(NLK) for rewarding the staker.
4. `period`: the time range for the staker to process the policy. it's counted by the number of block numbers.
5. `stakers`: the address of the worker who will serve the PRE service for the user.

![](https://github.com/NuLink-network/nulink/blob/main/img2/create_policy.jpg?raw=true)

You can check the current stake list through the state query `nuproxy.stakes`.
![](https://github.com/NuLink-network/nulink/blob/main/img2/stakers.png?raw=true)



### Revoke Policy By User
Users who have created a policy can use `policy.revoke_policy` to revoke the policy before it expires.  The remaining fees of this policy will be returned to the policy creator. 

1. `origin`: the user account(`alice`) who has created the policy.
2. `pid`: the ID of the policy which is produced by the user outside.

![](https://github.com/NuLink-network/nulink/blob/main/img2/revoke_policy.jpg?raw=true)


### Claim the Remaining Balance By Policy Creator

The policy creator can claim back all his remaining balance after he revokes the policy. He can simply  send the`nuproxy.claim_reward_by_user` extrinsic. 

![](https://github.com/NuLink-network/nulink/blob/main/img2/claimbyuser.png?raw=true)


