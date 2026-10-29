use std::cmp::Ordering;
use std::ops::Range;

use gpui_kit::component::menu::PopupMenu;
use gpui_kit::component::table::{Column, ColumnGroup, ColumnSort, TableDelegate, TableState};
use gpui_kit::prelude::*;
use gpui_kit::{AnyElement, App, Context, Div, IntoElement, Stateful, Window, div};

use crate::hooks::Signal;

type CellFn<T> = dyn Fn(&T, &str) -> AnyElement;

type LoadCallback = Box<dyn Fn(&mut App)>;

/// Table rows: owned, or read straight from a signal without copying.
pub enum Rows<T: 'static> {
    Owned(Vec<T>),
    Signal(Signal<Vec<T>>),
}

impl<T> From<Vec<T>> for Rows<T> {
    fn from(rows: Vec<T>) -> Self {
        Self::Owned(rows)
    }
}

impl<T> From<Signal<Vec<T>>> for Rows<T> {
    fn from(rows: Signal<Vec<T>>) -> Self {
        Self::Signal(rows)
    }
}

impl<T: 'static> Rows<T> {
    fn slice<'a>(&'a self, cx: &'a App) -> &'a [T] {
        match self {
            Self::Owned(rows) => rows,
            Self::Signal(rows) => rows.try_read(cx).map_or(&[], Vec::as_slice),
        }
    }

    /// An empty slice of the row type, so generated code can infer `T`.
    #[must_use]
    pub fn probe(&self) -> &'static [T] {
        &[]
    }
}

pub type SortFn<T> = dyn Fn(&[T]) -> Vec<usize>;

pub fn col_sorter<T: 'static, F, V>(f: F, _rows: &[T]) -> Box<SortFn<T>>
where
    F: Fn(&T) -> V + 'static,
    V: PartialOrd + 'static,
{
    Box::new(move |rows| {
        let mut keyed = rows
            .iter()
            .enumerate()
            .map(|(index, row)| (index, f(row)))
            .collect::<Vec<_>>();
        keyed.sort_by(|(_, left), (_, right)| left.partial_cmp(right).unwrap_or(Ordering::Equal));
        keyed.into_iter().map(|(index, _)| index).collect()
    })
}

pub fn any_order<T: 'static>() -> Box<SortFn<T>> {
    Box::new(|rows| (0..rows.len()).collect())
}

pub fn cell_of<T: 'static, F, V: IntoElement>(f: F, probe: &T) -> AnyElement
where
    F: Fn(&T) -> V,
{
    _ = probe;
    f(probe).into_any_element()
}

pub struct DeclarativeTableDelegate<T>
where
    T: Clone + 'static,
{
    data: Rows<T>,
    /// version of a signal's rows when they were last sorted
    sorted_version: Option<u32>,
    order: Option<Vec<usize>>,
    active_sort: Option<(usize, ColumnSort)>,
    columns: Vec<Column>,
    group_headers: Option<Vec<Vec<ColumnGroup>>>,

    #[allow(clippy::type_complexity)]
    render_row: Option<
        Box<dyn Fn(usize, &T, &mut Window, &mut Context<TableState<Self>>) -> Stateful<Div>>,
    >,
    #[allow(clippy::type_complexity)]
    render_cell: Box<CellFn<T>>,
    #[allow(clippy::type_complexity)]
    on_sort: Option<Box<dyn Fn(&Column, ColumnSort, &mut App)>>,
    #[allow(clippy::type_complexity)]
    on_context_menu: Option<
        Box<dyn Fn(&T, usize, PopupMenu, &mut Window, &mut Context<TableState<Self>>) -> PopupMenu>,
    >,
    on_lazy_load: Option<LoadCallback>,
    explicit_row_count: Option<usize>,
    data_offset: usize,
    #[allow(clippy::type_complexity)]
    on_visible_rows_changed: Option<Box<dyn Fn(Range<usize>, &mut App)>>,
    sorters: Vec<Option<Box<SortFn<T>>>>,

    lazy_loading: bool,
    eof: bool,
    full_loading: bool,
    visible_rows: Range<usize>,
    visible_cols: Range<usize>,
}

