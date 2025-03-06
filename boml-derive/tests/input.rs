use boml::prelude::*;
use boml::{FromToml, TomlTryInto, FromTomlError};
use boml_derive::{FromToml, boml};


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
#[derive(FromToml, Debug)]
struct Example<'a> {
    a: i64,
    b: String,
    c: bool,
    d: f64,
    e: &'a str,    
}

#[allow(dead_code)]
#[derive(FromToml, Debug)]
struct Nested<'a> {
    a: i64,
    b: &'a str,
}

#[allow(dead_code)]
#[derive(FromToml, Debug)]
#[boml(untagged)]
enum Test<'a> {
    A(i64),
    B(&'a str, i64),
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
    let example = Example::from_toml(Some(&v)).unwrap();
    
    println!("{:?}", example);
}