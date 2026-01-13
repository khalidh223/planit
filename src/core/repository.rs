use std::cmp::Ordering;
use std::collections::HashMap;

use crate::core::models::BaseEntity;
use crate::errors::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sort {
    Unordered,
    IdAsc,
    IdDesc,
}

#[derive(Debug)]
struct Staged<T: BaseEntity> {
    pending: Vec<T>,
    next_id_start: i32,
    cleared: bool,
}

#[derive(Debug)]
pub struct PreparedRepo<T: BaseEntity> {
    pub items: HashMap<i32, T>,
    pub next_id: i32,
}

trait QueryableStore<T: BaseEntity> {
    fn items(&self) -> &HashMap<i32, T>;
}

struct FilterSorter<'a, T: BaseEntity> {
    filters: Vec<Box<dyn Fn(&T) -> bool + 'a>>,
    sort: Sort,
    cmp: Option<Box<dyn Fn(&T, &T) -> Ordering + 'a>>,
}

impl<'a, T: BaseEntity> FilterSorter<'a, T> {
    fn new() -> Self {
        Self {
            filters: Vec::new(),
            sort: Sort::Unordered,
            cmp: None,
        }
    }

    fn push_filter(mut self, pred: impl Fn(&T) -> bool + 'a) -> Self {
        self.filters.push(Box::new(pred));
        self
    }

    fn with_sort(mut self, sort: Sort) -> Self {
        self.sort = sort;
        self
    }

    fn with_cmp(mut self, cmp: impl Fn(&T, &T) -> Ordering + 'a) -> Self {
        self.cmp = Some(Box::new(cmp));
        self
    }

    fn sorted_ids(&self, items: &HashMap<i32, T>) -> Vec<i32> {
        let mut ids: Vec<i32> = items
            .iter()
            .filter(|(_, e)| self.filters.iter().all(|f| f(e)))
            .map(|(id, _)| *id)
            .collect();

        if let Some(cmp) = &self.cmp {
            ids.sort_by(|a, b| {
                let ea = items.get(a).expect("id missing");
                let eb = items.get(b).expect("id missing");
                cmp(ea, eb)
            });
        } else {
            match self.sort {
                Sort::Unordered => {}
                Sort::IdAsc => ids.sort(),
                Sort::IdDesc => ids.sort_by(|a, b| b.cmp(a)),
            }
        }
        ids
    }
}

#[derive(Debug)]
pub struct Repository<T: BaseEntity> {
    items: HashMap<i32, T>,
    next_id: i32,
    staged: Option<Staged<T>>,
}

impl<T: BaseEntity> Default for Repository<T> {
    fn default() -> Self {
        Self {
            items: HashMap::new(),
            next_id: 1,
            staged: None,
        }
    }
}

impl<T: BaseEntity> QueryableStore<T> for Repository<T> {
    fn items(&self) -> &HashMap<i32, T> {
        &self.items
    }
}

