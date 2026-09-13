// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! [`FilteredVec`] is an append only vector which allows to filter its items. The
//! filter result is cached so that the filter doesn't need to be applied each time
//! the vector is iterated.
//!
//! The elements of the vector need to implement [`Filterable`].

#[derive(Debug, Clone)]
pub struct Item<W> {
    pub inner: W,
    visible: bool,
}

impl<W> Item<W> {
    pub fn visible(&self) -> bool {
        self.visible
    }
}

#[derive(Debug, Clone)]
pub struct Filter {
    filter_str: String,
}

impl Filter {
    fn new() -> Filter {
        Self {
            filter_str: String::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.filter_str.is_empty()
    }

    pub fn apply(&mut self, raw: &str) -> bool {
        raw.contains(&self.filter_str)
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct FilteredVec<W> {
    items: Vec<Item<W>>,

    filter: Filter,
}

impl<W: Filterable> FilteredVec<W> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            filter: Filter::new(),
        }
    }

    pub fn push(&mut self, widget: W) {
        self.items.push(Item {
            visible: widget.apply(&mut self.filter),
            inner: widget,
        });
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Item<W>> {
        self.items.iter_mut()
    }

    pub fn update(&mut self) {
        if self.filter.is_empty() {
            self.show_all();
            return;
        }
        for item in &mut self.items {
            item.visible = item.inner.apply(&mut self.filter);
        }
    }

    fn show_all(&mut self) {
        for item in &mut self.items {
            item.visible = true;
        }
    }

    pub fn filter_string(&mut self) -> &mut String {
        &mut self.filter.filter_str
    }
}

pub trait Filterable {
    fn apply(&self, filter: &mut Filter) -> bool;
}
