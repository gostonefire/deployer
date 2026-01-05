#!/bin/bash

REPO_NAME=$1
VERSION_TAG=$2
DEV_DIR=$3
SCRIPT_LOG_DIR=$4

if [ -z "$REPO_NAME" ] || [ -z "$VERSION_TAG" ] || [ -z "$DEV_DIR" ] || [ -z "$SCRIPT_LOG_DIR" ]; then
  echo "Usage: $0 <repo_name> <version_tag> <dev_dir> <script_log>"
  exit 1
fi

# Get the owner of the DEV_DIR
DEV_USER=$(stat -c '%U' "$DEV_DIR")

MASTER_LOG="$SCRIPT_LOG_DIR"/master_deploy.log
SUB_SCRIPT_LOG="$SCRIPT_LOG_DIR"/"$REPO_NAME"/deploy.log
mkdir -p "$SCRIPT_LOG_DIR"/"$REPO_NAME"
chown -R "$DEV_USER" "$SCRIPT_LOG_DIR"

# Function containing the logic to be run as the directory owner
run_as_user() {
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

  chmod 755 ./scripts/deploy.sh >> "$MASTER_LOG" 2>&1
  EXIT_CODE=$?
  if [ $EXIT_CODE -ne 0 ]; then
    echo "could not make repo deploy script executable while deploying $REPO_NAME with version $VERSION_TAG..."
    exit $EXIT_CODE
  fi
}

# Export variables so the subshell can see them, then run the function as the owner
export REPO_NAME VERSION_TAG DEV_DIR MASTER_LOG

sudo -u "$DEV_USER" -E bash -c "$(declare -f run_as_user); run_as_user"
EXIT_CODE=$?
if [ $EXIT_CODE -ne 0 ]; then
  exit $EXIT_CODE
fi

./scripts/deploy.sh "$REPO_NAME" "$DEV_DIR" "$SUB_SCRIPT_LOG" >> "$MASTER_LOG" 2>&1
EXIT_CODE=$?
if [ $EXIT_CODE -ne 0 ]; then
  echo "could not run repo deploy script while deploying $REPO_NAME with version $VERSION_TAG..."
  exit $EXIT_CODE
fi


exit 0
