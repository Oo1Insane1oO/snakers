#!/bin/bash

FILELIST=$(git diff --diff-filter=d --cached --name-only | grep -E "\.rs$")

if [ ${#FILELIST} -gt 0 ]; then
    if ! cargo fmt --check -- ${FILELIST} --config skip_children=true; then
        echo "Running cargo fmt for $FILELIST"
        cargo fmt -- ${FILELIST}
        exit 1
    fi
fi
