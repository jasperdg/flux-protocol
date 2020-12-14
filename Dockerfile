FROM mstorsjo/llvm-mingw

RUN apt-get update
RUN apt-get install curl -y
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN source $HOME/.cargo/env
RUN rustup install nightly-2020-05-15
RUN rustup target add wasm32-unknown-unknown --toolchain stable
RUN rustup target add wasm32-unknown-unknown --toolchain nightly-2020-05-15
