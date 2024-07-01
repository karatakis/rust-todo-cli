#!/bin/bash

# Loop 100000 times
for i in {1..100000}
do
   # Run the cargo command with the current iteration number
   echo "Running task $i..."
   cargo run --quiet task add "task $i"
done