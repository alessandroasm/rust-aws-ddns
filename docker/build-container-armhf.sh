#!/bin/bash
cd "$(dirname "$0")"
docker build -t rust-aws-ddns/base-armhf-img -f Dockerfile-armhf .
