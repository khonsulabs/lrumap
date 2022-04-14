var searchIndex = JSON.parse('{\
"lrumap":{"doc":"A set of safe Least-Recently-Used (LRU) cache types aimed …","t":[3,13,3,3,3,8,13,4,11,11,11,11,11,11,11,11,11,11,10,11,11,11,11,11,10,11,11,11,11,11,11,11,11,11,11,11,11,11,10,11,11,11,11,10,11,11,11,11,10,11,11,11,11,11,11,11,11,11,11,11,10,11,11,11,11,10,11,11,11,11,11,11,10,11,11,11,11,11,11,11,10,11,11,11,11,11,11,11,10,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,12,12,12],"n":["EntryRef","Evicted","Iter","LruBTreeMap","LruHashMap","LruMap","PreviousValue","Removed","borrow","borrow","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","entry","entry","entry","entry","entry","eq","extend","extend","extend","extend","extend","fmt","fmt","fmt","fmt","from","from","from","from","from","get","get","get","get","get","get_without_update","get_without_update","get_without_update","get_without_update","get_without_update","head","head","head","into","into","into","into","into","into_iter","into_iter","into_iter","is_empty","iter","iter","iter","iter","key","len","len","len","most_recent_in_range","move_next","move_previous","ne","new","new","new","new","new","next","next_back","peek_value","push","push","push","push","push","remove_moving_next","remove_moving_previous","staleness","tail","tail","tail","take","take_and_move_next","take_and_move_previous","touch","try_from","try_from","try_from","try_from","try_from","try_into","try_into","try_into","try_into","try_into","type_id","type_id","type_id","type_id","type_id","value","with_hasher","0","0","1"],"q":["lrumap","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","lrumap::Removed","",""],"d":["A reference to an entry in a Least Recently Used map.","An entry was evicted to make room for the key that was …","A double-ended iterator over a cache’s keys and values …","A Least Recently Used map with fixed capacity that stores …","A Least Recently Used map with fixed capacity that stores …","A Least Recently Used map interface that supports all map …","The previously stored value for the key that was written …","A removed value or entry.","","","","","","","","","","","Returns an <code>EntryRef</code> for <code>key</code>, if present.","","Returns an <code>EntryRef</code> for <code>key</code>, if present.","","Returns an <code>EntryRef</code> for <code>key</code>, if present.","","Pushes all items from <code>iterator</code> into this map. If there are …","","Pushes all items from <code>iterator</code> into this map. If there are …","Pushes all items from <code>iterator</code> into this map. If there are …","","","","","","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the stored value for <code>key</code>, if present.","Returns the stored value for <code>key</code>, if present.","","Returns the stored value for <code>key</code>, if present.","","Returns the stored value for <code>key</code>, if present.","","Returns the stored value for <code>key</code>, if present.","Returns the stored value for <code>key</code>, if present.","","Returns a reference to the most recently used key.","","","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","","","","Retruns true if this map contains no keys.","Returns an iterator over the keys and values in order from …","","Returns an iterator over the least-recently used keys …","","Returns the key of this entry.","Returns the number of keys present in this map.","","","Returns the most recently touched entry with a key within …","Updates this reference to point to the next least recently …","Updates this reference to point to the next most recently …","","Creates a new map with the maximum <code>capacity</code>.","","Creates a new map with the maximum <code>capacity</code>.","Creates a new map with the maximum <code>capacity</code>.","","","","Returns the value of this entry.","Inserts <code>value</code> for <code>key</code> into this map. If a value is already …","","Inserts <code>value</code> for <code>key</code> into this map. If a value is already …","Inserts <code>value</code> for <code>key</code> into this map. If a value is already …","","Removes the current entry. If this was not the last entry, …","Removes the current entry. If this was not the first …","Returns the number of changes to the cache since this key …","Returns a reference to the least recently used key.","","","Removes and returns the current entry’s key and value.","Removes and returns the current entry’s key and value. …","Removes and returns the current entry’s key and value. …","Touches this key, making it the most recently used key.","","","","","","","","","","","","","","","","Returns the value of this entry.","Creates a new map with the maximum <code>capacity</code> and <code>hasher</code>.","","",""],"i":[0,1,0,0,0,0,1,0,2,3,4,1,5,2,3,4,1,5,6,3,3,5,5,1,6,3,3,5,5,3,4,1,5,2,3,4,1,5,6,3,3,5,5,6,3,3,5,5,6,3,5,2,3,4,1,5,2,3,5,6,6,3,4,5,4,6,3,5,5,4,4,1,6,3,3,5,5,2,2,4,6,3,3,5,5,4,4,4,6,3,5,4,4,4,4,2,3,4,1,5,2,3,4,1,5,2,3,4,1,5,4,3,7,8,8],"f":[null,null,null,null,null,null,null,null,[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0]],["",0]],[[["",0],["",0]],["option",4,[["entryref",3]]]],[[["",0],["",0]],["option",4,[["entryref",3]]]],[[["",0],["",0]],["option",4,[["entryref",3]]]],[[["",0],["",0]],["option",4,[["entryref",3]]]],[[["",0],["",0]],["option",4,[["entryref",3]]]],[[["",0],["removed",4]],["bool",0]],[[["",0],["intoiterator",8]]],[[["",0],["intoiterator",8]]],[[["",0],["intoiterator",8]]],[[["",0],["intoiterator",8]]],[[["",0],["intoiterator",8]]],[[["",0],["formatter",3]],["result",6]],[[["",0],["formatter",3]],["result",6]],[[["",0],["formatter",3]],["result",6]],[[["",0],["formatter",3]],["result",6]],[[]],[[]],[[]],[[]],[[]],[[["",0],["",0]],["option",4]],[[["",0],["",0]],["option",4]],[[["",0],["",0]],["option",4]],[[["",0],["",0]],["option",4]],[[["",0],["",0]],["option",4]],[[["",0],["",0]],["option",4]],[[["",0],["",0]],["option",4]],[[["",0],["",0]],["option",4]],[[["",0],["",0]],["option",4]],[[["",0],["",0]],["option",4]],[[["",0]],["option",4,[["entryref",3]]]],[[["",0]],["option",4,[["entryref",3]]]],[[["",0]],["option",4,[["entryref",3]]]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[["",0]],["bool",0]],[[["",0]],["iter",3]],[[["",0]],["iter",3]],[[["",0]],["iter",3]],[[["",0]],["iter",3]],[[["",0]],["",0]],[[["",0]],["usize",0]],[[["",0]],["usize",0]],[[["",0]],["usize",0]],[[["",0]],["option",4,[["entryref",3]]]],[[["",0]],["bool",0]],[[["",0]],["bool",0]],[[["",0],["removed",4]],["bool",0]],[[["usize",0]]],[[["usize",0]]],[[["usize",0]]],[[["usize",0]]],[[["usize",0]]],[[["",0]],["option",4]],[[["",0]],["option",4]],[[["",0]],["",0]],[[["",0]],["option",4,[["removed",4]]]],[[["",0]],["option",4,[["removed",4]]]],[[["",0]],["option",4,[["removed",4]]]],[[["",0]],["option",4,[["removed",4]]]],[[["",0]],["option",4,[["removed",4]]]],[[],["option",4]],[[],["option",4]],[[["",0]],["usize",0]],[[["",0]],["option",4,[["entryref",3]]]],[[["",0]],["option",4,[["entryref",3]]]],[[["",0]],["option",4,[["entryref",3]]]],[[]],[[]],[[]],[[["",0]]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]],[[["",0]],["typeid",3]],[[["",0]],["",0]],[[["usize",0]]],null,null,null],"p":[[4,"Removed"],[3,"Iter"],[3,"LruHashMap"],[3,"EntryRef"],[3,"LruBTreeMap"],[8,"LruMap"],[13,"PreviousValue"],[13,"Evicted"]]}\
}');
if (window.initSearch) {window.initSearch(searchIndex)};