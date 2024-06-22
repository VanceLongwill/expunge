use expunge::Expunge;

#[derive(Expunge)]
struct UserLogin {
    username: String,
    #[expunge(as = "<redacted>".to_string(), zeroize)]
    password: String, // password will be scrubbed from memory after expunging
}
