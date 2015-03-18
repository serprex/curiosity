# Curiosity

Curiosity is a drone to collect container data on a planet. If you want to monitor several Curiosity drones, use the Cosmos. You can find out more about [Cosmos](https://github.com/cosmos-io/cosmos).

## Run
```
$ docker run -v /var/run/docker.sock:/var/run/docker.sock -e COSMOS_HOST=cosmos.io -e COSMOS_PLANET_NAME=Mars --rm --name curiosity cosmosio/curiosity
```