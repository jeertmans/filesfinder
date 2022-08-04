#!/bin/sh -l

files=$(ff "$@" | paste -sd " ")
echo "::set-output name=files::$files"
