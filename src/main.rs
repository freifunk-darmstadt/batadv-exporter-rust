#[macro_use]
extern crate prometheus;
#[macro_use]
extern crate clap;
extern crate iron;


use iron::prelude::*;
use iron::status;


use prometheus::{Gauge, TextEncoder, Encoder};
use std::process::Command;
use std::str::FromStr;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod util;

struct DeviceStatistics {
    device: String,
    tx_packets_total: Gauge,
    tx_bytes_total: Gauge,
    tx_dropped_packets_total: Gauge,
    rx_packets_total: Gauge,
    rx_bytes_total: Gauge,
//    forward_packets_total: Gauge,
//    forward_bytes_total: Gauge,
//    mgmt_tx_total: Gauge,
//    mgmt_tx_bytes_total: Gauge,
//    mgmt_rx_counter: Gauge,
//    mgmt_rx_bytes_total: Gauge,
//    frag_tx_packets_total: Gauge,
//    frag_tx_bytes_total: Gauge,
//    frag_rx_packets_total: Gauge,
//    frag_rx_bytes_total: Gauge,
//    frag_fwd_packets_total: Gauge,
//    tt_request_tx_packets_total: Gauge,
//    tt_request_rx_packets_total: Gauge,
//    tt_response_rx_packets_total: Gauge,
//    tt_roam_adv_tx_packets_total: Gauge,
//    tt_roam_adv_rx_packets_total: Gauge,
//    dat_get_tx_packets_total: Gauge,
//    dat_get_rx_packets_total: Gauge,
//    dat_put_tx_packets_total: Gauge,
//    dat_put_rx_packets_total: Gauge,
//    dat_cached_reply_tx_packets_total: Gauge
}

impl DeviceStatistics {
    fn new(device: &str) -> Result<DeviceStatistics, String> {
        let d = DeviceStatistics {
            device: String::from_str(device).unwrap(),
            tx_packets_total: register_gauge!(opts!(
                "tx_packets_total",
                "tx packet gauge",
                labels!{"device" => device,}
            )).unwrap(),
            tx_bytes_total: register_gauge!(opts!(
                "tx_bytes_total",
                "tx bytes gauge",
                labels!{"device" => device,}
            )).unwrap(),
            tx_dropped_packets_total: register_gauge!(opts!(
                "tx_dropped_packets_total",
                "tx dropped packets gauge",
                labels!{"device" => device,}
            )).unwrap(),
            rx_packets_total: register_gauge!(opts!(
                "tx_packets_total",
                "tx packets gauge",
                labels!{"device" => device,}
            )).unwrap(),
            rx_bytes_total: register_gauge!(opts!(
                "tx_bytes_total",
                "tx bytes gauge",
                labels!{"device" => device,}
            )).unwrap(),
        };
        Ok(d)
    }

    fn update(&mut self) -> std::io::Result<()> {
        let raw_statistics = get_batadv_statistics(&self.device)?;
        let statistics = parse_batadv_statistics(&raw_statistics).unwrap();
        println!("statistics: {:?}", statistics);

        let zero: f64 = 0.0;

        self.tx_packets_total.set(
            *statistics.get("tx").unwrap_or(&zero),
        );
        self.tx_bytes_total.set(
            *statistics.get("tx_bytes").unwrap_or(
                &zero,
            ),
        );
        self.tx_dropped_packets_total.set(*statistics
            .get("tx_dropped")
            .unwrap_or(&zero));
        self.rx_packets_total.set(
            *statistics.get("rx").unwrap_or(&zero),
        );
        self.rx_bytes_total.set(
            *statistics.get("rx_bytes").unwrap_or(
                &zero,
            ),
        );
        Ok(())
    }
}


fn get_batadv_statistics(device: &str) -> std::io::Result<String> {
    let output = match Command::new("batctl").args(&["-m", device, "s"]).output() {
        Ok(o) => o,
        Err(e) => return Err(e),
    };

    Ok(String::from_utf8(output.stdout).unwrap())
}

fn parse_batadv_statistics(stats: &str) -> Result<HashMap<String, f64>, String> {
    let values: Vec<(&str, f64)> = stats
        .split("\n")
        .map(|l| l.trim())
        .filter(|l| l.len() > 0)
        .map(|l| {
            let parts: Vec<&str> = l.split_terminator(":").collect();
            parts
        })
        .filter(|parts| parts.len() == 2)
        .map(|parts| {
            (parts[0].trim(), f64::from_str(parts[1].trim()).unwrap())
        })
        .collect();

    let mut map: HashMap<String, f64> = HashMap::new();
    for (key, value) in values {
        map.insert(String::from(key), value);
    }
    Ok(map)
}
fn render_prometheus() -> Vec<u8> {
    let encoder = TextEncoder::new();
    let metric_familys = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_familys, &mut buffer).unwrap();
    buffer
}

fn main() {

    let matches = clap::App::new("batadv-prometheus-rust")
        .version(crate_version!())
        .author(crate_authors!())
        .about("provides batadv prometheus metrics")
        .arg(
            clap::Arg::with_name("BATMAN_IFACE")
                .help("the subject batadvd interface")
                .required(true),
        )
        .get_matches();

    let device_name = matches.value_of("BATMAN_IFACE").unwrap();

    if !util::which("batctl") {
        println!("batctl not found in path.");
        std::process::exit(1);
    }

    let device = DeviceStatistics::new(device_name).unwrap();

    let addr = "[::1]:12345";

    let devices: Arc<Mutex<Vec<DeviceStatistics>>> = Arc::new(Mutex::new(vec![device]));

    {
        let devices_arc = devices.clone();
        Iron::new(move |req: &mut Request| {
            // FIXME: parse request headers

            {
                // update statistics
                // FIXME: add some grace period since the last update
                let mut guard = devices_arc.lock().unwrap();
                for ref mut device in &mut guard.iter_mut() {
                    device.update().unwrap();
                }
            }

            Ok(Response::with((status::Ok, render_prometheus())))
        }).http(addr)
            .unwrap();
    }
}
