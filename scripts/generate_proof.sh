#!/bin/bash
echo "Step 1: Compiling circuit..."
cd circuits/spending_proof
nargo execute
echo "Step 2: Writing verification key..."
bb write_vk -b ./target/spending_proof.json -o ./target/vk
echo "Step 3: Generating proof..."
bb prove -b ./target/spending_proof.json -w ./target/spending_proof.gz -o ./target/proof -k ./target/vk/vk
echo "Step 4: Verifying proof..."
bb verify -p ./target/proof/proof -k ./target/vk/vk -i ./target/proof/public_inputs
echo "Done! Proof verified successfully."
