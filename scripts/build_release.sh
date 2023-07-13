#!/usr/bin/env bash

set -e
set -o pipefail

projectPath=$(cd "$(dirname "${0}")" && cd ../ && pwd)

U="cosmwasm"
V="0.12.6"

M=$(uname -m)
#M="x86_64" # Force Intel arch

A="linux/${M/x86_64/amd64}"
S=${M#x86_64}
S=${S:+-$S}

docker run --platform $A --rm -v "$projectPath":/code \
  --mount type=volume,source="$(basename "$projectPath")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  $U/workspace-optimizer$S:$V
