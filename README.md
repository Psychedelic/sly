![](https://storageapi.fleek.co/fleek-team-bucket/logos/sly-metad.png)

# SLY

SLY is an open source, collaborative, and friendly CLI for developing on the Internet Computer. Providing abstractions, templates, and tools to kick-start and speed-up Internet Computer development.

- Visit [our website](https://sly.ooo)
- Visit [SLY's repository](https://github.com/psychedelic/sly)
- Follow [SLY on Twitter](https://twitter.com/sly_ooo) 

> This is an early version of SLY ✨ and more features are in development, anyone is welcome to contribute to build a friendly CLI tool for the IC that helps provide templates, streamlined flows, and abstractions when it comes to canister development and more!


## 🧰 Getting Started with SLY

To get started with SLY, you visit our getting started section and **start by installing SLY** in your system. You can find different guides for SLY's use cases, as well as the different utility tools it provides, on the menu to the left or in this section below:

1. [Install SLY](https://docs.sly.ooo/getting-started/installing-sly/)
2. [Prepare a Project](https://docs.sly.ooo/getting-started/workspace-management/)
3. [Create Local Replicas](https://docs.sly.ooo/getting-started/local-replicas/)
4. [Manage Identities](https://docs.sly.ooo/getting-started/identity-management/)
5. [Interact with Canisters](https://docs.sly.ooo/getting-started/interacting-with-canisters/)

### Utility Tools

There are several utility tools that SLY provides to help either abstract development flows of the IC, improve its performance, or help streamline it with examples and templates.

1. [Candid Utility Tools](https://docs.sly.ooo/getting-started/candid-utility/)
2. [WASM Optimizer](https://docs.sly.ooo/getting-started/wasm-optimizer/)

## How to use Sly?

Let's start by creating a new project:

```sh
sly new --name hello-world
cd hello-world
```

### How do templates work?

By default, the `sly new` command creates a new project with the rust backend template. If you have something else on your mind you can use the `--template` flag to use the template you need. Currently three templates can be used to create your project:

- Non Fungible Token
- Fungible Token
- Rust Backend

Example for Non Fungible Token template:

```sh
sly new --name hello-world --template non_fungible_token
cd hello-world
```

### How does the code architecture look like?

Now that we have created our new project, we can dive in and inspect the structure of the project. Every project Sly creates regardless of its template, follows this code structure:

```sh
.
├── sly.json
├── canister_ids.json
├── src/
└── .sly/
```

Each template might change the code structure in certain ways but these two json files and two directories will always be generated. Your `sly.json` file will contain your canister configurations and the `canister_ids.json` will contain the principal IDs of your canisters with the related network.

### How to start the local replica?

You can start your local replica with the `sly start` command.
