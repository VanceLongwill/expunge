use redact::Redact;
use sha256::digest;

#[test]
fn it_works_struct() {
    #[derive(Clone, Redact)]
    struct User<G> {
        #[redact]
        pub first_name: String,
        #[redact]
        pub middle_name: Option<String>,
        #[redact(as = "anon.".to_string())]
        pub last_name: String,
        #[redact(with = digest)]
        pub address: String,
        pub id: u64,
        #[redact]
        pub location: Location,
        #[redact]
        pub initial_location: G,
    }

    #[derive(Clone, Redact)]
    struct Location {
        #[redact]
        city: String,
    }

    let user = User {
        first_name: "Bob".to_string(),
        middle_name: Some("James".to_string()),
        last_name: "Smith".to_string(),
        address: "101 Some Street".to_string(),
        id: 99,
        location: Location {
            city: "New York".to_string(),
        },
        initial_location: Location {
            city: "Los Angeles".to_string(),
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
        "", redacted.initial_location.city,
        "it should redact generic values"
    );

    assert_eq!(
        Some("".to_string()),
        redacted.middle_name,
        "it should redact optional values"
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

#[test]
fn it_works_unnamed_struct() {
    #[derive(Redact)]
    struct User(String, #[redact] Location);

    #[derive(Redact)]
    struct Location {
        #[redact]
        city: String,
    }

    let user = User(
        "Bob".to_string(),
        Location {
            city: "New York".to_string(),
        },
    );

    let redacted = user.redact();

    assert_eq!("Bob", redacted.0);
    assert_eq!("", redacted.1.city,);
}

#[test]
fn it_works_struct_all() {
    #[derive(Clone, Redact)]
    #[redact(all)]
    struct User<G> {
        pub first_name: String,
        pub middle_name: Option<String>,
        #[redact(as = "anon.".to_string())]
        pub last_name: String,
        #[redact(with = digest)]
        pub address: String,
        #[redact(ignore)]
        pub id: u64,
        pub location: Location,
        pub initial_location: G,
    }

    #[derive(Clone, Redact)]
    struct Location {
        #[redact]
        city: String,
    }

    let user = User {
        first_name: "Bob".to_string(),
        middle_name: Some("James".to_string()),
        last_name: "Smith".to_string(),
        address: "101 Some Street".to_string(),
        id: 99,
        location: Location {
            city: "New York".to_string(),
        },
        initial_location: Location {
            city: "Los Angeles".to_string(),
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
        "", redacted.initial_location.city,
        "it should redact generic values"
    );

    assert_eq!(
        Some("".to_string()),
        redacted.middle_name,
        "it should redact optional values"
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

#[test]
fn it_works_enum() {
    #[derive(PartialEq, Debug, Clone, Redact)]
    enum SensitiveNested {
        Name(#[redact] String, i32),
    }

    #[derive(PartialEq, Debug, Clone, Redact)]
    enum SensitiveItem {
        Name(#[redact] String, i32),
        DateOfBirth(String),
        BankDetails {
            #[redact]
            account_number: i32,
        },
        Location(#[redact] Location),
        #[redact]
        Nested(SensitiveNested, i32),
        #[redact]
        LocationHistory(Vec<Location>),
    }

    #[derive(PartialEq, Debug, Clone, Redact, Default)]
    struct Location {
        #[redact]
        city: String,
    }

    let item = SensitiveItem::Name("Bob".to_string(), 1);

    let redacted = item.redact();

    assert_eq!(SensitiveItem::Name("".to_string(), 1), redacted);

    let item = SensitiveItem::BankDetails {
        account_number: 123,
    };
    let redacted = item.redact();
    assert_eq!(SensitiveItem::BankDetails { account_number: 0 }, redacted);

    let new_york = Location {
        city: "New York".to_string(),
    };
    let item = SensitiveItem::Location(new_york.clone());

    let redacted = item.redact();
    assert_eq!(SensitiveItem::Location(Location::default()), redacted);

    let item = SensitiveItem::Nested(SensitiveNested::Name("Alice".to_string(), 1), 99);
    let redacted = item.redact();
    assert_eq!(
        SensitiveItem::Nested(SensitiveNested::Name("".to_string(), 1), 0),
        redacted
    );

    let boston = Location {
        city: "Boston".to_string(),
    };
    let item = SensitiveItem::LocationHistory(vec![new_york, boston]);
    let redacted = item.redact();
    assert_eq!(
        SensitiveItem::LocationHistory(vec![Location::default(), Location::default()],),
        redacted
    );
}
