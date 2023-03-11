#!/bin/sh -l

files=$(echo "$@" | xargs ff | paste -sd " ")
echo "files=$files" >> $GITHUB_OUTPUT
