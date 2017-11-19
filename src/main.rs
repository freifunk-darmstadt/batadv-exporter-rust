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
    forward_packets_total: Gauge,
    forward_bytes_total: Gauge,
    mgmt_tx_packets_total: Gauge,
    mgmt_tx_bytes_total: Gauge,
    mgmt_rx_packets_total: Gauge,
    mgmt_rx_bytes_total: Gauge,
    frag_tx_packets_total: Gauge,
    frag_tx_bytes_total: Gauge,
    frag_rx_packets_total: Gauge,
    frag_rx_bytes_total: Gauge,
    frag_fwd_packets_total: Gauge,

    tt_request_tx_packets_total: Gauge,
    tt_request_rx_packets_total: Gauge,
    tt_response_rx_packets_total: Gauge,
    tt_roam_adv_tx_packets_total: Gauge,
    tt_roam_adv_rx_packets_total: Gauge,
    dat_get_tx_packets_total: Gauge,
    dat_get_rx_packets_total: Gauge,
    dat_put_tx_packets_total: Gauge,
    dat_put_rx_packets_total: Gauge,
    dat_cached_reply_tx_packets_total: Gauge,
}


macro_rules! init_gauge {
    ($field_name:ident, $labels:expr) => {
        {
        let name = stringify!($field_name);
        let description = name.replace("_", " ");
        register_gauge!(opts!(
            name,
            &description,
            $labels
        )).unwrap()
        }
    }
}

impl DeviceStatistics {
    fn new(device: &str) -> Result<DeviceStatistics, String> {
        let labels = labels!{"device" => device,};
        let d = DeviceStatistics {
            device: String::from_str(device).unwrap(),
            tx_packets_total: init_gauge!(tx_packets_total, labels),
            tx_bytes_total: init_gauge!(tx_bytes_total, labels),
            tx_dropped_packets_total: init_gauge!(tx_dropped_packets_total, labels),
            rx_packets_total: init_gauge!(rx_packets_total, labels),
            rx_bytes_total: init_gauge!(rx_bytes_octal, labels),
            forward_packets_total: init_gauge!(forward_packets_total, labels),
            forward_bytes_total: init_gauge!(forward_bytes_total, labels),
            mgmt_tx_packets_total: init_gauge!(mgmt_tx_packets_total, labels),
            mgmt_tx_bytes_total: init_gauge!(mgmt_tx_bytes_total, labels),
            mgmt_rx_packets_total: init_gauge!(mgmt_rx_packets_total, labels),
            mgmt_rx_bytes_total: init_gauge!(mgmt_rx_bytes_total, labels),
            frag_tx_packets_total: init_gauge!(frag_tx_packets_total, labels),
            frag_tx_bytes_total: init_gauge!(frag_tx_bytes_total, labels),
            frag_rx_packets_total: init_gauge!(frag_rx_packets_total, labels),
            frag_rx_bytes_total: init_gauge!(frag_rx_bytes_total, labels),
            frag_fwd_packets_total: init_gauge!(frag_fwd_packets_total, labels),
            tt_request_tx_packets_total: init_gauge!(tt_request_tx_packets_total, labels),
            tt_request_rx_packets_total: init_gauge!(tt_request_rx_packets_total, labels),
            tt_response_rx_packets_total: init_gauge!(tt_response_rx_packets_total, labels),
            tt_roam_adv_tx_packets_total: init_gauge!(tt_roam_adv_tx_packets_total, labels),
            tt_roam_adv_rx_packets_total: init_gauge!(tt_roam_adv_rx_packets_total, labels),
            dat_get_tx_packets_total: init_gauge!(dat_get_tx_packets_total, labels),
            dat_get_rx_packets_total: init_gauge!(dat_get_rx_packets_total, labels),
            dat_put_tx_packets_total: init_gauge!(dat_put_tx_packets_total, labels),
            dat_put_rx_packets_total: init_gauge!(dat_put_rx_packets_total, labels),
            dat_cached_reply_tx_packets_total: init_gauge!(
                dat_cached_reply_tx_packets_total,
                labels
            ),
        };
        Ok(d)
    }

    fn update_batctl_statistics(&mut self) -> std::io::Result<()> {
        let raw_statistics = get_batctl_statistics(&self.device)?;
        let statistics = parse_batctl_statistics(&raw_statistics).unwrap();
        println!("statistics: {:?}", statistics);

        let zero: f64 = 0.0;

        let get = |key| match statistics.get(key) {
            Some(v) => *v,
            None => { println!("{} is undefined!", key); zero}
        }; //_or(&zero);

        self.tx_packets_total.set(get("tx"));
        self.tx_bytes_total.set(get("tx_bytes"));
        self.tx_dropped_packets_total.set(get("tx_dropped"));
        self.rx_packets_total.set(get("rx"));
        self.rx_bytes_total.set(get("tx_bytes"));
        self.forward_packets_total.set(get("forward"));
        self.forward_bytes_total.set(get("forward_bytes"));
        self.mgmt_tx_packets_total.set(get("mgmt_tx"));
        self.mgmt_tx_bytes_total.set(get("mgmt_tx_bytes"));
        self.mgmt_rx_packets_total.set(get("mgmt_rx"));
        self.mgmt_rx_bytes_total.set(get("mgmt_rx_bytes"));
        self.frag_tx_packets_total.set(get("frag_tx"));
        self.frag_tx_bytes_total.set(get("frag_tx_bytes"));
        self.frag_rx_packets_total.set(get("frag_rx"));
        self.frag_rx_bytes_total.set(get("frag_rx_bytes"));
        self.frag_fwd_packets_total.set(get("frag_fwd"));
        self.tt_request_tx_packets_total.set(get("tt_request_tx"));
        self.tt_request_rx_packets_total.set(get("tt_request_rx"));
        self.tt_response_rx_packets_total.set(get("tt_response_rx"));
        self.tt_roam_adv_tx_packets_total.set(get("tt_roam_adv_tx"));
        self.tt_roam_adv_rx_packets_total.set(get("tt_roam_adv_rx"));
        self.dat_get_tx_packets_total.set(get("dat_get_tx"));
        self.dat_get_rx_packets_total.set(get("dat_get_rx"));
        self.dat_put_tx_packets_total.set(get("dat_put_tx"));
        self.dat_put_rx_packets_total.set(get("dat_put_rx"));
        self.dat_cached_reply_tx_packets_total.set(get(
            "dat_cached_reply_tx",
        ));

        Ok(())

    }

    fn update_sysfs_statistics() -> std::io::Result<()> {
        // FIXME: implement
        Ok(()
    }

    fn update(&mut self) -> std::io::Result<()> {
        self.update_batctl_statistics()?;
        self.update_sysfs_statistics()?;
    }
}


fn get_batctl_statistics(device: &str) -> std::io::Result<String> {
    let output = match Command::new("batctl").args(&["-m", device, "s"]).output() {
        Ok(o) => o,
        Err(e) => return Err(e),
    };

    Ok(String::from_utf8(output.stdout).unwrap())
}

fn parse_batctl_statistics(stats: &str) -> Result<HashMap<String, f64>, String> {
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
