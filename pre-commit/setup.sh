#!/bin/bash

echo "Installing pre-commit hook ..."

cp pre-commit/pre-commit.sh .git/hooks/pre-commit

echo -e "\033[32mFinished!\033e"
