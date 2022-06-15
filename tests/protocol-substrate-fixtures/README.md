# protocol-substrate-fixtures

## Building new keys
In order to create keys for circuits with new configurations we should modify the script's main function (`builder/src/main.rs`) in the following way:
```rust
fn main() {
    ...
    // We are adding a new VAnchor configuration
    // We are using a tree of heigth 30
    // and anchor count of 3
    // and number of input utxos of 8
    // and number of output utxos of 16
    // First argument is the elliptic curve we want to use
    
    // Second argument is the path of the folder we wish to
    // save our keys into -- relative to the builder/Cargo.toml
    // Folder name follows the rule of "[anchor]-[input]-[output]", like "3-8-16"
    generate_vanchor_keys::<Bn254, 30, 3, 8, 16>(Curve::Bn254, "../vanchor/bn254/x5/3-8-16");
    
    ...
}
```
> **Note: The folder structure needs to be setup before running the script**