use convert_case::{Case, Casing};
use serde_json::value::{to_value, Value};
use std::collections::HashMap;
use tera::{try_get_value, Tera};

pub fn register_filter(tera: &mut Tera) {
    tera.register_filter("repeat", repeat);
    tera.register_filter("as_upper_camel_case", as_upper_camel_case);
    tera.register_filter("as_snake_case", as_snake_case);
    tera.register_filter("to_rust_type", to_rust_type);
    tera.register_filter("to_rust_initialize", to_rust_initialize);
}

pub fn as_upper_camel_case(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    let s = try_get_value!("as_upper_camel_case", "value", String, value);

    Ok(to_value(s.to_case(Case::UpperCamel)).unwrap())
}

pub fn as_snake_case(value: &Value, _: &HashMap<String, Value>) -> tera::Result<Value> {
    let s = try_get_value!("as_snake_case", "value", String, value);

    Ok(to_value(s.to_case(Case::Snake)).unwrap())
}

pub fn to_rust_type(value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    let s = try_get_value!("to_rust_type", "value", String, value);

    let is_nullable = match args.get("is_nullable") {
        Some(val) => try_get_value!("to_rust_type", "is_nullable", bool, val),
        None => {
            return Err(tera::Error::msg(
                "Filter `to_rust_type` expected an arg called `is_nullable`",
            ))
        }
    };

    let t = if is_uuid(&s) {
        String::from("uuid::Uuid")
    } else if is_datetime(&s) {
        String::from("chrono::NaiveDateTime")
    } else if is_string(&s) {
        String::from("String")
    } else if is_bool(&s) {
        String::from("bool")
    } else if is_i8(&s) {
        String::from("i8")
    } else if is_u8(&s) {
        String::from("u8")
    } else if is_i16(&s) {
        String::from("i16")
    } else if is_u16(&s) {
        String::from("u16")
    } else if is_i32(&s) {
        String::from("i32")
    } else if is_u32(&s) {
        String::from("u32")
    } else {
        s
    };

    let t = if is_nullable {
        format!("Option<{t}>")
    } else {
        t
    };

    Ok(to_value(t).unwrap())
}

pub fn to_rust_initialize(value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    let s = try_get_value!("to_rust_initialize", "value", String, value);

    let is_nullable = match args.get("is_nullable") {
        Some(val) => try_get_value!("to_rust_initialize", "is_nullable", bool, val),
        None => {
            return Err(tera::Error::msg(
                "Filter `to_rust_initialize` expected an arg called `is_nullable`",
            ))
        }
    };

    let t = if is_uuid(&s) {
        String::from("uuid::Uuid::new_v4()")
    } else if is_datetime(&s) {
        String::from("chrono::Utc::now().naive_utc()")
    } else if is_i8(&s) {
        String::from("0")
    } else if is_u8(&s) {
        String::from("0")
    } else if is_i16(&s) {
        String::from("0")
    } else if is_u16(&s) {
        String::from("0")
    } else if is_i32(&s) {
        String::from("0")
    } else if is_u32(&s) {
        String::from("0")
    } else {
        s
    };

    let t = if is_nullable { format!("Some({t})") } else { t };

    Ok(to_value(t).unwrap())
}

fn is_uuid(value: &str) -> bool {
    value == "binary(16)"
}

fn is_datetime(value: &str) -> bool {
    value == "datetime"
}

fn is_string(value: &str) -> bool {
    value.starts_with("varchar") || value.starts_with("char") || value.starts_with("text")
}

fn is_bool(value: &str) -> bool {
    value.starts_with("tinyint(1)")
}

fn is_i8(value: &str) -> bool {
    value.starts_with("tinyint") && !value.ends_with("unsigned")
}

fn is_u8(value: &str) -> bool {
    value.starts_with("tinyint") && value.ends_with("unsigned")
}

fn is_i16(value: &str) -> bool {
    value.starts_with("smallint") && !value.ends_with("unsigned")
}

fn is_u16(value: &str) -> bool {
    value.starts_with("smallint") && value.ends_with("unsigned")
}

fn is_i32(value: &str) -> bool {
    value.starts_with("int") && !value.ends_with("unsigned")
}

fn is_u32(value: &str) -> bool {
    value.starts_with("int") && value.ends_with("unsigned")
}

pub fn repeat(value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    let s = try_get_value!("repeat", "value", String, value);
    let count = match args.get("count") {
        Some(val) => try_get_value!("repeat", "count", usize, val),
        None => {
            return Err(tera::Error::msg(
                "Filter `repeat` expected an arg called `count`",
            ))
        }
    };

    Ok(to_value(std::iter::repeat(s).take(count).collect::<Vec<String>>()).unwrap())
}
