# Contributing
Contributions are welcome. 🙂

## Issues
Please include logs and error messages.


## Pull Requests
Pull requests should pass our checks.

### Formatting
Code is formatted using:

```console
$ cargo fmt --all
```

### Linting
Code is linted:

```console
$ cargo clippy --all-features --all-targets --workspace -- -D warnings
$ yamllint --strict .
```

### Tests
Tests come in unit and integration tests. The latter require credentials. Run ALL tests by using:

```console
$ cargo test --all-features --workspace
```

You can skip integration tests:

```console
$ cargo test --all-features --workspace -- --skip integration
```

**Pull requests do NOT have access to the test credentials!**

### Documentation
Code should have a rough documentation. Docs are checked using:

```console
$ cargo doc --document-private-items --no-deps --all-features --workspace
```
