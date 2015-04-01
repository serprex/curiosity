# Curiosity

[![Build Status](https://travis-ci.org/cosmos-io/curiosity.svg)](https://travis-ci.org/cosmos-io/curiosity)

Curiosity is a drone to collect container data on a planet. If you want to monitor several Curiosity drones, use the Cosmos. You can find out more about [Cosmos](https://github.com/cosmos-io/cosmos).

## Debug
```
$ docker pull cosmosio/rust
$ docker run --rm -it -v $(pwd):/source -v /var/run/docker.sock:/var/run/docker.sock -e COSMOS_HOST=cosmos.io -e COSMOS_PLANET_NAME=Mars cosmosio/rust
```

## Run
```
$ docker run --rm --name curiosity -v /var/run/docker.sock:/var/run/docker.sock -e COSMOS_HOST=cosmos.io cosmosio/curiosity
```