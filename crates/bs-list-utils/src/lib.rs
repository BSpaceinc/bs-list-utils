pub mod diff;
pub mod dup;

pub trait HasItemKey<'s, K> {
  fn get_item_key(&'s self) -> K;
}

impl<'s, T, K> HasItemKey<'s, K> for (usize, T)
where
  T: HasItemKey<'s, K>,
{
  fn get_item_key(&'s self) -> K {
    self.1.get_item_key()
  }
}

impl<'s, T, K> HasItemKey<'s, K> for &'s T
where
  T: HasItemKey<'s, K>,
{
  fn get_item_key(&'s self) -> K {
    (self as &T).get_item_key()
  }
}

impl<'s> HasItemKey<'s, i32> for i32 {
  fn get_item_key(&'s self) -> i32 {
    *self
  }
}

impl<'s> HasItemKey<'s, &'s str> for String {
  fn get_item_key(&'s self) -> &'s str {
    self
  }
}

#[macro_export]
macro_rules! impl_has_item_key {
  (| $s:ident : & $t:ty | -> $k:ty { $expr:expr }) => {
    impl $crate::list::HasItemKey<$k> for $t {
      fn get_item_key(&self) -> $k {
        let $s = self;
        $expr
      }
    }
  };
}
