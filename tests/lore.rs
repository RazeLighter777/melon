use melon::lore;
use serde::Deserialize;

#[derive(Deserialize)]
struct TestLore {
    value: i32,
}

impl lore::LoreType for TestLore {}

#[test]
fn lore_build() -> () {
    lore::LorebookBuilder::new()
        .register::<TestLore>()
        .build(std::path::Path::new("./tests/lore"))
        .unwrap();
}

#[test]
fn lore_get() -> () {
    let x = lore::LorebookBuilder::new()
        .register::<TestLore>()
        .build(std::path::Path::new("./tests/lore"))
        .unwrap();
    assert_eq!(
        x.get::<TestLore>(lore::Tags::new().with("tag1").with("tag2").with("tag3"))
            .unwrap()
            .value,
        32
    );
}

#[test]
fn lore_get_all() -> () {
    let x = lore::LorebookBuilder::new()
        .register::<TestLore>()
        .build(std::path::Path::new("./tests/lore"))
        .unwrap();
    assert_eq!(
        2,
        x.get_all_with_tag::<TestLore>(&lore::Tags::new().with("tag2"))
            .unwrap()
            .len()
    );
}
