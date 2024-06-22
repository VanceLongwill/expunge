use expunge::Expunge;
use serde::{Deserialize, Serialize};
use slog::{info, o};
use slog::{Drain, Logger};
use std::sync::Mutex;

#[derive(Clone, Expunge, Deserialize, Serialize, PartialEq, Eq)] // must implement Serialize
#[expunge(slog)]
#[serde(rename_all = "snake_case")]
enum LocationType {
    #[expunge(as = "<expunged>".to_string())]
    City(String),
    Address {
        #[expunge(as = "line1".to_string())]
        line1: String,
        #[expunge(as = "line2".to_string())]
        line2: String,
    },
}

fn main() {
    let buf = vec![];
    let drain = Mutex::new(slog_json::Json::default(buf)).fuse();
    let logger = Logger::root(drain, o!());

    // Just log as is and it will be automatically expunged

    let city = LocationType::City("New York".to_string());
    info!(logger, "it should log city"; "location" => city);

    let address = LocationType::Address {
        line1: "101 Some street".to_string(),
        line2: "Some Town".to_string(),
    };
    info!(logger, "it should log address"; "location" => address);

    // {"msg":"it should log city","location":{"city":"<expunged>"},"level":"INFO","ts":"2024-02-04T12:55:28.627592Z"}
    // {"msg":"it should log address","location":{"address":{"line1":"line1","line2":"line2"}},"level":"INFO","ts":"2024-02-04T12:55:28.627627Z"}
}
