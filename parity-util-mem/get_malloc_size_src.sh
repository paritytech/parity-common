#!/bin/bash

# script/process to update code from servo project (malloc_size_of)
# untested, note that we do not use submodule due to size of git repo
git clone https://github.com/servo/servo.git
cd servo
git checkout 5bdea7dc1c80790a852a3fb03edfb2b8fbd403dc
git apply ../slim_malloc_size_of.patch
#git merge master
cp components/malloc_size_of/lib.rs ../src/malloc_size.rs
cp -r components/malloc_size_of_derive ..
cd ..
#rm -rf ./servo
