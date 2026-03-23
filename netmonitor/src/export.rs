use crate::core::domain::MonitoringSnapshot;
use anyhow::Result;
use serde::Serialize;
use std::io::Write;

pub trait Formatter {
    fn format(&self, snapshot: &MonitoringSnapshot, writer: &mut dyn Write) -> Result<()>;
}

pub struct JsonFormatter;

impl Formatter for JsonFormatter {
    fn format(&self, snapshot: &MonitoringSnapshot, writer: &mut dyn Write) -> Result<()> {
        let json = serde_json::to_string(snapshot)?;
        writeln!(writer, "{}", json)?;
        Ok(())
    }
}

#[derive(Serialize)]
struct CsvRow<'a> {
    timestamp: String,
    pid: u32,
    name: &'a str,
    context: String,
    up_bytes: u64,
    down_bytes: u64,
    up_rate_kb: f64,
    down_rate_kb: f64,
}

pub struct CsvFormatter {
    pub include_header: bool,
}

impl Formatter for CsvFormatter {
    fn format(&self, snapshot: &MonitoringSnapshot, writer: &mut dyn Write) -> Result<()> {
        let mut wtr = csv::WriterBuilder::new()
            .has_headers(self.include_header)
            .from_writer(writer);

        let timestamp = snapshot.timestamp.to_rfc3339();

        for proc in &snapshot.processes {
            let row = CsvRow {
                timestamp: timestamp.clone(),
                pid: proc.pid.0,
                name: &proc.name,
                context: proc.context.label(),
                up_bytes: proc.up.0,
                down_bytes: proc.down.0,
                up_rate_kb: proc.up_rate.to_kb(),
                down_rate_kb: proc.down_rate.to_kb(),
            };
            wtr.serialize(row)?;
        }
        wtr.flush()?;
        Ok(())
    }
}

pub struct PlainFormatter;

impl Formatter for PlainFormatter {
    fn format(&self, snapshot: &MonitoringSnapshot, writer: &mut dyn Write) -> Result<()> {
        writeln!(
            writer,
            "--- Snapshot at {} ---",
            snapshot.timestamp.to_rfc3339()
        )?;
        writeln!(
            writer,
            "{:<8} {:<20} {:<15} {:<10} {:<10}",
            "PID", "NAME", "CONTEXT", "UP/s", "DOWN/s"
        )?;

        let mut sorted_procs = snapshot.processes.clone();
        sorted_procs.sort_by(|a, b| (b.up_rate.0 + b.down_rate.0).cmp(&(a.up_rate.0 + a.down_rate.0)));

        for proc in sorted_procs.iter().take(20) {
            if proc.up_rate.0 > 0 || proc.down_rate.0 > 0 {
                writeln!(
                    writer,
                    "{:<8} {:<20} {:<15} {:<10} {:<10}",
                    proc.pid.0,
                    truncate(&proc.name, 20),
                    truncate(&proc.context.label(), 15),
                    proc.up_rate.to_string(),
                    proc.down_rate.to_string()
                )?;
            }
        }
        writeln!(writer)?;
        Ok(())
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}
