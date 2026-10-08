#!/bin/sh

cargo test --features std,proptest,alloy-enabled $@
