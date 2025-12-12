//! OTLP gRPC receiver for ingesting OpenTelemetry data
//!
//! This module implements an OTLP receiver that accepts telemetry data
//! via gRPC and converts it to our UI-friendly data model.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use opentelemetry_proto::tonic::collector::trace::v1::{
    trace_service_server::{TraceService, TraceServiceServer},
    ExportTraceServiceRequest, ExportTraceServiceResponse,
};
use opentelemetry_proto::tonic::trace::v1::span::SpanKind as ProtoSpanKind;
use opentelemetry_proto::tonic::trace::v1::Status as ProtoStatus;
use tokio::sync::mpsc;
use tonic::{transport::Server, Request, Response, Status};

use crate::telemetry::{SpanData, SpanKind, SpanStatus, TelemetryEvent};

/// Convert protobuf timestamp (nanos since epoch) to SystemTime
fn proto_time_to_system_time(time_unix_nano: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_nanos(time_unix_nano)
}

/// Convert protobuf span kind to our SpanKind
fn convert_span_kind(kind: i32) -> SpanKind {
    match ProtoSpanKind::try_from(kind) {
        Ok(ProtoSpanKind::Internal) => SpanKind::Internal,
        Ok(ProtoSpanKind::Server) => SpanKind::Server,
        Ok(ProtoSpanKind::Client) => SpanKind::Client,
        Ok(ProtoSpanKind::Producer) => SpanKind::Producer,
        Ok(ProtoSpanKind::Consumer) => SpanKind::Consumer,
        _ => SpanKind::Internal,
    }
}

/// Convert protobuf status to our SpanStatus
fn convert_status(status: Option<ProtoStatus>) -> SpanStatus {
    match status {
        Some(s) => match s.code() {
            opentelemetry_proto::tonic::trace::v1::status::StatusCode::Ok => SpanStatus::Ok,
            opentelemetry_proto::tonic::trace::v1::status::StatusCode::Error => SpanStatus::Error,
            _ => SpanStatus::Unset,
        },
        None => SpanStatus::Unset,
    }
}

/// Convert bytes to hex string
fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// OTLP Trace service implementation
pub struct OtlpTraceReceiver {
    tx: mpsc::UnboundedSender<TelemetryEvent>,
}

impl OtlpTraceReceiver {
    pub fn new(tx: mpsc::UnboundedSender<TelemetryEvent>) -> Self {
        Self { tx }
    }
}

#[tonic::async_trait]
impl TraceService for OtlpTraceReceiver {
    async fn export(
        &self,
        request: Request<ExportTraceServiceRequest>,
    ) -> Result<Response<ExportTraceServiceResponse>, Status> {
        let req = request.into_inner();

        for resource_spans in req.resource_spans {
            // Extract service name from resource attributes
            let service_name = resource_spans
                .resource
                .as_ref()
                .map(|r| {
                    r.attributes
                        .iter()
                        .find(|a| a.key == "service.name")
                        .and_then(|a| a.value.as_ref())
                        .and_then(|v| v.value.as_ref())
                        .map(|v| match v {
                            opentelemetry_proto::tonic::common::v1::any_value::Value::StringValue(s) => s.clone(),
                            _ => "unknown".to_string(),
                        })
                        .unwrap_or_else(|| "unknown".to_string())
                })
                .unwrap_or_else(|| "unknown".to_string());

            for scope_spans in resource_spans.scope_spans {
                for span in scope_spans.spans {
                    let trace_id = bytes_to_hex(&span.trace_id);
                    let span_id = bytes_to_hex(&span.span_id);
                    let parent_span_id = if span.parent_span_id.is_empty() {
                        None
                    } else {
                        Some(bytes_to_hex(&span.parent_span_id))
                    };

                    // Convert attributes
                    let attributes: HashMap<String, String> = span
                        .attributes
                        .iter()
                        .filter_map(|a| {
                            a.value.as_ref().and_then(|v| v.value.as_ref()).map(|v| {
                                let val = match v {
                                    opentelemetry_proto::tonic::common::v1::any_value::Value::StringValue(s) => s.clone(),
                                    opentelemetry_proto::tonic::common::v1::any_value::Value::IntValue(i) => i.to_string(),
                                    opentelemetry_proto::tonic::common::v1::any_value::Value::DoubleValue(d) => d.to_string(),
                                    opentelemetry_proto::tonic::common::v1::any_value::Value::BoolValue(b) => b.to_string(),
                                    _ => "...".to_string(),
                                };
                                (a.key.clone(), val)
                            })
                        })
                        .collect();

                    let span_data = SpanData {
                        trace_id,
                        span_id,
                        parent_span_id,
                        name: span.name.clone(),
                        kind: convert_span_kind(span.kind),
                        start_time: proto_time_to_system_time(span.start_time_unix_nano),
                        end_time: if span.end_time_unix_nano > 0 {
                            Some(proto_time_to_system_time(span.end_time_unix_nano))
                        } else {
                            None
                        },
                        status: convert_status(span.status),
                        attributes,
                        service_name: service_name.clone(),
                    };

                    // Send SpanEnded event (OTLP typically sends completed spans)
                    let _ = self.tx.send(TelemetryEvent::SpanEnded(span_data));
                }
            }
        }

        Ok(Response::new(ExportTraceServiceResponse {
            partial_success: None,
        }))
    }
}

