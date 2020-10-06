use std::collections::{BTreeMap, btree_map::Entry};
use crate::HasItemKey;
use std::ops::Deref;

pub fn get_dups<'k, T, K>(list: &'k [T]) -> BTreeMap<K, usize>
  where
    T: HasItemKey<'k, K>,
    K: Ord
{
  let mut map = BTreeMap::new();

  for item in list {
    let key = item.get_item_key();
    let value = map.entry(key).or_insert(0);
    *value += 1;
  }

  map.into_iter().filter(|(_, v)| {
    *v > 1
  }).collect()
}

#[derive(Debug)]
pub struct ItemSet<T>(Vec<T>);

impl<T> ItemSet<T> {
  pub fn into_inner(self) -> Vec<T> {
    self.0
  }
}

impl<T> Deref for ItemSet<T> {
  type Target = Vec<T>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[derive(Debug)]
pub struct Dedup<T, K> {
  pub set: ItemSet<T>,
  pub removed: BTreeMap<K, Vec<T>>,
}

pub fn dedup<T, K>(list: Vec<T>) -> Dedup<T, K>
where
  K: Ord,
  T: for<'k> HasItemKey<'k, K>
{
  let mut map = BTreeMap::new();

  enum Value<V> {
    Placeholder,
    Single(V),
    Multi(Vec<V>),
  }

  for item in list {
    match map.entry(item.get_item_key()) {
      Entry::Occupied(mut e) => {
        match std::mem::replace(e.get_mut(), Value::Placeholder) {
          Value::Placeholder => unreachable!(),
          Value::Single(value) => {
            *e.get_mut() = Value::Multi(vec![value, item])
          },
          Value::Multi(mut values) => {
            values.push(item);
            *e.get_mut() = Value::Multi(values)
          },
        }
      },
      Entry::Vacant(e) => {
        e.insert(Value::Single(item));
      },
    }
  }

  let mut res = Dedup {
    set: ItemSet(vec![]),
    removed: BTreeMap::new(),
  };

  for (k, v) in map {
    match v {
      Value::Placeholder => unreachable!(),
      Value::Single(value) => {
        res.set.0.push(value);
      }
      Value::Multi(mut values) => {
        res.set.0.push(values.pop().unwrap());
        res.removed.insert(k, values);
      }
    }
  }

  res
}

#[test]
fn test_get_dups() {
  let list = &[1,1,1,1,2,2,2,3,3,4,5,6];
  let map = get_dups(list);
  assert_eq!(map, {
    vec![
      (1, 4),
      (2, 3),
      (3, 2),
    ].into_iter().collect()
  });
}

#[test]
fn test_dedup() {
  let list = &[1,1,1,1,2,2,2,3,3,4,5,6];
  let dedup = dedup(list.to_vec());
  assert_eq!(&dedup.set as &[i32], &[1,2,3,4,5,6]);
  assert_eq!(dedup.removed, {
    vec![
      (1, vec![1, 1, 1]),
      (2, vec![2, 2]),
      (3, vec![3]),
    ].into_iter().collect()
  })
}