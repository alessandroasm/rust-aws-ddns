## Cross compilation

This project can be cross compiled to another targets like raspberry pis
(armv7). In order to do so, follow the steps on this document.

**This document is incomplete. The compiled application fails because it is
dynamically linked to GLIBC. We must use Musl somehow in order to generate
a statically linked application.**

### Installing the toolchain

These steps were copied from [japaric's rust-cross](https://github.com/japaric/rust-cross)
and assumes Ubuntu 20.04.

```
# Step 0: Our target is an ARMv7 device, the triple for this target is `armv7-unknown-linux-gnueabihf`

# Step 1: Install the C cross toolchain
$ sudo apt-get install -qq gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf

# Step 2: Install the cross compiled standard crates
$ rustup target add armv7-unknown-linux-gnueabihf
$ rustup target add armv7-unknown-linux-musleabihf

# Step 3: Configure cargo for cross compilation
$ mkdir -p ~/.cargo
$ cat >>~/.cargo/config <<EOF
> [target.armv7-unknown-linux-gnueabihf]
> linker = "arm-linux-gnueabihf-gcc"
>
> [target.armv7-unknown-linux-musleabihf]
> linker = "arm-linux-musleabihf-gcc"
> EOF

# Test cross compiling a Cargo project
$ cargo new --bin hello
$ cd hello
$ cargo build --target=armv7-unknown-linux-gnueabihf
   Compiling hello v0.1.0 (file:///home/ubuntu/hello)
```

### OpenSSL

It is necessary to have a OpenSSL compilation for the specified target. The
following steps assume that openssl will be cloned to ~/src

```
# Step 0: clone OpenSSL (v1.1.1-stable)
$ git clone -b OpenSSL_1_1_1-stable https://github.com/openssl/openssl.git

# Step 1: configure for building
$ ./Configure linux-armv4 --prefix=/usr/local/openssl/ --openssldir=/usr/local/openssl shared

# Step 2: build using the gcc cross compiler
$ make CC=arm-linux-gnueabihf-gcc-9
```

### Building the application

```
PKG_CONFIG_ALLOW_CROSS=1 \
OPENSSL_LIB_DIR=~/src/openssl \
OPENSSL_INCLUDE_DIR=~/src/openssl/include \
OPENSSL_STATIC=1 \
cargo build --release --target=armv7-unknown-linux-gnueabihf
```
