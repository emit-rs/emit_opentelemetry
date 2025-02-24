# `emit_opentelemetry`

[![opentelemetry](https://github.com/emit-rs/emit_opentelemetry/actions/workflows/opentelemetry.yml/badge.svg)](https://github.com/emit-rs/emit_opentelemetry/actions/workflows/opentelemetry.yml)

[Current docs](https://docs.rs/emit_opentelemetry/0.28.1/emit_opentelemetry/index.html)

Integrate `emit` with the OpenTelemetry SDK.

This library forwards diagnostic events from emit through the OpenTelemetry SDK as log records and spans.

## Getting started

Configure the OpenTelemetry SDK as per its documentation, then add `emit` and `emit_opentelemetry` to your Cargo.toml:

```toml
[dependencies.emit]
version = "1"

[dependencies.emit_opentelemetry]
version = "0.28.1"
```

Initialize `emit` to send diagnostics to the OpenTelemetry SDK using the `emit_opentelemetry::setup` function:

```rust
fn main() {
    // Configure the OpenTelemetry SDK
    // See the OpenTelemetry SDK docs for details on configuration
    let logger_provider = opentelemetry_sdk::logs::SdkLoggerProvider::builder()
        .with_simple_exporter(opentelemetry_stdout::LogExporter::default())
        .build();

    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
        .build();

    // Configure `emit` to point to the OpenTelemetry SDK
    let rt = emit_opentelemetry::setup(logger_provider, tracer_provider).init();

    // Your app code goes here

    rt.blocking_flush(std::time::Duration::from_secs(30));

    // Shutdown the OpenTelemetry SDK
}
```

This function accepts a [`LoggerProvider`](https://docs.rs/opentelemetry/0.28/opentelemetry/logs/trait.LoggerProvider.html) and [`TracerProvider`](https://docs.rs/opentelemetry/0.28/opentelemetry/trace/trait.TracerProvider.html) from the OpenTelemetry SDK to forward `emit` events to.

## Versioning and compatibility

`emit_opentelemetry` version `x.y.z` is compatible with `opentelemetry_sdk` version `x.y.*`.
