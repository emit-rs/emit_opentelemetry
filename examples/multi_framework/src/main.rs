/*!
This example demonstrates configuring a few different diagnostic frameworks in the same application, all coordinating via the OpenTelemetry SDK.

In this case we're using:

- `log` and `emit` for logging.
- `tracing`, `opentelemetry`, and `emit` for tracing.

Large or complex applications may end up needing to integration components with different frameworks.
The OpenTelemetry SDK presents an ideal target for each of these frameworks to integrate with.
*/

use std::sync::Arc;

use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithTonicConfig as _;
use tracing_subscriber::layer::SubscriberExt as _;

#[tokio::main]
async fn main() {
    // 1. Configure the OpenTelemetry SDK
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

    // 2. Configure `emit` to point to `opentelemetry`
    let _ = emit_opentelemetry::setup(logger_provider.clone(), tracer_provider.clone()).init();

    // 3. Configure `log` to point to `opentelemetry`
    log::set_boxed_logger(Box::new(
        opentelemetry_appender_log::OpenTelemetryLogBridge::new(&logger_provider),
    ))
    .unwrap();
    log::set_max_level(log::LevelFilter::max());

    // 4. Configure `tracing` to point to `opentelemetry`
    let subscriber = Arc::new(tracing_subscriber::Registry::default().with(
        tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("multi_framework")),
    ));

    // 5. Run our example program
    run_opentelemetry(tracer_provider.clone(), || {
        log::info!("log in opentelemetry");
        emit::info!("emit in opentelemetry");

        run_tracing(subscriber.clone(), || {
            log::info!("log in tracing");
            emit::info!("emit in tracing");

            run_emit(|| {
                log::info!("log in emit");
                emit::info!("emit in emit");

                run_tracing(subscriber.clone(), || {
                    log::info!("log in tracing in emit");
                    emit::info!("emit in tracing in emit");
                });

                run_opentelemetry(tracer_provider.clone(), || {
                    log::info!("log in opentelemetry in emit");
                    emit::info!("emit in opentelemetry in emit");
                })
            });
        });

        run_emit(|| {
            log::info!("log in emit");
            emit::info!("emit in emit");
        });
    });

    // 6. Shutdown the SDK
    let _ = logger_provider.shutdown();
    let _ = tracer_provider.shutdown();
}

#[emit::span("run_emit")]
fn run_emit(f: impl FnOnce()) {
    f()
}

fn run_opentelemetry<T: opentelemetry::trace::TracerProvider>(tracer_provider: T, f: impl FnOnce())
where
    <T::Tracer as opentelemetry::trace::Tracer>::Span: Send + Sync + 'static,
{
    use opentelemetry::trace::Tracer;

    tracer_provider
        .tracer("run_opentelemetry")
        .in_span("Running OTel", |_| f())
}

fn run_tracing<T: tracing::Subscriber + Send + Sync>(subscriber: T, f: impl FnOnce()) {
    tracing::subscriber::with_default(subscriber, || {
        let span = tracing::span!(tracing::Level::INFO, "run_tracing");
        let _enter = span.enter();

        f()
    });
}
