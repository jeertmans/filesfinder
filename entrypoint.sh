#!/bin/sh -l

echo "Hello $1"
files=$(ff args)
echo "::set-output name=files::$files"
