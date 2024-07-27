#!/bin/bash

target=$1

if [[ -z $target ]]; then
    echo "Target not provided"
    exit 1
fi

if [[ -f "target/$target/release/comet" ]]; then
    cp "target/$target/release/comet" "./comet-$target"
else 
    cp "target/$target/release/comet.exe" "./comet-$target.exe"
fi

