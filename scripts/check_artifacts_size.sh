#!/usr/bin/env bash

set -e
set -o pipefail

projectPath=$(cd "$(dirname "${0}")" && cd ../ && pwd)

maximum_size=600

echo "Max size: $maximum_size"

for artifact in "$projectPath"/artifacts/*.wasm; do
  artifactsize=$(du -k "$artifact" | cut -f 1)
  if [ "$artifactsize" -gt $maximum_size ]; then
    echo "Artifact file size exceeded: $artifact"
    echo "Artifact size: $artifactsize"
#    exit 1
  fi
done