/// Configuration for the OTLP receiver
#[derive(Debug, Clone)]
pub struct ReceiverConfig {
    pub grpc_addr: SocketAddr,
}

impl Default for ReceiverConfig {
    fn default() -> Self {
        Self {
            grpc_addr: "127.0.0.1:4317".parse().unwrap(),
        }
    }
}

/// Handle for the running receiver
pub struct ReceiverHandle {
    pub events_rx: mpsc::UnboundedReceiver<TelemetryEvent>,
}

/// Start the OTLP receiver server
pub async fn start_receiver(config: ReceiverConfig) -> anyhow::Result<ReceiverHandle> {
    let (tx, rx) = mpsc::unbounded_channel();
    let trace_service = OtlpTraceReceiver::new(tx);

    let addr = config.grpc_addr;

    // Spawn the gRPC server in a background task
    tokio::spawn(async move {
        if let Err(e) = Server::builder()
            .add_service(TraceServiceServer::new(trace_service))
            .serve(addr)
            .await
        {
            eprintln!("OTLP receiver error: {}", e);
        }
    });

    Ok(ReceiverHandle { events_rx: rx })
}

/// Create a mock telemetry stream for demo/testing purposes
pub fn mock_telemetry_stream() -> mpsc::UnboundedReceiver<TelemetryEvent> {
    let (tx, rx) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        use std::time::SystemTime;

        let services = ["api-gateway", "user-service", "auth-service", "database"];
        let operations = [
            "HTTP GET /api/users",
            "authenticate",
            "query users",
            "validate token",
            "fetch profile",
            "cache lookup",
            "serialize response",
        ];

        let mut span_counter = 0u64;
        let mut trace_counter = 0u64;

        loop {
            tokio::time::sleep(Duration::from_millis(500 + rand_delay())).await;

            // Generate a trace with 2-4 spans
            trace_counter += 1;
            let trace_id = format!("{:032x}", trace_counter);
            let num_spans = 2 + (trace_counter % 3) as usize;
            let service = services[(trace_counter as usize) % services.len()];

            let mut parent_id: Option<String> = None;

            for i in 0..num_spans {
                span_counter += 1;
                let span_id = format!("{:016x}", span_counter);
                let op = operations[(span_counter as usize) % operations.len()];
                let start = SystemTime::now();

                // Simulate span duration
                let duration_ms = 10 + (span_counter % 200);
                let end = start + Duration::from_millis(duration_ms);

                // Random status (mostly OK, occasional error)
                let status = if span_counter.is_multiple_of(10) {
                    SpanStatus::Error
                } else {
                    SpanStatus::Ok
                };

                let span = SpanData {
                    trace_id: trace_id.clone(),
                    span_id: span_id.clone(),
                    parent_span_id: parent_id.clone(),
                    name: op.to_string(),
                    kind: if i == 0 {
                        SpanKind::Server
                    } else {
                        SpanKind::Internal
                    },
                    start_time: start,
                    end_time: Some(end),
                    status,
                    attributes: HashMap::new(),
                    service_name: service.to_string(),
                };

                let _ = tx.send(TelemetryEvent::SpanEnded(span));
                parent_id = Some(span_id);

                // Small delay between spans
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
    });

    rx
}

fn rand_delay() -> u64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 500) as u64
}
