#!/bin/bash

target=$1
params=''

if [[ -z $target ]]; then
    echo "Target not provided"
    exit 1
fi

if [ "$RUNNER_OS" = "Linux" ]; then
    params='docs/steamdeck'
fi

cp "target/$target/release/comet"* .

if [ "$RUNNER_OS" = "Windows" ]; then
    7z a "comet-$target.zip" comet.exe dummy-service/*{.exe,.md} 
    echo "archive_name=comet-$target.zip" >> "$GITHUB_OUTPUT"
else 
    tar -czvf "comet-$target.tar.gz" comet dummy-service/*{.exe,.md} $params
    echo "archive_name=comet-$target.tar.gz" >> "$GITHUB_OUTPUT"
fi

