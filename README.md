<div align="center">

  <h1><code>flux-protocol</code></h1>

  <p>
    <strong>Open market protocol, build on NEAR.</strong>
  </p>

</div>

## Pre-requisites
To develop Rust contracts you would need to:
* Install [Rustup](https://rustup.rs/):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
* Add wasm target to your toolchain:
```bash
rustup target add wasm32-unknown-unknown
```
* Clone the Flux Protocol repo 
```bash
git clone https://github.com/jasperdg/flux-protocol.git
```
* (On Linux make sure you have `build-essentials`, `clang` and `librocksdb-dev` installed)

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