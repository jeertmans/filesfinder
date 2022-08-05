#!/bin/sh -l

files=$(echo "$@" | xargs ff | paste -sd " ")
echo "::set-output name=files::$files"
