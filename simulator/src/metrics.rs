use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    pub(crate) static METRICS: RefCell<HashMap<String, Box<dyn Any>>> = RefCell::new(HashMap::new());
}

pub fn Set<T: 'static>(key: &str, value: T) {
    METRICS.with(|m| {
        m.borrow_mut().insert(key.to_string(), Box::new(value));
    });
}

pub fn Get<T: 'static + Clone>(key: &str) -> Option<T> {
    METRICS.with(|m| {
        m.borrow()
            .get(key)
            .and_then(|v| v.downcast_ref::<T>())
            .cloned()
    })
}

pub fn Modify<T: 'static>(key: &str, f: impl FnOnce(&mut T)) {
    METRICS.with(|m| {
        if let Some(value) = m.borrow_mut().get_mut(key) {
            if let Some(typed_value) = value.downcast_mut::<T>() {
                f(typed_value);
            }
        }
    });
}

pub fn Clear() {
    METRICS.with(|m| {
        m.borrow_mut().clear();
    });
}
