use expunge::Expunge;
use serde::Deserialize;

#[cfg(test)]
mod buf {
    use std::io::{BufRead, BufReader};
    use std::sync::{Arc, Mutex};

    /// Simple in memory buffer for testing logs
    #[derive(Default, Clone)]
    pub struct Buf(Arc<Mutex<Vec<u8>>>);

    impl std::io::Write for Buf {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().write(buf)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.0.lock().unwrap().flush()
        }
    }

    impl Buf {
        pub fn inner(&self) -> Vec<u8> {
            self.0.lock().unwrap().to_vec()
        }

        pub fn lines(&self) -> Vec<String> {
            let raw = self.inner();
            let output = BufReader::new(raw.as_slice());
            output.lines().collect::<Result<Vec<String>, _>>().unwrap()
        }
    }
}

#[test]
fn it_derives_logging_with_slog() {
    use crate::buf::Buf;
    use serde::Serialize;
    use slog::{info, o, Drain, Logger};
    use std::sync::Mutex;

    #[derive(Clone, Expunge, Deserialize, Serialize, PartialEq, Eq)]
    #[expunge(slog)]
    struct Location {
        #[expunge(as = "<expunged>".to_string())]
        city: String,
    }

    let loc = Location {
        city: "New York".to_string(),
    };

    let buf = Buf::default();
    let drain = Mutex::new(slog_json::Json::default(buf.clone())).fuse();
    let logger = Logger::root(drain, o!());

    info!(logger, "it should log"; "location" => loc.clone());

    #[derive(Deserialize)]
    struct Log {
        location: Location,
    }

    let lines = buf.lines();
    println!("{}", lines.join("\n"));

    let got: Log = serde_json::from_str(&lines[0]).unwrap();
    assert_eq!(
        loc.clone().expunge(),
        got.location,
        "the slogged value should be expunged"
    );
}

#[test]
fn it_derives_logging_with_slog_enum() {
    use crate::buf::Buf;
    use serde::Serialize;
    use slog::{info, o, Drain, Logger};
    use std::sync::Mutex;

    #[derive(Clone, Expunge, Deserialize, Serialize, PartialEq, Eq)]
    #[expunge(slog)]
    enum LocationType {
        #[expunge(as = "<expunged>".to_string())]
        City(String),
        #[expunge]
        Address {
            #[expunge(as = "line1".to_string())]
            line1: String,
            #[expunge(as = "line2".to_string())]
            line2: String,
            #[expunge(as = "line3".to_string())]
            line3: String,
        },
    }

    let buf = Buf::default();
    let drain = Mutex::new(slog_json::Json::default(buf.clone())).fuse();
    let logger = Logger::root(drain, o!());

    let city = LocationType::City("New York".to_string());
    info!(logger, "it should log city"; "location" => city.clone());
    let address = LocationType::Address {
        line1: "101 Some street".to_string(),
        line2: "Some Town".to_string(),
        line3: "Some Province".to_string(),
    };
    info!(logger, "it should log address"; "location" => address.clone());

    #[derive(Deserialize)]
    struct Log {
        location: LocationType,
    }

    let lines = buf.lines();
    println!("{}", lines.join("\n"));

    let got: Log = serde_json::from_str(&lines[0]).unwrap();
    assert_eq!(
        city.clone().expunge(),
        got.location,
        "the slogged value for city should be expunged"
    );

    let got: Log = serde_json::from_str(&lines[1]).unwrap();
    assert_eq!(
        address.clone().expunge(),
        got.location,
        "the slogged value for address should be expunged"
    );
}

#[test]
fn it_works_struct() {
    #[derive(Clone, Expunge)]
    struct User<G> {
        #[expunge]
        pub first_name: String,
        #[expunge]
        pub middle_name: Option<String>,
        #[expunge(as = "anon.".to_string())]
        pub last_name: String,
        #[expunge(with = sha256::digest)]
        pub address: String,
        pub id: u64,
        #[expunge]
        pub location: Location,
        #[expunge]
        pub initial_location: G,
        #[allow(dead_code)]
        #[expunge(ignore)]
        pub some_unit: UnitStruct,
    }

    #[derive(Clone)]
    struct UnitStruct;

    impl Expunge for UnitStruct {
        fn expunge(self) -> Self
        where
            Self: Sized,
        {
            self
        }
    }

    #[derive(Clone, Expunge)]
    struct Location {
        #[expunge]
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
        some_unit: UnitStruct,
    };

    let original = user.clone();

    let expunged = user.expunge();

    assert_eq!("", expunged.first_name);
    assert_eq!(
        "", expunged.location.city,
        "it should expunge nested structs"
    );

    assert_eq!(
        "", expunged.initial_location.city,
        "it should expunge generic values"
    );

    assert_eq!(
        Some("".to_string()),
        expunged.middle_name,
        "it should expunge optional values"
    );

    assert_eq!(
        "anon.", expunged.last_name,
        "the `as` attribute can be used to provide a literal value"
    );
    assert_eq!(
        "75f6ac468f71b588f1f6e5d10e468efffab086a9e440c378d8018a7b3ff28b45", expunged.address,
        "the `with` attribute can be used to hash etc"
    );
    assert_eq!(
        original.id, expunged.id,
        "fields without the expunge attribute should be left as is"
    );
}

