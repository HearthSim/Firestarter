# Firestarter

HS server simulation framework.

> This README file is a work in progress. Sections will be filled in when supporting code is present.

# Installation

Project contributors aim to make Firestarter very easy to install by limiting the need for required system components.
Getting the server built and running also requires few manual steps.

## Dependancies

- The Rust compiler toolchain.
  Download Rustup from [the official website](https://www.rust-lang.org/en-US/install.html). Execute it in your command prompt.
  By default Rustup will install itself, the latest stable Rust compiler and tools like Cargo (Rust's package manager).
  Enable the option to use Rust from the PATH environment variable!

## Build

These steps suppose the Rust binaries are accessible trough the PATH environment variable.

- Download the repository through Git, or use the zip file option and unpack;
- Open the downloaded repository within your command prompt;
- Execute the command `cargo test`, all framework components will be built and tests are ran.
    All tests should pass!
- Execute the command `cargo build --release --bin vanilla-server`, this will build a server executable in release mode.
    The executable can be found within the folder `target/release/`.

## Run

### Environment data

[ ] Setup environment data retrieval
[ ] Setup .env file
[ ] Explain ENV file+keys in README

Altough the project will use defaults for missing environment data, it's recommended that you create a file specifically for your
system.

Running the framework can happen in two ways.

- In-tree, using `cargo run --release --bin vanilla-server`;
- Distributed, having downloaded an archive containing the executable and runtime dependancies. Run the executable, preferrably
from a command prompt.

# Contributing

## Guidelines

Idiomatic Rust differs from popular scripting languages and C++ from early 2000. At the same time we understand that Rust is still a young language
and idomatic concepts are still in development or will change in the coming years.
We would like to keep the code Rust-beginner-friendly so anyone could technically download the source, start experimenting and be productive in a few hours. To this end we ask to try making the code understandable at a quick glance.

Explicit guidelines to this end are following:

- Make use of Cargo and crates.io for dependancies;
- Stick to the Cargo [crate layout](#project-layout);
- Do NOT use Nightly;
- Do NOT use crates with an unstable frontend;
- Use Rustfmt to achieve consistent formatting accross the entire codebase;
- Document all implemented functionality;
- Have at least one concrete usage example per module (inside the module docs for example);
- Try to write unit tests for implemented functionality;
- Try to use more clearly named variables while coding complicated logic.

## Project layout

This project follows a directory structure [expected by Cargo](https://doc.rust-lang.org/beta/cargo/reference/manifest.html#the-project-layout), Rust's package manager. Please read that section for detailed information.
Note that `firestarter/examples/` and `firestarter/benchmarks/` are put into dependant crates, instead of their respective crate-level folder.

A brief explanation follows:

- `firestarter/`
  The 'firestarter crate'; a collection of code files defining, testing and executing server logic. This is the core library.

- `firestarter/src/`
  The server code library.

- `firestarter/src/bin/`
  Contains executable entry points for the library. Each file is a seperate entry point.
  You can 'run' the library by invoking one of these entry points; e.g. `cargo run --bin [bin_target]`

- `firestarter/tests/`
  Integration tests. Unit tests should be placed within the source files themselves.
  It's idiomatic to create a private submodule to group test cases for some functionality.

- `firestarter-XXX/`
  These folders are dependant crates. Code that doesn't change often is moved into a seperate crate to lower the total compilation time.
  These crates are also used to extend the core library for specialized needs.

## Submit a PR

All contributions are reviewed and merged through pull-requests here on Github. Please do not hesitate to [contact us](https://hearthsim.info/join/) for help and information.

# License

Copyright (C) 2018 HearthSim community

This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.
