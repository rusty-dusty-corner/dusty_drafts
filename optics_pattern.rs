#![allow(dead_code)]

use std::marker::PhantomData as Ph;

// Simple structures to demonstrate optics
#[derive(Debug, Clone)]
struct User {
    name: String,
    age: u32,
    address: Address,
}

#[derive(Debug, Clone)]
struct Address {
    city: String,
    street: String,
}

#[derive(Debug)]
enum Status {
    Active(String),
    Inactive,
}

// Lens trait - fundamental interface for accessing structure fields
trait Lens<S, A> {
    fn get<'a>(&self, source: &'a S) -> &'a A;
    fn set(self, source: S, value: A) -> S;
}

// Prism trait - for working with enum variants
trait Prism<S, A> {
    fn preview<'a>(&self, source: &'a S) -> Option<&'a A>;
    fn review(self, value: A) -> S;
}

// Singleton types for each field
struct NameLens;
struct AgeLens;
struct AddressLens;
struct CityLens;
struct StreetLens;
struct ActiveStatusPrism;

// Lens implementations for User fields
impl Lens<User, String> for NameLens {
    fn get<'a>(&self, source: &'a User) -> &'a String {
        &source.name
    }

    fn set(self, mut source: User, value: String) -> User {
        source.name = value;
        source
    }
}

impl Lens<User, u32> for AgeLens {
    fn get<'a>(&self, source: &'a User) -> &'a u32 {
        &source.age
    }

    fn set(self, mut source: User, value: u32) -> User {
        source.age = value;
        source
    }
}

impl Lens<User, Address> for AddressLens {
    fn get<'a>(&self, source: &'a User) -> &'a Address {
        &source.address
    }

    fn set(self, mut source: User, value: Address) -> User {
        source.address = value;
        source
    }
}

// Lens implementations for Address fields
impl Lens<Address, String> for CityLens {
    fn get<'a>(&self, source: &'a Address) -> &'a String {
        &source.city
    }

    fn set(self, mut source: Address, value: String) -> Address {
        source.city = value;
        source
    }
}

impl Lens<Address, String> for StreetLens {
    fn get<'a>(&self, source: &'a Address) -> &'a String {
        &source.street
    }

    fn set(self, mut source: Address, value: String) -> Address {
        source.street = value;
        source
    }
}

// Prism implementation for Status enum
impl Prism<Status, String> for ActiveStatusPrism {
    fn preview<'a>(&self, source: &'a Status) -> Option<&'a String> {
        match source {
            Status::Active(s) => Some(s),
            Status::Inactive => None,
        }
    }

    fn review(self, value: String) -> Status {
        Status::Active(value)
    }
}

// Composition helper for Lenses
struct ComposedLens<L1, L2, I>(L1, L2, Ph<I>);

impl<S, I, B, L1, L2> Lens<S, B> for ComposedLens<L1, L2, I>
where
    L1: Lens<S, I>,
    L2: Lens<I, B>,
    I: 'static + Clone,
{
    fn get<'a>(&self, source: &'a S) -> &'a B {
        self.1.get(self.0.get(source))
    }

    fn set(self, source: S, value: B) -> S {
        let intermediate = self.0.get(&source).clone();
        let new_intermediate = self.1.set(intermediate, value);
        self.0.set(source, new_intermediate)
    }
}

// Helper function for lens composition
fn compose<L1, L2, I>(l1: L1, l2: L2) -> ComposedLens<L1, L2, I> {
    ComposedLens(l1, l2, Ph)
}

fn main() {
    // Example usage
    let user = User {
        name: "John".to_string(),
        age: 30,
        address: Address {
            city: "New York".to_string(),
            street: "Broadway".to_string(),
        },
    };

    // Using lenses
    let name = NameLens.get(&user);
    println!("Name: {}", name);

    // Composition example
    let composed_lens = compose(AddressLens, CityLens);
    let city = composed_lens.get(&user);
    println!("City: {}", city);

    // Modifying data
    let updated_user = NameLens.set(user.clone(), "Jack".to_string());
    println!("Updated name: {}", updated_user.name);

    // Prism example
    let status = Status::Active("online".to_string());
    let active_status = ActiveStatusPrism.preview(&status);
    println!("Active status: {:?}", active_status);
}
