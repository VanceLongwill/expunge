use expunge::Expunge;

#[derive(Expunge)]
#[expunge(as = "<redacted>".to_string())]
struct ConnectionInfo {
  username: String,
  password: String,
  host: String,
}
