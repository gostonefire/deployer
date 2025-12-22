#!/bin/bash

REPO_NAME=$1
VERSION_TAG=$2
DEV_DIR=$3
SCRIPT_LOG=$4

if [ -z "$REPO_NAME" ] || [ -z "$VERSION_TAG" ] || [ -z "$DEV_DIR" ] || [ -z "$SCRIPT_LOG" ]; then
  echo "Usage: $0 <repo_name> <version_tag> <dev_dir> <script_log>"
  exit 1
fi

# shellcheck disable=SC2164
cd "$DEV_DIR"/"$REPO_NAME" >> "$SCRIPT_LOG" 2>&1
EXIT_CODE=$?
if [ $EXIT_CODE -ne 0 ]; then
  echo "could not change directory to repo while deploying $REPO_NAME with version $VERSION_TAG..."
  exit $EXIT_CODE
fi

git fetch --tags --prune >> "$SCRIPT_LOG" 2>&1
EXIT_CODE=$?
if [ $EXIT_CODE -ne 0 ]; then
  echo "could not fetch --tags --prune while deploying $REPO_NAME with version $VERSION_TAG..."
  exit $EXIT_CODE
fi

git checkout -f "$VERSION_TAG" >> "$SCRIPT_LOG" 2>&1
EXIT_CODE=$?
if [ $EXIT_CODE -ne 0 ]; then
  echo "could not checkout from git while deploying $REPO_NAME with version $VERSION_TAG..."
  exit $EXIT_CODE
fi

cargo clean >> "$SCRIPT_LOG" 2>&1
EXIT_CODE=$?
if [ $EXIT_CODE -ne 0 ]; then
  echo "could not execute cargo clean while deploying $REPO_NAME with version $VERSION_TAG..."
  exit $EXIT_CODE
fi

cargo build --release >> "$SCRIPT_LOG" 2>&1
EXIT_CODE=$?
if [ $EXIT_CODE -ne 0 ]; then
  echo "could not build release while deploying $REPO_NAME with version $VERSION_TAG..."
  exit $EXIT_CODE
fi

./scripts/deploy.sh >> "$SCRIPT_LOG" 2>&1
EXIT_CODE=$?
if [ $EXIT_CODE -ne 0 ]; then
  echo "could not run repo deploy script while deploying $REPO_NAME with version $VERSION_TAG..."
  exit $EXIT_CODE
fi

exit 0