#[test]
fn it_works_unnamed_struct() {
    #[derive(Expunge)]
    struct User(String, #[expunge] Location);

    #[derive(Expunge)]
    struct Location {
        #[expunge]
        city: String,
    }

    let user = User(
        "Bob".to_string(),
        Location {
            city: "New York".to_string(),
        },
    );

    let expunged = user.expunge();

    assert_eq!("Bob", expunged.0);
    assert_eq!("", expunged.1.city,);
}

#[test]
fn it_works_struct_all() {
    #[derive(Clone, Expunge)]
    #[expunge(all)]
    struct User<G> {
        pub first_name: String,
        pub middle_name: Option<String>,
        #[expunge(as = "anon.".to_string())]
        pub last_name: String,
        #[expunge(with = sha256::digest)]
        pub address: String,
        #[expunge(ignore)]
        pub id: u64,
        pub location: Location,
        pub initial_location: G,
    }

    #[derive(Clone, Expunge)]
    struct Location {
        #[expunge]
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

    let expunged = user.expunge();

    assert_eq!("", expunged.first_name);
    assert_eq!(
        "", expunged.location.city,
        "it should expunge nested structs"
    );

    assert_eq!(
        "", expunged.initial_location.city,
        "it should expunge generic values"
    );

    assert_eq!(
        Some("".to_string()),
        expunged.middle_name,
        "it should expunge optional values"
    );

    assert_eq!(
        "anon.", expunged.last_name,
        "the `as` attribute can be used to provide a literal value"
    );
    assert_eq!(
        "75f6ac468f71b588f1f6e5d10e468efffab086a9e440c378d8018a7b3ff28b45", expunged.address,
        "the `with` attribute can be used to hash etc"
    );
    assert_eq!(
        original.id, expunged.id,
        "fields without the expunge attribute should be left as is"
    );
}

#[test]
fn it_works_enum() {
    #[derive(PartialEq, Clone, Expunge)]
    enum SensitiveNested {
        Name(#[expunge] String, i32),
    }

    #[derive(Clone, PartialEq)]
    struct UnitStruct;

    impl Expunge for UnitStruct {
        fn expunge(self) -> Self
        where
            Self: Sized,
        {
            self
        }
    }

    #[derive(PartialEq, Clone, Expunge)]
    enum SensitiveItem {
        Name(#[expunge] String, i32),
        DateOfBirth(String),
        BankDetails {
            #[expunge]
            account_number: i32,
        },
        Location(#[expunge] Location),
        #[expunge]
        Nested(SensitiveNested, i32),
        #[expunge]
        LocationHistory(Vec<Location>),
        #[expunge]
        WithUnit(i32, UnitStruct),
        #[expunge(as = Default::default())]
        DoesntImplementExpunge(Unexpungeable),
        #[expunge(as = i32::MAX, zeroize)]
        Zeroizable(i32),
        #[expunge(as = "99".to_string(), zeroize)]
        ZeroizableString(String),
    }

    #[derive(PartialEq, Clone, Default)]
    struct Unexpungeable {
        name: String,
    }

    #[derive(PartialEq, Clone, Expunge, Default)]
    struct Location {
        #[expunge]
        city: String,
    }

    let item = SensitiveItem::Name("Bob".to_string(), 1);

    let expunged = item.expunge();

    assert_eq!(SensitiveItem::Name("".to_string(), 1), expunged);

    let item = SensitiveItem::BankDetails {
        account_number: 123,
    };
    let expunged = item.expunge();
    assert_eq!(SensitiveItem::BankDetails { account_number: 0 }, expunged);

    let new_york = Location {
        city: "New York".to_string(),
    };
    let item = SensitiveItem::Location(new_york.clone());

    let expunged = item.expunge();
    assert_eq!(SensitiveItem::Location(Location::default()), expunged);

    let item = SensitiveItem::Nested(SensitiveNested::Name("Alice".to_string(), 1), 99);
    let expunged = item.expunge();
    assert_eq!(
        SensitiveItem::Nested(SensitiveNested::Name("".to_string(), 1), 0),
        expunged
    );

    let boston = Location {
        city: "Boston".to_string(),
    };
    let item = SensitiveItem::LocationHistory(vec![new_york, boston]);
    let expunged = item.expunge();
    assert_eq!(
        SensitiveItem::LocationHistory(vec![Location::default(), Location::default()],),
        expunged
    );

    let item = SensitiveItem::Zeroizable(12309812);
    let expunged = item.expunge();
    assert_eq!(SensitiveItem::Zeroizable(2147483647), expunged);

    let item = SensitiveItem::ZeroizableString("my_password".to_string());
    let expunged = item.expunge();
    assert_eq!(SensitiveItem::ZeroizableString("99".to_string()), expunged);
}

#[test]
fn it_works_enum_all() {
    #[derive(PartialEq, Clone, Expunge)]
    enum SensitiveNested {
        Name(#[expunge] String, i32),
    }

    #[derive(Clone, PartialEq)]
    struct UnitStruct;

    impl Expunge for UnitStruct {
        fn expunge(self) -> Self
        where
            Self: Sized,
        {
            self
        }
    }

    #[derive(PartialEq, Clone, Expunge)]
    #[expunge(all)]
    enum SensitiveItem {
        Name(String, i32),
        DateOfBirth(String),
        BankDetails {
            account_number: i32,
        },
        Location(Location),
        Nested(SensitiveNested, i32),
        LocationHistory(Vec<Location>),
        WithUnit(i32, UnitStruct),
        #[expunge(as = i32::MAX, zeroize)]
        Zeroizable(i32),
        #[expunge(as = "99".to_string(), zeroize)]
        ZeroizableString(String),
    }

    #[derive(PartialEq, Clone, Expunge, Default)]
    struct Location {
        #[expunge]
        city: String,
    }

    let item = SensitiveItem::Name("Bob".to_string(), 1);

    let expunged = item.expunge();

    assert_eq!(SensitiveItem::Name("".to_string(), 0), expunged);

    let item = SensitiveItem::BankDetails {
        account_number: 123,
    };
    let expunged = item.expunge();
    assert_eq!(SensitiveItem::BankDetails { account_number: 0 }, expunged);

    let new_york = Location {
        city: "New York".to_string(),
    };
    let item = SensitiveItem::Location(new_york.clone());

    let expunged = item.expunge();
    assert_eq!(SensitiveItem::Location(Location::default()), expunged);

    let item = SensitiveItem::Nested(SensitiveNested::Name("Alice".to_string(), 1), 99);
    let expunged = item.expunge();
    assert_eq!(
        SensitiveItem::Nested(SensitiveNested::Name("".to_string(), 1), 0),
        expunged
    );

    let boston = Location {
        city: "Boston".to_string(),
    };
    let item = SensitiveItem::LocationHistory(vec![new_york, boston]);
    let expunged = item.expunge();
    assert_eq!(
        SensitiveItem::LocationHistory(vec![Location::default(), Location::default()],),
        expunged
    );

    let item = SensitiveItem::Zeroizable(12309812);
    let expunged = item.expunge();
    assert_eq!(SensitiveItem::Zeroizable(2147483647), expunged);

    let item = SensitiveItem::ZeroizableString("my_password".to_string());
    let expunged = item.expunge();
    assert_eq!(SensitiveItem::ZeroizableString("99".to_string()), expunged);
}

#[test]
fn it_returns_boxed() {
    #[derive(Expunge)]
    struct Location {
        #[expunge]
        city: String,
    }

    let location = Box::new(Location {
        city: "New York".to_string(),
    });

    let _: Box<Location> = location.expunge();
}

#[test]
fn it_expunges_default() {
    #[derive(Default)]
    struct SomeData {
        pub name: String,
    }

    #[derive(Expunge)]
    struct Person {
        #[expunge(default)]
        data: SomeData,
    }

    let p = Person {
        data: SomeData {
            name: "John Smith".to_string(),
        },
    };

    assert_eq!(String::default(), p.expunge().data.name);
}

#[test]
fn it_allows_or_prevents_debug() {
    #[derive(Expunge)]
    struct ExpungeDebug {
        #[expunge]
        pub name: String,
    }

    let expunge_debug = ExpungeDebug {
        name: "John Smith".to_string(),
    };
    // debug is implemented by expunge
    assert_eq!("<expunged>", format!("{expunge_debug:?}"));

    #[derive(Debug, Expunge)]
    #[expunge(allow_debug)]
    struct CustomDebug {
        #[expunge]
        pub name: String,
    }

    let custom_debug = CustomDebug {
        name: "John Smith".to_string(),
    };
    // debug is manually derived
    assert_eq!(
        r#"CustomDebug { name: "John Smith" }"#,
        format!("{custom_debug:?}")
    );
}
