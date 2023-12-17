use redact::Redact;

fn hasher<P: AsRef<str>>(s: P) -> P {
    s
}

#[test]
fn it_works() {
    #[derive(Redact)]
    struct User {
        #[redact]
        pub first_name: String,
        #[redact(as = "anon.")]
        pub last_name: String,
        #[redact(with = hasher)]
        pub address: String,
        pub id: u64,
    }

    let user = User {
        first_name: "Bob".to_string(),
        last_name: "Smith".to_string(),
        address: "101 Some Street".to_string(),
        id: 99,
    };

    let redacted = user.redact();

    assert!(redacted.first_name.is_empty())
}
