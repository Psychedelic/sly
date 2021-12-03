# Sly

Sly is a humanized CLI tool for the Internet Computer. With Sly, you are able to but not limited to write and build your own smart contract canisters on IC. At this stage, Sly is focused on backend canisters and only provides basic utilities and functionalities for creating backend canisters. We aim at providing more useful functionalities in near future for every developer in the ecosystem.

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
