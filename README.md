# `emit_opentelemetry`

[![opentelemetry](https://github.com/emit-rs/emit_opentelemetry/actions/workflows/opentelemetry.yml/badge.svg)](https://github.com/emit-rs/emit_opentelemetry/actions/workflows/opentelemetry.yml)

[Current docs](https://docs.rs/emit_opentelemetry/0.28.1/emit_opentelemetry/index.html)

Integrate [`emit`](https://github.com/emit-rs/emit) with the OpenTelemetry SDK.

This library forwards diagnostic events from emit through the OpenTelemetry SDK as log records and spans.
It lets you use `emit`'s ergonomic developer-oriented API in your OpenTelemetry-instrumented applications.
It's also ideal for applications integrating multiple frameworks together, using the OpenTelemetry SDK as a common target.

See [the guide](https://emit-rs.io) for more details on using `emit` itself.

## Getting started

Configure the OpenTelemetry SDK as per its documentation, then add `emit` and `emit_opentelemetry` to your Cargo.toml:

```toml
[dependencies.emit]
version = "1"

# add `emit_openetelemetry` with the same major/minor as the OpenTelemetry SDK
[dependencies.emit_opentelemetry]
version = "0.32"

[dependencies.opentelemetry_sdk]
version = "0.32"
features = ["trace", "logs"]

[dependencies.opentelemetry]
version = "0.32"
features = ["trace", "logs"]
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
    
    // IMPORTANT: Traces need to be started through the OpenTelemetry SDK
    // Functions annotated with `#[emit::span]` only produce spans if they're
    // already in a sampled trace

    rt.blocking_flush(std::time::Duration::from_secs(30));

    // Shutdown the SDK
    let _ = logger_provider.shutdown();
    let _ = tracer_provider.shutdown();
}
```

This function accepts a [`LoggerProvider`](https://docs.rs/opentelemetry/0.28/opentelemetry/logs/trait.LoggerProvider.html) and [`TracerProvider`](https://docs.rs/opentelemetry/0.28/opentelemetry/trace/trait.TracerProvider.html) from the OpenTelemetry SDK to forward `emit` events to.

## Logging

Events emitted with `emit` will be mapped into OpenTelemetry log records:

```rust
let user = "Rust";

// This event will be emitted to the OpenTelemetry `LoggerProvider`
emit::info!("hello, {user}!");
```

## Tracing

Functions annotated with `emit` will be mapped into OpenTelemetry spans, with trace context managed by the OpenTelemetry SDK:

```rust
#[emit::span("instrumented function")]
fn instrumented_fn() {
    // Within the body of this function, the OpenTelemetry `Context` will carry the trace context computed by the `#[emit::span]` macro
}
```

For more details on how to use `emit` once you've initialized it, see [the guide](https://emit-rs.io), or [examples in the main `emit` repository](https://github.com/emit-rs/emit/tree/main/examples).

## Versioning and compatibility

`emit_opentelemetry` version `x.y.z` is compatible with `opentelemetry_sdk` version `x.y.*`.

## Limitations

The OpenTelemetry SDK's design imposes significant limitations on how other frameworks can integrate with it. For `emit` this means:

- Traces must be started by the OpenTelemetry SDK using its configured sampler. Any `emit` spans created outside an active sampled OpenTelemetry trace will be discarded.
- Any trace/span ids assigned by `emit` will be ignored, and assigned by the OpenTelemetry SDK instead. The trace/span ids exposed through `emit`'s context will reflect what's in the OpenTelemetry context.
