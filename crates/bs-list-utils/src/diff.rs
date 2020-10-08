use crate::HasItemKey;
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;

#[derive(Debug)]
pub struct Diff<'a, Left, Right> {
  pub left: Vec<&'a Left>,
  pub both: Vec<(&'a Left, &'a Right)>,
  pub right: Vec<&'a Right>,
  /// Ignored due to duplicate keys
  pub ignored: Vec<DiffIgnored<&'a Left, &'a Right>>,
}

#[derive(Debug)]
pub enum DiffIgnored<Left, Right> {
  Left(Left),
  Right(Right),
}

impl<Left, Right> PartialEq for DiffIgnored<Left, Right>
where
  Left: PartialEq,
  Right: PartialEq,
{
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (DiffIgnored::Left(ref l), DiffIgnored::Left(ref r)) => l == r,
      (DiffIgnored::Right(ref l), DiffIgnored::Right(ref r)) => l == r,
      _ => false,
    }
  }
}

pub fn diff<'a, 'k, K, Left, Right>(left: &'a [Left], right: &'a [Right]) -> Diff<'a, Left, Right>
where
  'a: 'k,
  K: Ord,
  Left: HasItemKey<'k, K>,
  Right: HasItemKey<'k, K>,
{
  let mut ignored = vec![];
  let mut left_map: BTreeMap<K, &Left> = BTreeMap::new();
  let mut right_map: BTreeMap<K, &Right> = BTreeMap::new();

  for item in left {
    if let Some(replaced) = left_map.insert(item.get_item_key(), item) {
      ignored.push(DiffIgnored::Left(replaced));
    }
  }

  for item in right {
    if let Some(replaced) = right_map.insert(item.get_item_key(), item) {
      ignored.push(DiffIgnored::Right(replaced));
    }
  }

  let mut diff = Diff {
    left: vec![],
    both: vec![],
    right: vec![],
    ignored,
  };

  for (k, v) in &left_map {
    match right_map.remove(k) {
      None => {
        diff.left.push(*v);
      }
      Some(right) => diff.both.push((*v, right)),
    }
  }

  if !right_map.is_empty() {
    diff.right = right_map.into_iter().map(|(_, v)| v).collect();
  }

  diff
}

pub fn with_key<'a, T, F, K>(list: &'a [T], f: F) -> Vec<WithKey<&'a T, K>>
where
  F: Fn(&'a T) -> K,
  K: 'a,
{
  list
    .into_iter()
    .map(|item| WithKey { key: f(item), item })
    .collect()
}

/// Wrapper type for foreign types that cannot impl `HasItemKey`
pub struct WithKey<T, K> {
  pub key: K,
  pub item: T,
}

impl<T, K> Deref for WithKey<T, K> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.item
  }
}

impl<T, K> Debug for WithKey<T, K>
where
  T: Debug,
  K: Debug,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("WithKey")
      .field("key", &self.key)
      .field("item", &self.item)
      .finish()
  }
}

impl<'s, T, K> HasItemKey<'s, &'s K> for WithKey<T, K> {
  fn get_item_key(&'s self) -> &'s K {
    &self.key
  }
}

#[test]
fn test_diff() {
  #[derive(Debug, PartialEq)]
  struct V1(String);
  impl<'s> HasItemKey<'s, &'s str> for V1 {
    fn get_item_key(&'s self) -> &'s str {
      self.0.as_ref()
    }
  }

  #[derive(Debug, PartialEq)]
  struct V2(&'static str);
  impl<'s> HasItemKey<'s, &'s str> for V2 {
    fn get_item_key(&'s self) -> &'s str {
      self.0
    }
  }

  let l1: Vec<V1> = ["0", "a", "a", "a", "b", "b", "c"]
    .iter()
    .map(|v| V1(v.to_string()))
    .collect();
  let l2: Vec<V2> = ["a", "a", "a", "b", "b", "c", "d"]
    .iter()
    .map(|v| V2(v))
    .collect();

  let res = diff(&l1, &l2);

  assert_eq!(res.left, vec![&V1("0".to_string())]);
  assert_eq!(
    res.both,
    vec![
      (&V1("a".to_string()), &V2("a")),
      (&V1("b".to_string()), &V2("b")),
      (&V1("c".to_string()), &V2("c"))
    ]
  );
  assert_eq!(res.right, vec![&V2("d")]);
  assert_eq!(
    res.ignored,
    vec![
      DiffIgnored::Left(&V1("a".to_string())),
      DiffIgnored::Left(&V1("a".to_string())),
      DiffIgnored::Left(&V1("b".to_string())),
      DiffIgnored::Right(&V2("a")),
      DiffIgnored::Right(&V2("a")),
      DiffIgnored::Right(&V2("b")),
    ]
  );
}

#[test]
fn test_with_key() {
  let l1: Vec<String> = [1, 2, 3, 4].iter().map(|v| v.to_string()).collect();
  let l2: Vec<String> = [3, 4, 5, 6].iter().map(|v| v.to_string()).collect();

  let w1 = with_key(&l1, |v| v.as_str());
  let w2 = with_key(&l2, |v| v.as_str());

  let res = diff(&w1, &w2);

  assert_eq!(
    res
      .left
      .into_iter()
      .map(|item| item.item.clone())
      .collect::<Vec<_>>(),
    vec!["1".to_string(), "2".to_string(),]
  );

  assert_eq!(
    res
      .both
      .into_iter()
      .map(|(l, r)| (l.item.clone(), r.item.clone()))
      .collect::<Vec<_>>(),
    vec![
      ("3".to_string(), "3".to_string()),
      ("4".to_string(), "4".to_string()),
    ]
  );

  assert_eq!(
    res
      .right
      .into_iter()
      .map(|item| item.item.clone())
      .collect::<Vec<_>>(),
    vec!["5".to_string(), "6".to_string(),]
  );
  assert!(res.ignored.is_empty());
}
