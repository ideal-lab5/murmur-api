# Murmur API

This is a simple API that allows you to interact with [Murmur Core](https://github.com/ideal-lab5/murmur).

## Environment Variables

Copy `.env.example` to `.env` and set its variables. If you are running this project locally with docker compose, you don't need to change anything.

## Usage

Either run this project natively or with docker compose, the exposed endpoint will be `http://127.0.0.1:8080`.

### Native
```bash
cargo build --release
./target/release/murmur-api
```

#### Database

This project requires MongoDB as its database. If you are running it natively (not using the docker compose provided), make sure you have an instance running. You can set up one on [mongodb.com](mongodb.com).

### Docker

```bash
docker-compose up --build
```
