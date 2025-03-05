use boml::prelude::*;
use boml::{FromToml, TomlTryInto};
use boml_derive::FromToml;
use syn::token::Ne;


// #[derive(FromToml)]
// struct UnitStruct;

// #[derive(FromToml)]
// struct NamedStruct {
//     a: i32,
//     b: String,
//     c: bool,
//     d: f64,

// }

#[allow(dead_code)]
#[derive(FromToml, Debug, Clone)]
struct Example<'a, T: Clone> {
    a: i64,
    b: String,
    c: bool,
    d: f64,
    e: &'a str,
    f: T   
}

#[allow(dead_code)]
#[derive(FromToml, Debug, Clone)]
struct Nested<'a> {
    a: i64,
    b: &'a str,
}

fn main() {
    let toml = Toml::parse(r#"
        a = 42
        b = "Hello, World!"
        c = true
        d = 3.14
        e = "Hello, World!"
        
        [f]
        a = 42
        b = "Hello, World!"
    "#).unwrap();
    

    
    let v = TomlValue::Table(toml.into());
    let example = Example::<Nested<'_>>::from_toml(Some(&v)).unwrap();

    println!("{:?}", example);
}
