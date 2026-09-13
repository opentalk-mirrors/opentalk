#!/usr/bin/env sh
# SPDX-License-Identifier: EUPL-1.2
# SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
#
# Shell script to start the roomserver in the background, load an environment file,
# run a load test, and stop the roomserver when done.
# This script uses `sh` for compatibility reasons. This script is used in the CI/CD
# environment and different container images (e.g. grafana/k6) which don't provide bash.

set -eu -o noglob

# Paths and inputs
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LOAD_TEST_DIR="${SCRIPT_DIR}/../load-test"
ROOMSERVER_CONFIG="${LOAD_TEST_DIR}/roomserver.toml"
TEST_NAME="$1"
TEST_BASENAME="$(basename "${TEST_NAME}" .js)"
ENV_FILE="${SCRIPT_DIR}/run-load-test-quick-env/${TEST_BASENAME}"
LOG_DIR="${LOG_DIR:-$(mktemp -d "${TMPDIR:-/tmp}/load-test_${TEST_BASENAME}_XXXXXX")}"
ROOMSERVER_LOG_FILE="${LOG_DIR}/roomserver.log"
ROOMSERVER_READY_TIMEOUT=10
ROOMSERVER_READY_CHECK_INTERVAL=1
ROOMSERVER_PID=""

# Keep ROOMSERVER_BIN as a single env var. Word splitting is intentional when
# invoking the command, so it must not be quoted. Default to "cargo run --".
ROOMSERVER_BIN="${ROOMSERVER_BIN:-cargo run --}"

cleanup() {
    exit_code=$?
    echo "Cleaning up..."

    # Stop the roomserver if it's still running
    if [ -n "${ROOMSERVER_PID}" ] && kill -0 "${ROOMSERVER_PID}" 2>/dev/null; then
        echo "Stopping roomserver (PID: ${ROOMSERVER_PID})..."
        kill "${ROOMSERVER_PID}" || true
        # Wait for the process to terminate gracefully
        count=0
        while kill -0 "${ROOMSERVER_PID}" 2>/dev/null && [ "$count" -lt 10 ]; do
            sleep 1
            count=$((count + 1))
        done
        # Force kill if necessary
        if kill -0 "${ROOMSERVER_PID}" 2>/dev/null; then
            echo "Roomserver did not stop gracefully, force killing..."
            kill -9 "${ROOMSERVER_PID}" || true
        fi
    fi

    exit "${exit_code}"
}

# Set trap for cleanup on script exit
trap cleanup EXIT

# Ensure log directory exists (needed when LOG_DIR is provided externally)
mkdir -p "${LOG_DIR}"

# Load test-specific environment file if it exists
if [ -f "${ENV_FILE}" ]; then
    echo "Loading environment file: ${ENV_FILE}"
    set +u
    set -a
    # shellcheck source=/dev/null
    . "${ENV_FILE}"
    set +a
    set -u
else
    echo "No environment file found for test: ${TEST_BASENAME}"
fi

echo "Starting roomserver in the background..."
cd "${SCRIPT_DIR}" || exit 1

# Start roomserver in background (word splitting on ROOMSERVER_BIN is intentional)
# shellcheck disable=SC2086
NO_COLOR=1 ${ROOMSERVER_BIN} --config "${ROOMSERVER_CONFIG}" >"${ROOMSERVER_LOG_FILE}" &
ROOMSERVER_PID=$!
echo "Roomserver started with PID: ${ROOMSERVER_PID}"

# Wait for roomserver to be ready
echo "Waiting for roomserver to be ready (timeout: ${ROOMSERVER_READY_TIMEOUT}s)..."
READY=0
ELAPSED=0

while [ "$ELAPSED" -lt "$ROOMSERVER_READY_TIMEOUT" ]; do
    # shellcheck disable=SC2086
    if ${ROOMSERVER_BIN} --config "${ROOMSERVER_CONFIG}" health > /dev/null 2>&1; then
        READY=1
        break
    fi

    # Check if process is still alive
    if ! kill -0 "${ROOMSERVER_PID}" 2>/dev/null; then
        exit 1
    fi

    sleep "${ROOMSERVER_READY_CHECK_INTERVAL}"
    ELAPSED=$((ELAPSED + ROOMSERVER_READY_CHECK_INTERVAL))
done

if [ "$READY" -eq 0 ]; then
    echo "Roomserver did not become ready within ${ROOMSERVER_READY_TIMEOUT}s"
    exit 1
fi

echo "Roomserver is ready!"

# Run the load test
echo "Running load test: ${TEST_NAME}"
cd "${LOAD_TEST_DIR}" || exit 1
k6 run --no-usage-report "${TEST_NAME}"
echo "Load test completed successfully!"
echo "Log files stored at ${LOG_DIR}"
