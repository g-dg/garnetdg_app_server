use std::rc::Rc;

use crate::helpers::tlru_cache::TLRUCache;

#[test]
fn insert_get_remove_no_eviction() {
    let mut cache: TLRUCache<String, String> = TLRUCache::new(None, None, None);

    // insert test1 and test2
    cache.insert(String::from("test1"), String::from("test1test1"));
    cache.insert(String::from("test2"), String::from("test2test2"));

    assert_eq!(
        cache.get(&String::from("test1")),
        Some(Rc::new(String::from("test1test1"))),
        "Test1 value not found after insertion"
    );
    assert_eq!(
        cache.get(&String::from("test2")),
        Some(Rc::new(String::from("test2test2"))),
        "Test2 value not found after insertion"
    );
    assert_eq!(
        cache.get(&String::from("test3")),
        None,
        "Test3 value found before insertion"
    );

    // insert test3
    cache.insert(String::from("test3"), String::from("test3test3"));

    assert_eq!(
        cache.get(&String::from("test3")),
        Some(Rc::new(String::from("test3test3"))),
        "Test3 value not found after insertion"
    );

    // remove test2
    cache.remove(&String::from("test2"));

    assert_eq!(
        cache.get(&String::from("test1")),
        Some(Rc::new(String::from("test1test1"))),
        "Test1 value not found after deletion of Test2"
    );
    assert_eq!(
        cache.get(&String::from("test2")),
        None,
        "Test2 value found after deletion"
    );
    assert_eq!(
        cache.get(&String::from("test3")),
        Some(Rc::new(String::from("test3test3"))),
        "Test3 value not found after deletion of Test2"
    );

    // remove test3
    cache.remove(&String::from("test3"));

    assert_eq!(
        cache.get(&String::from("test1")),
        Some(Rc::new(String::from("test1test1"))),
        "Test1 value not found after deletion of Test3"
    );
    assert_eq!(
        cache.get(&String::from("test2")),
        None,
        "Test2 value found after deletion"
    );
    assert_eq!(
        cache.get(&String::from("test3")),
        None,
        "Test3 value found after deletion"
    );

    // remove test1
    cache.remove(&String::from("test1"));

    assert_eq!(
        cache.get(&String::from("test1")),
        None,
        "Test1 value found after deletion"
    );
    assert_eq!(
        cache.get(&String::from("test2")),
        None,
        "Test2 value found after deletion"
    );
    assert_eq!(
        cache.get(&String::from("test3")),
        None,
        "Test3 value found after deletion"
    );
}

#[test]
fn insert_get_interspersed_max_size_eviction() {
    let mut cache: TLRUCache<String, String> = TLRUCache::new(Some(2), None, None);

    // insert test1
    cache.insert(String::from("test1"), String::from("test1test1"));

    assert_eq!(
        cache.get(&String::from("test1")),
        Some(Rc::new(String::from("test1test1"))),
        "Test1 value not found after insertion of Test1"
    );

    // insert test2
    cache.insert(String::from("test2"), String::from("test2test2"));

    assert_eq!(
        cache.get(&String::from("test1")),
        Some(Rc::new(String::from("test1test1"))),
        "Test1 value not found after insertion of Test2"
    );
    assert_eq!(
        cache.get(&String::from("test2")),
        Some(Rc::new(String::from("test2test2"))),
        "Test2 value not found after insertion of Test2"
    );

    // insert test3 (should remove least recently used: test1)
    cache.insert(String::from("test3"), String::from("test3test3"));

    assert_eq!(
        cache.get(&String::from("test3")),
        Some(Rc::new(String::from("test3test3"))),
        "Test3 value not found after insertion of Test3"
    );
    assert_eq!(
        cache.get(&String::from("test2")),
        Some(Rc::new(String::from("test2test2"))),
        "Test2 value not found after insertion of Test3"
    );
    assert_eq!(
        cache.get(&String::from("test1")),
        None,
        "Test1 found after insertion of Test3"
    );

    // insert test4 (should remove least recently used: test3, note the change in order of checking in previous test)
    cache.insert(String::from("test4"), String::from("test4test4"));

    assert_eq!(
        cache.get(&String::from("test1")),
        None,
        "Test1 found after insertion of Test4"
    );
    assert_eq!(
        cache.get(&String::from("test2")),
        Some(Rc::new(String::from("test2test2"))),
        "Test2 value not found after insertion of Test4"
    );
    assert_eq!(
        cache.get(&String::from("test3")),
        None,
        "Test3 found after insertion of Test4"
    );
    assert_eq!(
        cache.get(&String::from("test4")),
        Some(Rc::new(String::from("test4test4"))),
        "Test4 value not found after insertion of Test4"
    );
}
