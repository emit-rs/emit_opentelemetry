/*!
An integration test between `emit_opentelemetry` and the OpenTelemetry Collector.
*/

use std::{
    io::Read,
    process::{Child, Command, Stdio},
    time::Duration,
};

use opentelemetry_otlp::WithTonicConfig as _;

#[tokio::main]
async fn main() {
    let _ = emit::setup().emit_to(emit_term::stdout()).init_internal();

    let otelcol = OtelCol::spawn("config");

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

    // Generate some random ids
    // These are used to assert the collector received our events
    let log_uuid = uuid::Uuid::new_v4().to_string();
    let span_uuid = uuid::Uuid::new_v4().to_string();

    // Emit a log event
    emit::info!("A log message {log_uuid}");

    // Emit a span in a trace
    #[emit::span(name: "emit_otlp_test", "A span {span_uuid}")]
    async fn span(span_uuid: &str) {
        tokio::time::sleep(Duration::from_millis(3)).await;
    }
    span(&span_uuid).await;

    // Shutdown the SDK
    let _ = logger_provider.shutdown();
    let _ = tracer_provider.shutdown();

    // Give the collector time to process events
    tokio::time::sleep(Duration::from_secs(3)).await;

    let output = otelcol.output();

    // Ensure the collector received and accepted the events we emitted
    assert_exporter(&output, "LogsExporter", &log_uuid);
    assert_exporter(&output, "TracesExporter", &span_uuid);
}

fn assert_exporter(output: &str, exporter: &str, id: &str) {
    assert!(
        output.contains(&exporter),
        "{exporter} not found in:\n{output}"
    );
    assert!(output.contains(id), "{id} not found in:\n{output}");
}

struct OtelCol(Child);

impl Drop for OtelCol {
    fn drop(&mut self) {
        let _ = self.0.kill();
    }
}

impl OtelCol {
    fn spawn(config: &str) -> Self {
        OtelCol(
            Command::new("otelcol")
                .args(["--config", &format!("./{config}.yaml")])
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap(),
        )
    }

    fn output(mut self) -> String {
        let mut stdout = self.0.stdout.take().unwrap();
        let mut stderr = self.0.stderr.take().unwrap();

        self.0.kill().unwrap();

        let mut buf = String::new();
        stdout.read_to_string(&mut buf).unwrap();
        stderr.read_to_string(&mut buf).unwrap();

        buf
    }
}
