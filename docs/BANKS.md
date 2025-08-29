<div align="center">
  <img src="https://devalang.com/images/devalang-logo-min.png" alt="Devalang Logo" width="100" />
</div>

# Banks Forge

## Create

You will be prompted to enter a name for your new bank.

Bank will be generated at `generated/banks/<author>.<name>`

Once created, you can add sounds inside its `audio` folder.

```bash
cargo run -- bank create
```

## Build

This command will discover all audio files in the `audio` folder of each bank and place them into the bank's metadata.

This command will compile all banks and discover their sounds into their `.devabank` compressed file (`output/bank/<author>.<name>.devabank`).

```bash
cargo run -- bank build
```

After build, you can copy-paste the generated bank (`generated/banks/<author>.<name>/`) files to your Devalang project inside the `.deva/bank/<author>.<name>/` folder then use them in your project by declaring them in your `.devalang` like this :

```toml
...

[[banks]]
path = "devalang://bank/<author>.<name>"
version = "0.0.1"

...
```

For more information on how to use banks in your project, please refer to the Devalang documentation.

## List

List all available banks under `generated/banks`.

```bash
cargo run -- bank list
```

## Versioning

Bump a bank version by `major`, `minor`, or `patch`.

```bash
cargo run -- bank version <author>.<name> <major|minor|patch>

# Examples
cargo run -- bank version devaloop.808 major   # (M+1).0.0
cargo run -- bank version devaloop.808 minor   # M.(m+1).0
cargo run -- bank version devaloop.808 patch   # M.m.(p+1)
```
