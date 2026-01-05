#!/bin/bash

REPO_NAME=$1
VERSION_TAG=$2
DEV_DIR=$3
SCRIPTS_DIR=$4

if [ -z "$REPO_NAME" ] || [ -z "$VERSION_TAG" ] || [ -z "$DEV_DIR" ] || [ -z "$SCRIPTS_DIR" ]; then
  echo "Usage: $0 <repo_name> <version_tag> <dev_dir> <scripts_dir>"
  exit 1
fi

# Get the owner of the DEV_DIR
DEV_USER=$(stat -c '%U' "$DEV_DIR")
DEV_HOME=$(getent passwd "$DEV_USER" | cut -d: -f6)

MASTER_LOG="$SCRIPTS_DIR"/master_deploy.log
SUB_SCRIPT_DIR="$SCRIPTS_DIR"/"$REPO_NAME"
SUB_SCRIPT_LOG="$SUB_SCRIPT_DIR"/deploy.log
mkdir -p "$SUB_SCRIPT_DIR"
chown -R "$DEV_USER" "$SUB_SCRIPT_DIR"

# Function containing the logic to be run as the directory owner
run_as_user() {
  # Load cargo environment if it exists
  [ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"

  # shellcheck disable=SC2164
  cd "$DEV_DIR"/"$REPO_NAME" >> "$MASTER_LOG" 2>&1
  EXIT_CODE=$?
  if [ $EXIT_CODE -ne 0 ]; then
    echo "could not change directory to repo while deploying $REPO_NAME with version $VERSION_TAG..."
    exit $EXIT_CODE
  fi

  git fetch --tags --prune >> "$MASTER_LOG" 2>&1
  EXIT_CODE=$?
  if [ $EXIT_CODE -ne 0 ]; then
    echo "could not fetch --tags --prune while deploying $REPO_NAME with version $VERSION_TAG..."
    exit $EXIT_CODE
  fi

  git checkout -f "$VERSION_TAG" >> "$MASTER_LOG" 2>&1
  EXIT_CODE=$?
  if [ $EXIT_CODE -ne 0 ]; then
    echo "could not checkout from git while deploying $REPO_NAME with version $VERSION_TAG..."
    exit $EXIT_CODE
  fi

  cargo clean >> "$MASTER_LOG" 2>&1
  EXIT_CODE=$?
  if [ $EXIT_CODE -ne 0 ]; then
    echo "could not execute cargo clean while deploying $REPO_NAME with version $VERSION_TAG..."
    exit $EXIT_CODE
  fi

  cargo build --release >> "$MASTER_LOG" 2>&1
  EXIT_CODE=$?
  if [ $EXIT_CODE -ne 0 ]; then
    echo "could not build release while deploying $REPO_NAME with version $VERSION_TAG..."
    exit $EXIT_CODE
  fi

  cp ./scripts/deploy.sh "$SUB_SCRIPT_DIR/" >> "$MASTER_LOG" 2>&1
  EXIT_CODE=$?
  if [ $EXIT_CODE -ne 0 ]; then
    echo "could not copy deploy script while deploying $REPO_NAME with version $VERSION_TAG..."
    exit $EXIT_CODE
  fi

  chmod 755 "$SUB_SCRIPT_DIR"/deploy.sh >> "$MASTER_LOG" 2>&1
  EXIT_CODE=$?
  if [ $EXIT_CODE -ne 0 ]; then
    echo "could not make repo deploy script executable while deploying $REPO_NAME with version $VERSION_TAG..."
    exit $EXIT_CODE
  fi
}

# Export variables so the subshell can see them, then run the function as the owner
export REPO_NAME VERSION_TAG DEV_DIR MASTER_LOG SUB_SCRIPT_DIR

sudo -u "$DEV_USER" -E HOME="$DEV_HOME" bash -li -c "$(declare -f run_as_user); run_as_user"
EXIT_CODE=$?
if [ $EXIT_CODE -ne 0 ]; then
  exit $EXIT_CODE
fi

"$SUB_SCRIPT_DIR"/deploy.sh "$REPO_NAME" "$DEV_DIR" "$SUB_SCRIPT_LOG" >> "$MASTER_LOG" 2>&1
EXIT_CODE=$?
if [ $EXIT_CODE -ne 0 ]; then
  echo "could not run repo deploy script while deploying $REPO_NAME with version $VERSION_TAG..."
  exit $EXIT_CODE
fi


exit 0
