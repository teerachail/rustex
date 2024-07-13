use opentelemetry_sdk::{runtime, trace::{self, Tracer}, Resource};
use opentelemetry::{trace::TraceError, KeyValue};
use opentelemetry_otlp::WithExportConfig;


pub fn init_trace(servic_name: &'static str) -> Result<Tracer, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
            .tonic().with_endpoint("http://localhost:4317")
        )
        .with_trace_config(
            trace::config()
            .with_resource(Resource::new(vec![
                KeyValue::new("service.name", servic_name),
            ]))
        )
        .install_batch(runtime::Tokio)
}