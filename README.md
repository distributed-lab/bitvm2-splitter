# :axe: BitVM2 Made Practical

Practical implementation of the BitVM2 protocol

## :file_folder: Contents

The project contains multiple crates:

| Crate | Description |
| --- | --- |
| [splitter](splitter/README.md) | A crate for splitting the Bitcoin script into multiple parts as suggested by the recent [_BitVM2 paper_](https://bitvm.org/bitvm_bridge.pdf). |

## Setting up a Local Bitcoin Node

```shell
docker compose up -d
```

> [!WARNING]
> Sometimes Docker Compose may fail at step of creating the volumes, the most simple solution is, in regards of failure, just trying starting it again several times until it works.

Let us create a temporary alias for `bitcoin-cli` from the container like this:

```shell
alias bitcoin-cli="docker compose exec bitcoind bitcoin-cli"
```

Create a fresh wallet for your user:

```shell
bitcoin-cli createwallet "my"
```

> [!WARNING]
> Do not create more than one wallet, otherwise further steps would require
> a bit of modification.

Generate fresh address and store it to environmental variable:

```shell
export ADDRESS=$(bitcoin-cli getnewaddress "main" "bech32")
```

Then mine 101 blocks to your address:

```shell
bitcoin-cli generatetoaddress 101 $ADDRESS
```

> [!NOTE]
> Rewards for mined locally blocks will go to this address, but, by protocol rules, BTCs are mature only after 100 confirmations, so that's why 101 blocks are mined. You can see other in  `immature` balances fields, after executing next command.
>
> For more info about Bitcoin RPC API see [^1].

```shell
bitcoin-cli getbalances
```

[^1]: https://developer.bitcoin.org/reference/rpc/
