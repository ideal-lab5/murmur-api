# Murmur API

Murmur API is a Rust-based web service that allows you to interact with [Murmur Core](https://github.com/ideal-lab5/murmur).

## Environment Variables

Copy `.env.example` to `.env` and set its variables. If you are running this project locally with Docker Compose, you don't need to change anything.

```bash
cp .env.example .env
```

## Usage

You can run this project either natively or with Docker Compose. The exposed endpoint will be [http://127.0.0.1:8080](http://127.0.0.1:8080).

### Native

To run the project natively, follow these steps:

1. **Build the project**:

```bash
cargo build --release
```

2. **Run the executable**:

```bash
./target/release/murmur-api
```

#### Database

This project requires MongoDB as its database. If you are running it natively (not using Docker Compose), make sure you have an instance running. You can set up one on [mongodb.com](mongodb.com).

### Docker

To run the project using Docker Compose, follow these steps:

1. Build and start the containers:

```bash
docker-compose up --build
```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## License

This project is licensed under the Apache-2.0. See the [LICENSE](./LICENSE) file for details.

## Contact

For any inquiries, please contact [Ideal Labs](https://idealabs.network).
