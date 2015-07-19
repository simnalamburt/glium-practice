use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::atomic::{ AtomicUsize, Ordering };

pub type Id = usize;

pub trait Item<Param> {
    fn new(id: &Id, param: &Param) -> Self;
}

pub struct Manager<I, P> where I: Item<P> + Clone {
    next_id: AtomicUsize,
    items: HashMap<Id, I>,
    _param_type: PhantomData<P>,
}

impl<I, P> Manager<I, P> where I: Item<P> + Clone {
    pub fn new() -> Self {
        Manager {
            next_id: AtomicUsize::new(0),
            items: HashMap::new(),
            _param_type: PhantomData,
        }
    }

    pub fn create(&mut self, param: &P) -> I {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let item = I::new(&id, param);
        debug_assert!(!self.items.contains_key(&id));
        self.items.insert(id, item.clone());
        item
    }
}

#[cfg(test)]
#[derive(Clone)]
pub struct TestItem {
    pub id: Id,
    pub a: i32,
    pub b: bool,
}

#[cfg(test)]
impl Item<(i32, bool)> for TestItem {
    fn new(id: &Id, param: &(i32, bool)) -> Self {
        TestItem {
            id: *id,
            a: param.0,
            b: param.1,
        }
    }
}

#[test]
fn manager_create_item_starts_id_with_zero() {
    type TestManager = Manager<TestItem, (i32, bool)>;
    let mut manager = TestManager::new();

    let item = manager.create(&(1, true));

    assert_eq!(item.id, 0);
    assert_eq!(item.a, 1);
    assert_eq!(item.b, true);
}