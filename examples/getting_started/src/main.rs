/*!
This example demonstrates configuring `emit` to forward its events through the OpenTelemetry SDK.

This is useful if you're already using the OpenTelemtry SDK, or if your application uses multiple frameworks.
*/

use opentelemetry_otlp::WithTonicConfig;
use opentelemetry_sdk::runtime::Tokio;

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

    let tracer_provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(trace_exporter, Tokio)
        .build();

    let log_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_channel(channel.clone())
        .build()
        .unwrap();

    let logger_provider = opentelemetry_sdk::logs::LoggerProvider::builder()
        .with_batch_exporter(log_exporter, Tokio)
        .build();

    // Configure `emit` to point to `opentelemetry`
    let _ = emit_opentelemetry::setup(logger_provider.clone(), tracer_provider.clone()).init();

    // Run our example program
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
    emit::runtime::shared().emit(emit::Metric::new(
        emit::mdl!(),
        "counter",
        emit::well_known::METRIC_AGG_COUNT,
        emit::Empty,
        counter,
        emit::Empty,
    ));
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
