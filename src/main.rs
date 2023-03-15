use std::fs;

use simd_json::ValueAccess;

fn main() {
    unsafe {
        let mut fs1 = fs::read_to_string("0.json").unwrap();
        let mut fs2 = fs::read_to_string("1.json").unwrap();
        let mut fs3 = fs::read_to_string("2.json").unwrap();

        let js1: simd_json::OwnedValue = simd_json::from_str(&mut fs1).unwrap();
        let js2: simd_json::OwnedValue = simd_json::from_str(&mut fs2).unwrap();
        let js3: simd_json::OwnedValue = simd_json::from_str(&mut fs3).unwrap();

        println!(
            "{}",
            js1.as_array().unwrap().len()
                + js2.as_array().unwrap().len()
                + js3.as_array().unwrap().len()
        );
    }
}

fn split_json_file() {
    let fs = fs::read_to_string("namuwiki_202103012.json").unwrap();
    let json: serde_json::Value = serde_json::from_str(&fs).unwrap();
    let arr = json.as_array().unwrap();

    let length = arr.len();

    let mut r1: Vec<&serde_json::Value> = Vec::new();
    let mut r2: Vec<&serde_json::Value> = Vec::new();
    let mut r3: Vec<&serde_json::Value> = Vec::new();

    for i in 0..length {
        let x = match i % 3 {
            0 => &mut r1,
            1 => &mut r2,
            2 => &mut r3,
            _ => panic!(),
        }
        .push(&arr[i]);
    }

    fs::write("0.json", &serde_json::to_string(&r1).unwrap()).unwrap();
    fs::write("1.json", &serde_json::to_string(&r2).unwrap()).unwrap();
    fs::write("2.json", &serde_json::to_string(&r3).unwrap()).unwrap();
}
