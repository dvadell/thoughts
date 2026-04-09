# Hacking thoughts

This document provides a more detailed look at the internals of `thoughts`.

## Compiling / Testing
```
cargo fmt
cargo clippy    # or cargo clippy --fix --bin "thoughts" --allow-dirty to apply fixes
cargo test
cargo build
. release.sh    # to create a statically-compiled binary - Not here yet
git tag v0.2.6  # or whatever version
git push --tags
git push --follow-tags
```


## Project Structure
TBD
