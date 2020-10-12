<div align="center">

  <h1><code>flux-protocol</code></h1>

  <p>
    <strong>Open market protocol, build on NEAR.</strong>
  </p>

</div>

## Pre-requisites
To develop Rust contracts you would need to:
* (On Linux make sure you have `llvm`, `make`, `clang` and `librocksdb-dev` installed)

* Install [Rustup](https://rustup.rs/):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
* Install the nightly toolchain.rs nightly version.
```bash
rustup install nightly-2020-05-15
```
* Add wasm target to your toolchains:
```bash
rustup target add wasm32-unknown-unknown --toolchain stable
rustup target add wasm32-unknown-unknown --toolchain nightly-2020-05-15
```
* Clone the Flux Protocol repo 
```bash
git clone https://github.com/jasperdg/flux-protocol.git
```

## Running tests
Navigate to the protocol directory

```
cd flux-protocol
```

Create a res directory:
```
mkdir res
```

Run the test

```
bash scripts/test.sh
```