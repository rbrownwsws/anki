# Rust Example Addon

To build this example you need `cargo component`

You can install it with the following command:

```shell
cargo install cargo-component
```

Once you have done this you can build the example with the following command:

```shell
cargo component build --release
```

If all goes well you can then find the built addon in:

`<anki-repo-root>/addons/examples/rust/target/wasm32-unknown-unknown/release/example_addon.wasm`
