use std::collections::HashMap;
use std::rc::Rc;
use std::hash::Hash;
use crate::spec::Extension;

#[derive(Debug)]
pub struct ExtensionCache<K: Hash + Eq> {
    mymap: HashMap<K, Rc<dyn Extension>>
}

impl<K: Hash + Eq> ExtensionCache<K> {
    pub fn new() -> Self {
        ExtensionCache {
            mymap: HashMap::new()
        }
    }

    pub fn get_or_create(
        &mut self,
        token: K,
        create: impl FnOnce() -> Rc<dyn Extension>
    ) -> Rc<dyn Extension>
    {
        Rc::clone(self.mymap.entry(token).or_insert_with(|| create()))
    }
}
