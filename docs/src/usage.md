# Usage

## Basic usage

```rust
use expunge::Expunge;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Expunge)]
struct User {
  #[expunge(skip)] // skipped fields are not transformed
  id: i64,
  #[expunge(as = "Randy".to_string())]
  first_name: String,
  #[expunge(as = "Lahey".to_string())]
  last_name: String,
  #[expunge(with = sha256::digest)]
  date_of_birth: String,
  latitude: f64,
  longitude: f64,
  #[expunge(as = "<expunged>".to_string(), zeroize)]
  password_hash: String,
}

let user = User{
  id: 101,
  first_name: "Ricky".to_string(),
  last_name: "LaFleur".to_string(),
  date_of_birth: "02/02/1960".to_string(),
  latitude: 45.0778,
  longitude: 63.546,
  password_hash: "2f089e52def4cec8b911883fecdd6d8febe9c9f362d15e3e33feb2c12f07ccc1".to_string(),
};

let expunged_user = user.expunge();

let output = serde_json::to_string_pretty(&expunged_user).expect("should serialize");

assert_eq!(r#"{
  "id": 101,
  "first_name": "Randy",
  "last_name": "Lahey",
  "date_of_birth": "eeb98c815ae11240b563892c52c8735472bb8259e9a6477e179a9ea26e7a695a",
  "latitude": 0.0,
  "longitude": 0.0,
  "password_hash": "<expunged>"
}"#,
  output,
)
```
