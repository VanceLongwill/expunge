use expunge::Expunge;

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
    #[derive(PartialEq, Debug, Clone, Expunge)]
    enum SensitiveNested {
        Name(#[expunge] String, i32),
    }

    #[derive(Clone, Debug, PartialEq)]
    struct UnitStruct;

    impl Expunge for UnitStruct {
        fn expunge(self) -> Self
        where
            Self: Sized,
        {
            self
        }
    }

    #[derive(PartialEq, Debug, Clone, Expunge)]
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

    #[derive(PartialEq, Debug, Clone, Default)]
    struct Unexpungeable {
        name: String,
    }

    #[derive(PartialEq, Debug, Clone, Expunge, Default)]
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
