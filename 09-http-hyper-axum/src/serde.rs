use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let point = Point { x: 1, y: 2 };

    // DS → JSON string = serialization
    // Serialization is the process of converting a data structure (like a struct) into a format
    // that can be easily stored or transmitted (like a JSON string).
    // MOSTLY used in converting a struct into a JSON string to send as response body.
    let serialized = serde_json::to_string(&point).unwrap();
    println!("serialized = {}", serialized);


    // JSON string → DS = deserialization
    // Deserialization is the reverse process of serialization, where you take a serialized format
    // (like a JSON string) and convert it back into a data structure (like a struct).
    // MOSTLY used in converting request body (JSON) into a struct.
    let deserialized: Point = serde_json::from_str(&serialized).unwrap();
    println!("deserialized = {:?}", deserialized);
}