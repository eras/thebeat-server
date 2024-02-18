use std::collections::{BTreeMap, HashMap};

struct ExpiringData<Value> {
    value: Value,
    insert_count: u64,
    insert_time: chrono::DateTime<chrono::Utc>,
}

pub(crate) struct Expiring<Key, Value>
where
    Key: Clone,
    Value: Clone,
{
    value: HashMap<Key, ExpiringData<Value>>,
    by_insert: BTreeMap<u64, Key>,
    insert_count: u64,
}

impl<Key, Value> Expiring<Key, Value>
where
    Key: Clone + std::hash::Hash + PartialEq + Eq,
    Value: Clone,
{
    pub fn new() -> Self {
        Expiring {
            value: HashMap::new(),
            by_insert: BTreeMap::new(),
            insert_count: 0u64,
        }
    }

    pub fn put(&mut self, key: Key, value: Value) {
        self.remove(&key.clone());
        self.value.insert(
            key.clone(),
            ExpiringData {
                value: value,
                insert_count: self.insert_count,
                insert_time: chrono::Utc::now(),
            },
        );
        self.by_insert.insert(self.insert_count, key);
        self.insert_count += 1
    }

    pub fn remove(&mut self, key: &Key) {
        match self.value.get(key) {
            None => (),
            Some(value) => {
                self.by_insert.remove(&value.insert_count);
                self.value.remove(key);
            }
        }
    }

    pub fn purge_old_entries(&mut self) {
        let deadline = chrono::Utc::now() - chrono::Duration::seconds(10);
        loop {
            match self.by_insert.first_key_value() {
                None => break,
                Some((_, key)) if self.value.get(key).unwrap().insert_time < deadline => {
                    self.remove(&key.clone());
                }
                Some(_) => break,
            }
        }
    }

    pub fn all(&self) -> HashMap<Key, Value> {
        self.value
            .iter()
            .map(|(k, v)| (k.clone(), v.value.clone()))
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}
