#!/bin/bash

# Helper function to run commands with error checking
run_cmd() {
  local cmd="$1"
  local script_log="$2"
  local err_msg="$3"

  if [ -z "$cmd" ] || [ -z "$script_log" ] || [ -z "$err_msg" ]; then
    echo "ERROR: missing parameter to $0 <cmd> <script_log> <error_msg>"
    exit 1
  fi

  # We use eval so that redirected strings or complex commands work correctly
  eval "$cmd" >> "$script_log" 2>&1
  local code=$?
  if [ $code -ne 0 ]; then
    echo "ERROR: $err_msg"
    exit $code
  fi
}
