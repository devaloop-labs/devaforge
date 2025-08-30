<div align="center">
  <img src="https://devalang.com/images/devalang-logo-min.png" alt="Devalang Logo" width="100" />
</div>

![Rust](https://img.shields.io/badge/Made%20with-Rust-orange?logo=rust)

![Version](https://img.shields.io/npm/v/@devaloop/devaforge)
![License: MIT](https://img.shields.io/badge/license-MIT-green)

![npm](https://img.shields.io/npm/dt/@devaloop/devaforge)
![crates](https://img.shields.io/crates/d/devaforge)

# 🦊 Devaforge (addon creator for Devalang)

You know [Devalang](https://devalang.com), the powerful DSL for music and audio manipulation. Now, with Devaforge, you can easily create and manage your own addons.

This library provides a simple way to create and manage addons for Devalang.

## 📚 Quick Access

- [📦 Devalang (CORE)](https://github.com/devaloop-labs/devalang)
- [▶️ Playground](https://playground.devalang.com)
- [📖 Documentation](https://docs.devalang.com)
- [🧩 VSCode Extension](https://marketplace.visualstudio.com/items?itemName=devaloop.devalang-vscode)
- [🌐 Project Website](https://devalang.com)

## 🚀 Features

- [**BANK GENERATOR**: Create and manage sound banks effortlessly.](./docs/BANKS.md)
- More addon types coming soon !

## ▶️ Get started

### Installation

#### Node.js (NPM)

```bash
npm i -g @devaloop/devaforge
```

#### Rust (Cargo)

```bash
cargo install devaforge
```

### Commands

##### Run the following command to create a new bank:

```bash
devaforge bank create
```

##### Run the following command to delete a bank:

```bash
devaforge bank delete <author>.<bank_name>
```


### <center>[See more bank commands](./docs/BANKS.md)</center>

### Contributing

You must have Rust installed on your machine. Then, you can build the project using Cargo :

```bash
npm install
```

```bash
cargo build
```

```bash
cargo run
```

## 🤝 Contributing

Contributions, bug reports and suggestions are welcome !  
Feel free to open an issue or submit a pull request.

## 🛡️ License

MIT — see [LICENSE](./LICENSE)
