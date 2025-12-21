#!/bin/bash

REPO_NAME=$1
VERSION_TAG=$2

if [ -z "$REPO_NAME" ] || [ -z "$VERSION_TAG" ]; then
  echo "Usage: $0 <repo_name> <version_tag>"
  exit 1
fi

cd /home/petste/Develop/"$REPO_NAME" || exit
git fetch --tags --prune
git checkout -f "$VERSION_TAG"

cargo clean
cargo build --release

./scripts/deploy.sh
if [ $? -ne 0 ]; then
  exit $?
fi

echo "Deploying $REPO_NAME with version $VERSION_TAG..."


