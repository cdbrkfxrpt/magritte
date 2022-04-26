# magritte

_Ceci n'est pas une pipe._

> An implementation of a logic-based time series analysis approach.

As self declared, magritte is not a pipeline (at least not in the canonical
sense), even though a visualization would clearly look like one. It is
asynchronous on all paths, and it takes care of everything from data
acquisition to storing results in an easy-to-visualize manner.


## What else is it not?

Unless you are interested for the purpose of learning, this is not something
you should be looking at. In other words, **don't use this in prod**.


## Background

Analyzing time series data is an active field of research and engineering. A
number of methods and approaches exist and can be very roughly categorized as
follows, according to the author's perspective:

- classic approaches (rooted in signal processing)
- statistics based approaches (pure statistics, machine learning, ...)
- logic based approaches (event based or event driven, capable of composing
    expressions, etc.)

Most systems which have been created for the purpose of enabling time series
data analysis are some mixture of the above as a pure implementation seems all
but impossible. The different approaches are usually most applicable at
different stages of the process.

Using a logical approach in a concurrent way for the analysis stage of the
process is the subject of [my Master's
thesis](https://gitlab.bmc-labs.com/flrn/eich21), _Concurrent Stream Reasoning
for Complex Event Recognition_. This project here contains the implementation
of the (not-a-)pipeline part of the system.


## Architecture

**magritte** forms part of the overall architecture; specifically, it forms the
functional part, from data acquisition to results storage, but nothing outside
from those responsibilities. This allows for a good separation of concerns and
enables the use of programming languages and technologies better suited for the
respective purposes at both ends of the (not-a-)pipeline.

![Architecture showing **magritte**](./misc/architecture.png "Architecture showing magritte")

In actuality, **harvester** is a **magritte** component but can be easily
switched out for adaptation to a different data source.


## Base Technologies

All components of **magritte** are written in the Rust programming language.
The following core technologies are used:

- [tokio](https://tokio.rs/): asynchronous runtime
- [gRPC](https://grpc.io/): component interaction via RPC
    * [Protocol Buffers](https://developers.google.com/protocol-buffers): gRPC
        service description language designed by Google
    * [tonic](https://github.com/hyperium/tonic): Rust implementation of gRPC,
        asynchronous and from the tokio stack
- [actix](https://actix.rs/): now mainly a web framework, but also a Rust
    implementation of the Actor model based on tokio

**barbershop** and **opsroom** furthermore make heavy use of docker and
services running in docker containers. The database used by the setup is a
[PostgreSQL](https://www.postgresql.org/) (also running in a docker container).


## Code Base Structure

- **barbershop** and **opsroom**: separate entities, not included in this repo
- **magritte**: Rust project
    * text files (`Cargo.toml`, ...)
    * `proto`: contains `*.proto` file(s) specifying the gRPC protocol
    * `src`
        - `bin`: application code of binaries
            * `harvester`
            * `dylonet`
            * `bakery`
        - _library code..._

Tests are, as is possible and common in Rust, included in the respective source
files. The same is true for documentation which is included in source files in
the form of `cargo doc`-able comment strings.


## Build and Run

Building **magritte** is as canonical as it gets:

```sh
cargo build
```

Afterwards, if you intend running it, make sure that you have followed the
instructions which came with **barbershop** and everything is up and running.
When that's the case you're ready to run **magritte**:

```sh
cargo run --bin harvester
cargo run --bin dylonet
cargo run --bin bakery
```

The order doesn't matter here; **harvester** and **bakery** both wait for
**dylonet** to come up and **dylonet** in turn doesn't do anything without
receiving data from **harvester** and it buffers all the results (of recent
interest) of the current.

It's supposed to be simple. If you don't find it so, please let me know.

There are also tests:

```sh
cargo test
```

So also this is (supposed to be) simple.


## Usage

Input data must be stored in a PostgreSQL database.

#### Time Series Data

Time series data must be stored in a PostgreSQL database table as rows which
have as columns:

- exactly one source identifier (which vehicle/facility/sensor is this data
    point coming from?) in `int` or `bigint` format
- exactly one timestamp (`bigint` as UNIX epoch or `timestamp` format)
- one or more value domain elements

#### Background Knowledge

Background knowledge must be stored in PostgreSQL database table


## References and Acknowledgments

TO DO

---

<div align="center">
  This work is licensed under the Apache License, Version 2.0. You should have
  received a copy of this license along with the source code. If that is not
  the case, please find one <a
  href="http://www.apache.org/licenses/LICENSE-2.0">online</a>.
</div>
