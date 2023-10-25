
use json_lib2::json_lib2::Json;

fn main() {
    let mut j = Json::new();

    let str = r#"{
    "str": "hallo",
    "num": -12.0,
    "b" : true,
    "b2" : false,
    "nil" : null
}"#;
    j.parse(str);
}

