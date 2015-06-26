# Curiosity

[![Build Status](https://travis-ci.org/cosmos-io/curiosity.svg)](https://travis-ci.org/cosmos-io/curiosity)

Curiosity is a container monitoring agent. It runs on each host you want to monitor. It collects metrics of all containers including itself. You can aggregate metrics from several hosts and get a modern dashboard with [Cosmos](https://github.com/cosmos-io/cosmos).

## Quick start

You can simply run Curiosity.

```
$ docker run -d -v /var/run/docker.sock:/var/run/docker.sock -e COSMOS_HOST=cosmos.io --name curiosity cosmosio/curiosity:nightly
```

## Requirements

* Docker (>= v1.5.0)

## Debug

Documentation is available [here](https://cosmos-io.github.io/curiosity/doc/curiosity).

### Rust

It is built with [Rust](http://www.rust-lang.org) that runs blazing fast, prevents almost all crashes, and eliminates data races. It is recommend to use [a Rust container](https://registry.hub.docker.com/u/cosmosio/curiosity/) when you debug. Of course, you can directly install Rust on your machine. If you do, please follow [the instruction](http://www.rust-lang.org/install.html).

* Rust (>= v1.0.0)

```
$ docker run --rm -it -v $(pwd):/source -v /var/run/docker.sock:/var/run/docker.sock -e COSMOS_HOST=cosmos.io cosmosio/rust:1.0.0-beta
```

## Run

```
$ docker run --rm --name curiosity -v /var/run/docker.sock:/var/run/docker.sock -e COSMOS_HOST=cosmos.io cosmosio/curiosity
```