impl<T: Clone + 'static> DeclarativeTableDelegate<T> {
    #[must_use]
    pub fn new<C>(data: Rows<T>, columns: Vec<Column>, render_cell: C) -> Self
    where
        C: Fn(&T, &str) -> AnyElement + 'static,
    {
        let sorters = columns.iter().map(|_| None).collect();
        Self {
            data,
            sorted_version: None,
            order: None,
            active_sort: None,
            columns,
            group_headers: None,
            render_row: None,
            render_cell: Box::new(render_cell),
            on_sort: None,
            on_context_menu: None,
            on_lazy_load: None,
            explicit_row_count: None,
            data_offset: 0,
            on_visible_rows_changed: None,
            sorters,
            lazy_loading: false,
            eof: false,
            full_loading: false,
            visible_rows: 0..0,
            visible_cols: 0..0,
        }
    }

    /// Takes this render's rows; returns whether the table needs to re-render.
    /// Owned rows always count as new. Rows from the same signal only do once it was written.
    pub fn update_data(&mut self, data: Rows<T>, cx: &App) -> bool {
        let version = match &data {
            Rows::Signal(rows) => rows.version(cx),
            Rows::Owned(_) => None,
        };
        // a table built from a signal hasn't seen its version yet, but already shows its rows
        let unchanged = match (&self.data, &data) {
            (Rows::Signal(old), Rows::Signal(new)) => {
                old.same(*new) && self.sorted_version.is_none_or(|seen| Some(seen) == version)
            }
            _ => false,
        };
        self.data = data;
        self.sorted_version = version;
        if unchanged {
            return false;
        }
        if let Some((column, sort)) = self.active_sort {
            self.set_order(column, sort, cx);
        }
        true
    }

    fn set_order(&mut self, column: usize, sort: ColumnSort, cx: &App) {
        if let Rows::Signal(rows) = &self.data {
            self.sorted_version = rows.version(cx);
        }
        if sort == ColumnSort::Default {
            self.order = None;
            self.active_sort = None;
            return;
        }
        let Some(sorter) = self.sorters.get(column).and_then(Option::as_ref) else {
            self.order = None;
            self.active_sort = None;
            return;
        };
        let mut order = sorter(self.data.slice(cx));
        if sort == ColumnSort::Descending {
            order.reverse();
        }
        self.order = Some(order);
        self.active_sort = Some((column, sort));
    }

    fn data_index(&self, row: usize) -> usize {
        self.order
            .as_ref()
            .and_then(|order| order.get(row))
            .copied()
            .unwrap_or(row)
    }

    pub fn resolved_index(&self, row_ix: usize) -> usize {
        row_ix
            .checked_sub(self.data_offset)
            .map(|row| self.data_index(row) + self.data_offset)
            .unwrap_or(row_ix)
    }

    fn get_data<'a>(&'a self, row_ix: usize, cx: &'a App) -> Option<&'a T> {
        let row = row_ix.checked_sub(self.data_offset)?;
        self.data.slice(cx).get(self.data_index(row))
    }

    #[must_use]
    pub fn render_row(
        mut self,
        handler: impl Fn(usize, &T, &mut Window, &mut Context<TableState<Self>>) -> Stateful<Div>
        + 'static,
    ) -> Self {
        self.render_row = Some(Box::new(handler));
        self
    }

    #[must_use]
    pub fn on_sort(mut self, handler: impl Fn(&Column, ColumnSort, &mut App) + 'static) -> Self {
        self.on_sort = Some(Box::new(handler));
        self
    }

    #[must_use]
    pub fn on_context_menu(
        mut self,
        handler: impl Fn(&T, usize, PopupMenu, &mut Window, &mut Context<TableState<Self>>) -> PopupMenu
        + 'static,
    ) -> Self {
        self.on_context_menu = Some(Box::new(handler));
        self
    }

    #[must_use]
    pub fn rows_count(mut self, count: usize) -> Self {
        self.explicit_row_count = Some(count);
        self
    }

    #[must_use]
    pub fn data_offset(mut self, offset: usize) -> Self {
        self.data_offset = offset;
        self
    }

    #[must_use]
    pub fn on_visible_rows_changed(mut self, handler: impl Fn(Range<usize>, &mut App) + 'static) -> Self {
        self.on_visible_rows_changed = Some(Box::new(handler));
        self
    }

    pub fn set_rows_count(&mut self, count: usize) {
        self.explicit_row_count = Some(count);
    }

    pub fn set_data_offset(&mut self, offset: usize) {
        self.data_offset = offset;
    }

    pub fn set_on_visible_rows_changed(&mut self, handler: impl Fn(Range<usize>, &mut App) + 'static) {
        self.on_visible_rows_changed = Some(Box::new(handler));
    }

    #[must_use]
    pub fn on_lazy_load(mut self, handler: impl Fn(&mut App) + 'static) -> Self {
        self.on_lazy_load = Some(Box::new(handler));
        self
    }

    #[must_use]
    pub fn sorters(mut self, sorters: Vec<Option<Box<SortFn<T>>>>) -> Self {
        debug_assert_eq!(sorters.len(), self.columns.len());
        self.sorters = sorters;
        self
    }

    #[must_use]
    pub fn group_headers(mut self, headers: Vec<Vec<ColumnGroup>>) -> Self {
        self.group_headers = Some(headers);
        self
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.lazy_loading = loading;
    }

    pub fn set_full_loading(&mut self, full_loading: bool) {
        self.full_loading = full_loading;
    }

    pub fn set_eof(&mut self, eof: bool) {
        self.eof = eof;
    }

    #[must_use]
    pub fn is_loading(&self) -> bool {
        self.lazy_loading
    }

    #[must_use]
    pub fn eof(&self) -> bool {
        self.eof
    }

    #[must_use]
    pub fn visible_rows(&self) -> Range<usize> {
        self.visible_rows.clone()
    }

    #[must_use]
    pub fn visible_cols(&self) -> Range<usize> {
        self.visible_cols.clone()
    }
}

