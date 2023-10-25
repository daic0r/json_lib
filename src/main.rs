
use json_lib2::json_lib2::Json;

fn main() {
    let mut j = Json::new();

    let str = r#"{
    "str": "hallo",
    "num": -12.0,
}"#;
    j.parse(str);
}

