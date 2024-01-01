use redact::Redact;
use sha256::digest;

fn assert_default<T>(got: T)
where
    T: Default + std::cmp::PartialEq + std::fmt::Debug,
{
    assert_eq!(T::default(), got);
}

#[test]
fn it_works() {
    #[derive(Clone, Redact)]
    struct User {
        #[redact]
        pub first_name: String,
        #[redact(as = "anon.".to_string())]
        pub last_name: String,
        #[redact(with = digest)]
        pub address: String,
        pub id: u64,
        pub location: Location,
    }

    #[derive(Clone, Redact)]
    struct Location {
        #[redact]
        city: String,
    }

    let user = User {
        first_name: "Bob".to_string(),
        last_name: "Smith".to_string(),
        address: "101 Some Street".to_string(),
        id: 99,
        location: Location {
            city: "New York".to_string(),
        },
    };

    let original = user.clone();

    let redacted = user.redact();

    assert_eq!("", redacted.first_name);
    assert_eq!(
        "", redacted.location.city,
        "it should redact nested structs"
    );

    assert_eq!(
        "anon.", redacted.last_name,
        "the `as` attribute can be used to provide a literal value"
    );
    assert_eq!(
        "75f6ac468f71b588f1f6e5d10e468efffab086a9e440c378d8018a7b3ff28b45", redacted.address,
        "the `with` attribute can be used to hash etc"
    );
    assert_eq!(
        original.id, redacted.id,
        "fields without the redact attribute should be left as is"
    );
}
