#!/bin/bash

REPO_NAME=$1
VERSION_TAG=$2

if [ -z "$REPO_NAME" ] || [ -z "$VERSION_TAG" ]; then
  echo "Usage: $0 <repo_name> <version_tag>"
  exit 1
fi

# shellcheck disable=SC2164
cd /home/petste/Develop/"$REPO_NAME"
EXIT_CODE=$?
if [ $EXIT_CODE -ne 0 ]; then
  echo "could not change directory to repo while deploying $REPO_NAME with version $VERSION_TAG..."
  exit $EXIT_CODE
fi

git fetch --tags --prune
EXIT_CODE=$?
if [ $EXIT_CODE -ne 0 ]; then
  echo "could not fetch --tags --prune while deploying $REPO_NAME with version $VERSION_TAG..."
  exit $EXIT_CODE
fi

git checkout -f "$VERSION_TAG"
EXIT_CODE=$?
if [ $EXIT_CODE -ne 0 ]; then
  echo "could not checkout from git while deploying $REPO_NAME with version $VERSION_TAG..."
  exit $EXIT_CODE
fi

cargo clean
EXIT_CODE=$?
if [ $EXIT_CODE -ne 0 ]; then
  echo "could not execute cargo clean while deploying $REPO_NAME with version $VERSION_TAG..."
  exit $EXIT_CODE
fi

cargo build --release
EXIT_CODE=$?
if [ $EXIT_CODE -ne 0 ]; then
  echo "could not build release while deploying $REPO_NAME with version $VERSION_TAG..."
  exit $EXIT_CODE
fi

./scripts/deploy.sh
EXIT_CODE=$?
if [ $EXIT_CODE -ne 0 ]; then
  echo "could not run repo deploy script while deploying $REPO_NAME with version $VERSION_TAG..."
  exit $EXIT_CODE
fi

exit 0
