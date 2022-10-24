Provides specifications for `std` functions. By importing this and adding `extern crate prusti_std;` to your root file, one can avoid needing to write [external specifications](https://viperproject.github.io/prusti-dev/user-guide/verify/external.html). This crate does not replace `prusti-contracts`, you will still need to import the latter to write contracts in your crate.