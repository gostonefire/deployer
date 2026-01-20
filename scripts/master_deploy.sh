#!/bin/bash

REPO_NAME=$1
VERSION_TAG=$2
DEV_DIR=$3
SCRIPTS_DIR=$4

if [ -z "$REPO_NAME" ] || [ -z "$VERSION_TAG" ] || [ -z "$DEV_DIR" ] || [ -z "$SCRIPTS_DIR" ]; then
  echo "Usage: $0 <repo_name> <version_tag> <dev_dir> <scripts_dir>"
  exit 1
fi

# Load common.sh (it is within /usr/local/bin) for common functions
source common.sh

# Get the owner of the DEV_DIR
DEV_USER=$(stat -c '%U' "$DEV_DIR")
DEV_HOME=$(getent passwd "$DEV_USER" | cut -d: -f6)

MASTER_LOG="$SCRIPTS_DIR"/master_deploy.log
echo "$(date '+%Y-%m-%d %H:%M:%S') - *** Deploying $REPO_NAME with version $VERSION_TAG: ***" >> "$MASTER_LOG"
chown "$DEV_USER":"$DEV_USER" "$MASTER_LOG"

SUB_SCRIPT_DIR="$SCRIPTS_DIR"/"$REPO_NAME"
SUB_SCRIPT_LOG="$SUB_SCRIPT_DIR"/deploy.log
mkdir -p "$SUB_SCRIPT_DIR"
chown -R "$DEV_USER":"$DEV_USER" "$SUB_SCRIPT_DIR"

# Function containing the logic to be run as the directory owner
run_as_user() {
  # Load dependencies and cargo environment if it exists
  source common.sh
  [ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"

  run_cmd "cd $DEV_DIR/$REPO_NAME"                  "$MASTER_LOG" "could not change directory to repo while deploying $REPO_NAME with version $VERSION_TAG..."
  run_cmd "git fetch --tags --prune"                "$MASTER_LOG" "could not fetch --tags --prune while deploying $REPO_NAME with version $VERSION_TAG..."
  run_cmd "git checkout -f $VERSION_TAG"            "$MASTER_LOG" "could not checkout from git while deploying $REPO_NAME with version $VERSION_TAG..."
  run_cmd "cargo clean"                             "$MASTER_LOG" "could not execute cargo clean while deploying $REPO_NAME with version $VERSION_TAG..."
  run_cmd "cargo build --release"                   "$MASTER_LOG" "could not build release while deploying $REPO_NAME with version $VERSION_TAG..."
  run_cmd "cp ./scripts/deploy.sh $SUB_SCRIPT_DIR/" "$MASTER_LOG" "could not copy deploy script while deploying $REPO_NAME with version $VERSION_TAG..."
  run_cmd "chmod 755 $SUB_SCRIPT_DIR/deploy.sh"     "$MASTER_LOG" "could not make repo deploy script executable while deploying $REPO_NAME with version $VERSION_TAG..."
}

# Export variables so the subshell can see them, then run the function as the owner
export REPO_NAME VERSION_TAG DEV_DIR MASTER_LOG SUB_SCRIPT_DIR

sudo -u "$DEV_USER" -E HOME="$DEV_HOME" bash -li -c "$(declare -f run_as_user); run_as_user"
EXIT_CODE=$?
if [ $EXIT_CODE -ne 0 ]; then
  exit $EXIT_CODE
fi

run_cmd "$SUB_SCRIPT_DIR/deploy.sh $REPO_NAME $DEV_DIR $SUB_SCRIPT_LOG" "$MASTER_LOG" "could not run repo deploy script while deploying $REPO_NAME with version $VERSION_TAG..."

exit 0
