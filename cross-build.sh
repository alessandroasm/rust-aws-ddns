#!/bin/bash
./docker/build-container-armhf.sh
cross build --target armv7-unknown-linux-gnueabihf --features openssl
