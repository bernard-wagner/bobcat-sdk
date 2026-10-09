#!/bin/sh -e

# cargo nextest run --features std

# The upgrade test uses the SDK's existing proxy factory fixture.
make -C ../../e2e-test eip1967-proxy.wasm
make bozo.wasm

FOUNDRY_ISOLATE=false arbos-forge test --stylus-debug -vvv
FOUNDRY_ISOLATE=true arbos-forge test --stylus-debug -vvv
