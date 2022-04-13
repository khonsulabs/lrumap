use std::fmt::Debug;

use crate::{LruBTreeMap, LruHashMap, LruMap, Removed};

fn basic_tests<Map>()
where
    Map: LruMap<u32, u32> + Debug,
{
    let mut lru = Map::new(2);
    assert_eq!(lru.push(1, 1), None);
    println!("1: {lru:#?}");
    assert_eq!(lru.push(2, 2), None);
    println!("2: {lru:#?}");
    // Pushing a new value will expire the first push.
    assert_eq!(lru.push(3, 3), Some(Removed::Evicted(1, 1)));
    println!("3: {lru:#?}");
    // Replacing 2 will return the existing value.
    assert_eq!(lru.push(2, 22), Some(Removed::PreviousValue(2)));
    println!("4: {lru:#?}");
    // Replacing the value should have made 2 the most recent entry, meaning a
    // push will remove 3.
    assert_eq!(lru.push(4, 4), Some(Removed::Evicted(3, 3)));
    println!("5: {lru:#?}");
    // Getting an entry should update its access
    assert_eq!(lru.get(&2), Some(&22));
    // But not using get_without_update
    assert_eq!(lru.get_without_update(&4), Some(&4));
    println!("6: {lru:#?}");
    assert_eq!(lru.push(5, 5), Some(Removed::Evicted(4, 4)));
}

#[test]
fn hash_basics() {
    basic_tests::<LruHashMap<_, _>>();
}

#[test]
fn btree_basics() {
    basic_tests::<LruBTreeMap<_, _>>();
}

#[allow(clippy::cognitive_complexity)]
fn iteration_tests<Map>()
where
    Map: LruMap<u32, u32> + Debug,
{
    let mut lru = Map::new(3);
    assert!(lru.head().is_none());
    lru.push(1, 1);
    {
        let mut entry = lru.head().unwrap();
        assert_eq!(entry.key(), &1);
        assert!(!entry.move_next());
        assert_eq!(entry.key(), &1);
        assert!(!entry.move_previous());
        assert_eq!(entry.key(), &1);
    }
    lru.push(2, 2);
    {
        let mut entry = lru.head().unwrap();
        assert_eq!(entry.key(), &2);
        assert_eq!(entry.peek_value(), &2);
        assert!(entry.move_next());
        assert_eq!(entry.key(), &1);
        assert_eq!(entry.peek_value(), &1);
        assert!(!entry.move_next());
        assert!(entry.move_previous());
        assert_eq!(entry.key(), &2);
        assert_eq!(entry.peek_value(), &2);
        assert!(!entry.move_previous());
        assert_eq!(entry.key(), &2);
    }
    lru.push(3, 3);
    {
        // Test mutating and iterating.
        let mut entry = lru.tail().unwrap();
        assert_eq!(entry.key(), &1);
        // By accessing the value, this should now become the head.
        assert_eq!(entry.value(), &1);
        assert!(!entry.move_previous());
        // Iterate through the remaining entries.
        assert!(entry.move_next());
        assert_eq!(entry.key(), &3);
        assert_eq!(entry.peek_value(), &3);
        assert!(entry.move_next());
        assert_eq!(entry.key(), &2);
        assert_eq!(entry.peek_value(), &2);
        assert!(!entry.move_next());
    }
}

#[test]
fn hash_iteration() {
    iteration_tests::<LruHashMap<_, _>>();
}

#[test]
fn btree_iteration() {
    iteration_tests::<LruBTreeMap<_, _>>();
}

fn entry_removal_tests<Map>()
where
    Map: LruMap<u32, u32> + Debug,
{
    let mut lru = Map::new(3);
    lru.push(1, 1);
    lru.push(2, 2);
    lru.push(3, 3);
    let entry = lru.head().unwrap();
    // Remove 3, no previous, should return None.
    assert!(entry.remove_moving_previous().is_none());
    assert!(lru.get(&3).is_none());
    let entry = lru.tail().unwrap();
    // Remove 1, no next, should return None.
    assert!(entry.remove_moving_next().is_none());
    assert!(lru.get(&1).is_none());
    let (key, _value) = lru.head().unwrap().take();
    assert!(lru.get(&2).is_none());
    assert_eq!(key, 2);
    assert!(lru.head().is_none());
    assert!(lru.tail().is_none());

    // Start fresh and test deleting the other directions
    lru.push(1, 1);
    lru.push(2, 2);
    lru.push(3, 3);
    // Remove 3, moving next, should end up on 2
    let mut entry = lru.head().unwrap();
    entry = entry.remove_moving_next().unwrap();
    assert_eq!(entry.key(), &2);
    // Remove 2, moving previous, should end up on 2
    let mut entry = lru.tail().unwrap();
    entry = entry.remove_moving_previous().unwrap();
    let (key, _value) = entry.take();
    assert_eq!(key, 2);
    assert!(lru.head().is_none());
    assert!(lru.tail().is_none());
}

#[test]
fn hash_entry_removal() {
    entry_removal_tests::<LruHashMap<_, _>>();
}

#[test]
fn btree_entry_removal() {
    entry_removal_tests::<LruBTreeMap<_, _>>();
}
