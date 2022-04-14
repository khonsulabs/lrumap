use std::fmt::Debug;

use crate::{LruBTreeMap, LruHashMap, LruMap, Removed};

fn basic_tests<Map>()
where
    Map: LruMap<u32, u32> + Debug,
{
    let mut lru = Map::new(2);
    assert!(lru.is_empty());
    assert_eq!(lru.push(1, 1), None);
    assert_eq!(lru.len(), 1);
    assert_eq!(lru.push(2, 2), None);
    assert_eq!(lru.len(), 2);
    // Pushing a new value will expire the first push.
    assert_eq!(lru.push(3, 3), Some(Removed::Evicted(1, 1)));
    assert_eq!(lru.len(), 2);
    // Replacing 2 will return the existing value.
    assert_eq!(lru.push(2, 22), Some(Removed::PreviousValue(2)));
    // Replacing the value should have made 2 the most recent entry, meaning a
    // push will remove 3.
    assert_eq!(lru.push(4, 4), Some(Removed::Evicted(3, 3)));
    // Getting an entry should update its access
    assert_eq!(lru.get(&2), Some(&22));
    // But not using get_without_update
    assert_eq!(lru.get_without_update(&4), Some(&4));
    // Key 2 is still the front, and shouldn't be stale.
    assert_eq!(lru.entry(&2).unwrap().staleness(), 0);
    // Key 4 is the second, and there has been one modification since the entry
    // was last touched.
    assert_eq!(lru.entry(&4).unwrap().staleness(), 1);
    assert_eq!(lru.push(5, 5), Some(Removed::Evicted(4, 4)));
    // This will call move_node_to_front with the short-circuit evaluating true
    // at the start of the function.
    assert_eq!(lru.get(&5), Some(&5));
    assert_eq!(lru.head().unwrap().key(), &5);
    println!("Final State: {:?}", lru);
}

#[test]
fn hash_basics() {
    basic_tests::<LruHashMap<_, _>>();
}

#[test]
fn btree_basics() {
    basic_tests::<LruBTreeMap<_, _>>();
}
fn larger_tests<Map>()
where
    Map: LruMap<u32, u32> + Debug,
{
    // The final re-ordering edge case only arises with at least 3 entries. With
    // only 2 entries, either entry is either the head or the tail.
    let mut lru = Map::new(5);
    lru.extend([(1, 1), (2, 2), (3, 3), (4, 4), (5, 5)]);
    // Test the second to last moving to the front => 2, 5, 4, 3, 1
    assert_eq!(lru.get(&2), Some(&2));
    assert_eq!(
        lru.iter().map(|(_key, value)| *value).collect::<Vec<_>>(),
        vec![2, 5, 4, 3, 1]
    );
    // Test moving the middle entry => 4, 2, 5, 3, 1
    assert_eq!(lru.get(&4), Some(&4));
    assert_eq!(
        lru.iter().map(|(_key, value)| *value).collect::<Vec<_>>(),
        vec![4, 2, 5, 3, 1]
    );
    // Test moving the second entry => 2, 4, 5, 3, 1
    assert_eq!(lru.get(&2), Some(&2));
    // Test the staleness (number of changes since last touch). 7 total
    // operations.
    assert_eq!(lru.entry(&2).unwrap().staleness(), 0); // touched on op 5 and 7
    assert_eq!(lru.entry(&4).unwrap().staleness(), 1); // touched on op 6
    assert_eq!(lru.entry(&5).unwrap().staleness(), 3); // touched on op 4
    assert_eq!(lru.entry(&3).unwrap().staleness(), 5); // Never touched
    assert_eq!(lru.entry(&1).unwrap().staleness(), 7); // Never touched

    // Verify the order, but use into_iter() this time.
    assert_eq!(
        lru.into_iter()
            .map(|(_key, value)| value)
            .collect::<Vec<_>>(),
        vec![2, 4, 5, 3, 1]
    );
}

#[test]
fn hash_larger() {
    larger_tests::<LruHashMap<_, _>>();
}

#[test]
fn btree_larger() {
    larger_tests::<LruBTreeMap<_, _>>();
}

#[allow(clippy::cognitive_complexity)]
fn enumeration_tests<Map>()
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
fn hash_enumeration() {
    enumeration_tests::<LruHashMap<_, _>>();
}

#[test]
fn btree_enumeration() {
    enumeration_tests::<LruBTreeMap<_, _>>();
}

#[allow(clippy::cognitive_complexity)]
fn iteration_tests<Map>()
where
    Map: LruMap<u32, u32> + Debug,
{
    let mut lru = Map::new(5);
    lru.extend([(1, 1), (2, 2), (3, 3), (4, 4), (5, 5)]);
    assert_eq!(
        lru.iter().collect::<Vec<_>>(),
        &[(&5, &5), (&4, &4), (&3, &3), (&2, &2), (&1, &1)]
    );

    // Test double-ended iteration
    let mut iter = lru.iter();
    assert!(iter.next_back().is_none());
    for i in (1..=5).rev() {
        assert_eq!(iter.next().unwrap().0, &i);
    }
    assert!(iter.next().is_none());
    // We're now past the end of the tail, we should be able to recover and get
    // back to the head.
    for i in 1..=5 {
        assert_eq!(iter.next_back().unwrap().0, &i);
    }
    assert!(iter.next_back().is_none());

    // Test partial iteration
    assert_eq!(
        lru.entry(&3).unwrap().iter().collect::<Vec<_>>(),
        &[(&3, &3), (&2, &2), (&1, &1)]
    );
    // Moving back should return the previous entry from the starting point.
    assert_eq!(lru.entry(&3).unwrap().iter().next_back().unwrap().0, &4);
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
    assert_eq!(lru.len(), 2);
    assert!(lru.get(&3).is_none());
    let entry = lru.tail().unwrap();
    // Remove 1, no next, should return None.
    assert!(entry.remove_moving_next().is_none());
    assert_eq!(lru.len(), 1);
    assert!(lru.get(&1).is_none());
    let (key, _value) = lru.head().unwrap().take();
    assert!(lru.is_empty());
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
