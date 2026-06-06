/*!
This example demonstrates configuring `emit` to forward its events through the OpenTelemetry SDK.

This is useful if you're already using the OpenTelemtry SDK, or if your application uses multiple frameworks.
*/

use opentelemetry_otlp::WithTonicConfig as _;

#[tokio::main]
async fn main() {
    // Configure the OpenTelemetry SDK
    // In this example, we're configuring it to produce OTLP
    let channel = tonic::transport::Channel::from_static("http://localhost:4319")
        .connect()
        .await
        .unwrap();

    let trace_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_channel(channel.clone())
        .build()
        .unwrap();

    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(trace_exporter)
        .build();

    let log_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_channel(channel.clone())
        .build()
        .unwrap();

    let logger_provider = opentelemetry_sdk::logs::SdkLoggerProvider::builder()
        .with_batch_exporter(log_exporter)
        .build();

    // Configure `emit` to point to `opentelemetry`
    let _ = emit_opentelemetry::setup(logger_provider.clone(), tracer_provider.clone()).init();

    // Run our example program
    // IMPORTANT: Traces must be started through the OpenTelemetry SDK. `emit`'s integration
    // discards any of its own `#[span]` events unless there's already an OpenTelemetry trace active.
    run_opentelemetry(tracer_provider.clone());

    // Shutdown the SDK
    let _ = logger_provider.shutdown();
    let _ = tracer_provider.shutdown();
}

// Emit a span
#[emit::span("Running emit")]
fn run_emit() {
    let mut counter = 1;

    for _ in 0..100 {
        counter += counter % 3;
    }

    // Emit a log record
    emit::info!("Counted up to {counter}");

    // Emit a metric sample
    //
    // NOTE: This sample will be emitted as a log, not a metric.
    // To send `emit`'s metrics via the OpenTelemetry SDK, you'll
    // need to use `emit_otlp` instead.
    //
    // You can still use `emit` for logs and traces, and just use the
    // OpenTelemetry SDK for your metrics.
    emit::count_sample!(value: counter);
}

fn run_opentelemetry<T: opentelemetry::trace::TracerProvider>(tracer_provider: T)
where
    <T::Tracer as opentelemetry::trace::Tracer>::Span: Send + Sync + 'static,
{
    use opentelemetry::trace::Tracer;

    tracer_provider
        .tracer("run_opentelemetry")
        .in_span("Running OTel", |_| {
            std::thread::sleep(std::time::Duration::from_secs(1));

            run_emit();
        })
}
