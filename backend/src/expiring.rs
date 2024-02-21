use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Debug)]
struct ExpiringData<Value> {
    value: Value,
    insert_count: u64,
    insert_time: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug)]
pub(crate) struct Expiring<Key, Value>
where
    Key: Clone + std::fmt::Debug,
    Value: Clone + std::fmt::Debug,
{
    values: HashMap<Key, ExpiringData<Value>>,
    by_insert: BTreeMap<u64, Key>,
    insert_count: u64,
    expire_duration: chrono::Duration,
}

impl<Key, Value> Expiring<Key, Value>
where
    Key: Clone + std::hash::Hash + PartialEq + Eq + std::fmt::Debug,
    Value: Clone + std::fmt::Debug,
{
    pub fn new(expire_duration: chrono::Duration) -> Self {
        Expiring {
            values: HashMap::new(),
            by_insert: BTreeMap::new(),
            insert_count: 0u64,
            expire_duration,
        }
    }

    pub fn get(&self, key: &Key) -> Option<&Value> {
        self.values.get(key).map(|v| &v.value)
    }

    pub fn get_mut(&mut self, key: &Key) -> Option<&mut Value> {
        self.values.get_mut(key).map(|v| &mut v.value)
    }

    // Returns true if a new value was inserted (with the put function)
    pub fn get_or_put_mut<F>(&mut self, key: Key, put: F) -> (bool, &mut Value)
    where
        F: FnOnce() -> Value,
    {
        let entry = self.values.entry(key.clone());
        let was_vacant = matches!(entry, std::collections::hash_map::Entry::Vacant(_));
        let value = entry.or_insert_with(|| ExpiringData {
            value: put(),
            insert_count: self.insert_count,
            insert_time: chrono::Utc::now(),
        });

        if was_vacant {
            self.by_insert.insert(self.insert_count, key.clone());
            self.insert_count += 1;
        }

        (was_vacant, &mut value.value)
    }

    pub fn put(&mut self, key: Key, value: Value) {
        self.remove(&key);
        self.values.insert(
            key.clone(),
            ExpiringData {
                value,
                insert_count: self.insert_count,
                insert_time: chrono::Utc::now(),
            },
        );
        self.by_insert.insert(self.insert_count, key);
        self.insert_count += 1
    }

    pub fn refresh(&mut self, key: Key) -> bool {
        match self.values.get_mut(&key) {
            None => false,
            Some(ref mut v) => {
                self.by_insert.remove(&v.insert_count);
                v.insert_count = self.insert_count;
                v.insert_time = chrono::Utc::now();
                self.by_insert.insert(self.insert_count, key);
                self.insert_count += 1;
                true
            }
        }
    }

    pub fn remove(&mut self, key: &Key) {
        match self.values.get(key) {
            None => (),
            Some(value) => {
                self.by_insert.remove(&value.insert_count);
                self.values.remove(key);
            }
        }
    }

    pub fn purge_old_entries(&mut self) {
        let deadline = chrono::Utc::now() - self.expire_duration;
        loop {
            match self.by_insert.first_key_value() {
                None => break,
                Some((_, key)) if self.values.get(key).unwrap().insert_time < deadline => {
                    self.remove(&key.clone());
                }
                Some(_) => break,
            }
        }
    }

    pub fn all(&self) -> HashMap<Key, Value> {
        self.values
            .iter()
            .map(|(k, v)| (k.clone(), v.value.clone()))
            .collect()
    }

    pub fn all_ref(&self) -> HashMap<&Key, &Value> {
        self.values.iter().map(|(k, v)| (k, &v.value)).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}
