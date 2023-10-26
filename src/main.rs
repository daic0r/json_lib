
use json_lib2::json_lib2::Json;

fn main() {
    let mut j = Json::new();

    let str = r#"{
    "str": "hallo",
    "num": -12.0,
    "b" : true,
    "b2" : false,
    "nil" : null,
    "subObj" : {
        "one": 1,
        "two": 2,
        "subArr": [
            1,
            "yo",
            null,
            [
                "TestString"
            ]
        ]
    }
}"#;
    let b = j.parse(str);
    if !b {
        println!("Parse failed!");
        return;
    }
    
    dbg!(j.root);

}