/// Context menus and custom rows need `cx` mutably, so they get a clone of their one row.
impl<T: Clone + 'static> TableDelegate for DeclarativeTableDelegate<T> {
    fn columns_count(&self, _cx: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, cx: &App) -> usize {
        self.explicit_row_count
            .unwrap_or_else(|| self.data.slice(cx).len())
    }

    fn column(&self, col_ix: usize, _cx: &App) -> Column {
        self.columns[col_ix].clone()
    }

    fn group_headers(&self, _cx: &App) -> Option<Vec<Vec<ColumnGroup>>> {
        self.group_headers.clone()
    }

    fn move_column(
          &mut self,
          col_ix: usize,
          to_ix: usize,
          _window: &mut Window,
          _cx: &mut Context<TableState<Self>>,
      ) {
          let col = self.columns.remove(col_ix);
          self.columns.insert(to_ix, col);
          if col_ix < self.sorters.len() && to_ix < self.sorters.len() {
              let sorter = self.sorters.remove(col_ix);
              self.sorters.insert(to_ix, sorter);
          }
          self.active_sort = None;
          self.order = None;
      }

    fn render_th(
        &mut self,
        col_ix: usize,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let name = self
            .columns
            .get(col_ix)
            .map(|column| column.name.clone())
            .unwrap_or_default();
        div().child(name)
    }

    fn context_menu(
        &mut self,
        row_ix: usize,
        menu: PopupMenu,
        window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> PopupMenu {
        match (&self.on_context_menu, self.get_data(row_ix, cx).cloned()) {
            (Some(handler), Some(item)) => handler(&item, self.resolved_index(row_ix), menu, window, cx),
            _ => menu,
        }
    }

    fn render_tr(
        &mut self,
        row_ix: usize,
        window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> Stateful<Div> {
        if let Some(handler) = &self.render_row
            && let Some(item) = self.get_data(row_ix, cx).cloned()
        {
            return handler(self.resolved_index(row_ix), &item, window, cx);
        }
        div().id(("row", row_ix))
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        match (self.columns.get(col_ix), self.get_data(row_ix, cx)) {
            (Some(col), Some(item)) => (self.render_cell)(item, col.key.as_ref()),
            _ => div().into_any_element(),
        }
    }

    fn perform_sort(
        &mut self,
        col_ix: usize,
        sort: ColumnSort,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) {
        if let Some(handler) = &self.on_sort {
            if let Some(col) = self.columns.get(col_ix) {
                handler(col, sort, cx);
            }
            return;
        }
        self.set_order(col_ix, sort, cx);
        cx.notify();
    }

    fn has_more(&self, _cx: &App) -> bool {
        self.on_lazy_load.is_some() && !self.lazy_loading && !self.eof
    }

    fn loading(&self, _cx: &App) -> bool {
        self.full_loading
    }

    fn load_more(&mut self, _window: &mut Window, cx: &mut Context<TableState<Self>>) {
      if self.lazy_loading {
              return;
          }
      self.lazy_loading = true;
      if let Some(handler) = &self.on_lazy_load {
          handler(cx);
      }
    }

    fn visible_rows_changed(
        &mut self,
        visible_range: Range<usize>,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) {
        if let Some(handler) = &self.on_visible_rows_changed {
            use std::ops::DerefMut;
            handler(visible_range.clone(), cx.deref_mut());
        }
        self.visible_rows = visible_range;
    }

    fn visible_columns_changed(
        &mut self,
        visible_range: Range<usize>,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) {
        self.visible_cols = visible_range;
    }
}
