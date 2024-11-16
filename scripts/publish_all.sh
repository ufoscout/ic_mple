#!/usr/bin/env bash
set -e
set -x
export RUST_BACKTRACE=full

declare -a publish_list=(
    "src/ic_mple_pocket_ic"
)

for i in "${publish_list[@]}"
do
    LINE_SEPARATOR='--------------------------------------------------------'

    cd $i
    echo $LINE_SEPARATOR
    echo 'Run Cargo publish for [' $i ']'
    echo $LINE_SEPARATOR

    cargo publish
    sleep 2
    cd ../..
    rc=$?
    if [[ $rc -ne 0 ]] ; then
        echo "Failure publishing $i";
    fi

done
