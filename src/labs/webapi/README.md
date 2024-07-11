# Web API Rust Starter

This web api is based on axum and includes a basic setup for a web api in Rust.
Those setup are:
* Basic routing
* Basic error handling
* Basic logging (using tracing and opentelemetry)
* Graceful shutdown

## How to run

```bash
cargo run
```

## Open Telemetry

Using open telemetry to trace the application.
In the folder `accel` has a docker-compose file to run the jaeger and prometheus.

```bash
cd accel
docker-compose up
```

You can see the traces in the jaeger dashboard.
To open the jaeger dashboard, open the browser and access `http://localhost:16686/`.

After exit jetter and prometheus, you can stop the containers using the command:

```bash
docker-compose down
```
