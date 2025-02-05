#![feature(allocator_api)]

extern crate alloc;

mod store;
use rtori_os_fold_importer::{import_in, transform::transform_in};
use store::*;

const SIMPLE_EXPECTED: &'static str = include_str!("../testdata/simple.json");
const SIMPLE_FOLD: &'static str = include_str!("../../fold/testdata/simple.fold");

fn test_pair(
    expected: &str,
    fold: &str
) {
    let parsed_input = serde_json::from_str::<fold::File>(fold)
        .expect("source deserialization (json/fold file) failed");
    let decoded_expectation = serde_json::from_str::<Store>(expected)
        .expect("expectation deserialization (json) failed");

    let imported = {
        let allocator = alloc::alloc::Global;

        let transformed = transform_in(&parsed_input.key_frame, allocator)
            .expect("Transformation into importation input failed");

        let transformed_input = transformed.with_fold(&parsed_input.key_frame);

        let store = import_in(
            Store::with_size,
            &transformed_input,
            Default::default(),
            allocator,
        )
        .expect("import failed");

        store
    };

    {
        let imported_json = serde_json::to_string(&imported).unwrap();
        println!("{}", imported_json);
    }

    assert_json_diff::assert_json_eq!(decoded_expectation, imported);
}

#[test]
fn test_simple() {
    test_pair(SIMPLE_EXPECTED, SIMPLE_FOLD);
}
