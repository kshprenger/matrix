use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    pub(crate) static ANY_KV: RefCell<HashMap<String, Box<dyn Any>>> = RefCell::new(HashMap::new());
}

pub fn Set<T: 'static>(key: &str, value: T) {
    ANY_KV.with(|m| {
        m.borrow_mut().insert(key.to_string(), Box::new(value));
    });
}

pub fn Get<T: 'static + Clone>(key: &str) -> T {
    ANY_KV.with(|m| {
        m.borrow()
            .get(key)
            .expect("No key")
            .downcast_ref::<T>()
            .cloned()
            .expect("Wrong type cast")
    })
}

pub fn Modify<T: 'static>(key: &str, f: impl FnOnce(&mut T)) {
    ANY_KV.with(|m| {
        f(m.borrow_mut()
            .get_mut(key)
            .expect("No key")
            .downcast_mut::<T>()
            .expect("Wrong type cast"));
    });
}

pub fn Drop() {
    ANY_KV.take();
}