impl<T: BaseEntity> Repository<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn peek_next_id(&self) -> i32 {
        self.next_id
    }

    pub fn restore_next_id(&mut self, next_id: i32) {
        self.next_id = next_id;
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn insert(&mut self, mut entity: T) -> &T {
        if let Some(staged) = &mut self.staged {
            let id = self.next_id;
            self.next_id += 1;
            entity.set_id(id);
            staged.pending.push(entity);
            staged
                .pending
                .last()
                .expect("staged entity missing after push")
        } else {
            let id = self.next_id;
            self.next_id += 1;
            entity.set_id(id);
            self.items.insert(id, entity);
            self.items.get(&id).expect("inserted entity missing")
        }
    }

    pub fn insert_with_id(&mut self, entity: T) -> Result<()> {
        let id = entity.id();
        if id <= 0 {
            return Err(Error::Parse("ID must be positive.".into()));
        }

        if let Some(staged) = &mut self.staged {
            if staged.pending.iter().any(|e| e.id() == id) || self.items.contains_key(&id) {
                return Err(Error::Parse(format!(
                    "Entity with id {} already exists.",
                    id
                )));
            }
            self.next_id = self.next_id.max(id + 1);
            staged.pending.push(entity);
            return Ok(());
        }

        if self.items.contains_key(&id) {
            return Err(Error::Parse(format!(
                "Entity with id {} already exists.",
                id
            )));
        }
        self.next_id = self.next_id.max(id + 1);
        self.items.insert(id, entity);
        Ok(())
    }

    pub fn get(&self, id: i32) -> Result<&T> {
        self.items
            .get(&id)
            .ok_or_else(|| Error::Parse(format!("Entity with id {} not found.", id)))
    }

    pub fn get_mut(&mut self, id: i32) -> Result<&mut T> {
        self.items
            .get_mut(&id)
            .ok_or_else(|| Error::Parse(format!("Entity with id {} not found.", id)))
    }

    pub fn delete(&mut self, id: i32) -> Result<T> {
        self.items
            .remove(&id)
            .ok_or_else(|| Error::Parse(format!("Entity with id {} not found.", id)))
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.next_id = 1;
        self.staged = None;
    }

    pub fn exists_including_staged(&self, id: i32) -> bool {
        if let Some(staged) = &self.staged {
            if staged.cleared {
                return staged.pending.iter().any(|e| e.id() == id);
            }
            self.items.contains_key(&id) || staged.pending.iter().any(|e| e.id() == id)
        } else {
            self.items.contains_key(&id)
        }
    }

    pub fn values(&self, sort: Sort) -> Vec<&T> {
        let mut v: Vec<&T> = self.items.values().collect();
        match sort {
            Sort::Unordered => {}
            Sort::IdAsc => v.sort_by_key(|e| e.id()),
            Sort::IdDesc => v.sort_by_key(|e| std::cmp::Reverse(e.id())),
        }
        v
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.values_mut()
    }

    pub fn query(&self) -> Query<'_, T> {
        Query::new(self)
    }

    pub fn query_mut(&mut self) -> QueryMut<'_, T> {
        QueryMut::new(self)
    }

    pub fn begin_stage(&mut self, clear_existing: bool) -> Result<()> {
        if self.staged.is_some() {
            return Err(Error::Parse("Transaction already in progress.".into()));
        }
        if clear_existing {
            self.next_id = 1;
        }
        self.staged = Some(Staged {
            pending: Vec::new(),
            next_id_start: self.next_id,
            cleared: clear_existing,
        });
        Ok(())
    }

    pub fn discard_stage(&mut self) {
        if let Some(staged) = self.staged.take() {
            self.next_id = staged.next_id_start;
        }
    }

    pub fn staged_pending(&self) -> Option<&[T]> {
        self.staged.as_ref().map(|s| s.pending.as_slice())
    }

    pub fn staged_effective_ids(&self) -> Result<std::collections::HashSet<i32>> {
        let staged = self
            .staged
            .as_ref()
            .ok_or_else(|| Error::Parse("No active transaction to inspect.".into()))?;

        let mut ids: std::collections::HashSet<i32> = if staged.cleared {
            std::collections::HashSet::new()
        } else {
            self.items.keys().copied().collect()
        };

        for entity in &staged.pending {
            ids.insert(entity.id());
        }
        Ok(ids)
    }

    pub fn prepare_commit(&self) -> Result<PreparedRepo<T>>
    where
        T: Clone,
    {
        let Some(staged) = &self.staged else {
            return Err(Error::Parse("No active transaction to commit.".into()));
        };

        let mut items = if staged.cleared {
            HashMap::new()
        } else {
            self.items.clone()
        };

        for entity in &staged.pending {
            let id = entity.id();
            if items.contains_key(&id) {
                return Err(Error::Parse(format!(
                    "Entity with id {} already exists.",
                    id
                )));
            }
            items.insert(id, entity.clone());
        }

        let next_id = items
            .keys()
            .max()
            .map(|m| m + 1)
            .unwrap_or(1)
            .max(self.next_id);

        Ok(PreparedRepo { items, next_id })
    }

    pub fn apply_prepared(&mut self, prepared: PreparedRepo<T>) {
        self.items = prepared.items;
        self.next_id = prepared.next_id;
        self.staged = None;
    }
}

pub struct Query<'a, T: BaseEntity> {
    store: &'a dyn QueryableStore<T>,
    fs: FilterSorter<'a, T>,
}

impl<'a, T: BaseEntity> Query<'a, T> {
    fn new(store: &'a dyn QueryableStore<T>) -> Self {
        Self {
            store,
            fs: FilterSorter::new(),
        }
    }

    pub fn r#where(mut self, pred: impl Fn(&T) -> bool + 'a) -> Self {
        self.fs = self.fs.push_filter(pred);
        self
    }

    pub fn order(mut self, sort: Sort) -> Self {
        self.fs = self.fs.with_sort(sort);
        self
    }

    pub fn order_with(mut self, cmp: impl Fn(&T, &T) -> Ordering + 'a) -> Self {
        self.fs = self.fs.with_cmp(cmp);
        self
    }

    pub fn collect(self) -> Vec<&'a T> {
        let ids = self.fs.sorted_ids(self.store.items());
        ids.into_iter()
            .filter_map(|id| self.store.items().get(&id))
            .collect()
    }

    pub fn ids(self) -> Vec<i32> {
        self.collect().into_iter().map(|e| e.id()).collect()
    }

    pub fn exists(self) -> bool {
        if self.fs.filters.is_empty() {
            !self.store.items().is_empty()
        } else {
            self.store
                .items()
                .values()
                .any(|e| self.fs.filters.iter().all(|f| f(e)))
        }
    }
}

pub struct QueryMut<'a, T: BaseEntity> {
    store: &'a mut Repository<T>,
    fs: FilterSorter<'a, T>,
}

impl<'a, T: BaseEntity> QueryMut<'a, T> {
    fn new(store: &'a mut Repository<T>) -> Self {
        Self {
            store,
            fs: FilterSorter::new(),
        }
    }

    pub fn r#where(mut self, pred: impl Fn(&T) -> bool + 'a) -> Self {
        self.fs = self.fs.push_filter(pred);
        self
    }

    pub fn order(mut self, sort: Sort) -> Self {
        self.fs = self.fs.with_sort(sort);
        self
    }

    pub fn order_with(mut self, cmp: impl Fn(&T, &T) -> Ordering + 'a) -> Self {
        self.fs = self.fs.with_cmp(cmp);
        self
    }

    pub fn for_each_mut<F>(self, mut f: F)
    where
        F: FnMut(&mut T),
    {
        let ids = self.fs.sorted_ids(&self.store.items);

        for id in ids {
            if let Some(item) = self.store.items.get_mut(&id) {
                f(item);
            }
        }
    }
}
