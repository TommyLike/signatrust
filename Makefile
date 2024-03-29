GIT_COMMIT=$(shell git rev-parse --verify HEAD)

## Prepare mysql database
db:
	./scripts/initialize-database.sh

client-image:
	docker build -t tommylike/signatrust-client:$(GIT_COMMIT) --build-arg BINARY=client -f Dockerfile .

data-server-image:
	docker build -t tommylike/signatrust-data-server:$(GIT_COMMIT) --build-arg BINARY=data-server -f Dockerfile .

control-server-image:
	docker build -t tommylike/signatrust-control-server:$(GIT_COMMIT) --build-arg BINARY=control-server -f Dockerfile .