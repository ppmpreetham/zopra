use std::cmp::Ordering;
use std::ops::Range;
use std::rc::Rc;

use gpui_kit::component::menu::PopupMenu;
use gpui_kit::component::table::{Column, ColumnGroup, ColumnSort, TableDelegate, TableState};
use gpui_kit::prelude::*;
use gpui_kit::{AnyElement, App, Context, Div, IntoElement, Stateful, Window, div};

type CellFn<T> = dyn Fn(
    &T,
    &str,
    &mut Window,
    &mut Context<TableState<DeclarativeTableDelegate<T>>>,
) -> AnyElement;

type LoadCallback = Rc<dyn Fn(&mut App)>;

pub trait IntoRows<T> {
    fn into_rows(self) -> Rc<Vec<T>>;
}

impl<T> IntoRows<T> for Vec<T> {
    fn into_rows(self) -> Rc<Vec<T>> {
        Rc::new(self)
    }
}
impl<T> IntoRows<T> for Rc<Vec<T>> {
    fn into_rows(self) -> Rc<Vec<T>> {
        self
    }
}
impl<T> IntoRows<T> for crate::hooks::Snap<Vec<T>> {
    fn into_rows(self) -> Rc<Vec<T>> {
        self.into_rc()
    }
}
pub type SortFn<T> = dyn Fn(&[T]) -> Vec<usize>;

pub fn col_sorter<T: 'static, F, V>(f: F, _rows: &[T]) -> Rc<SortFn<T>>
where
    F: Fn(&T) -> V + 'static,
    V: PartialOrd + 'static,
{
    Rc::new(move |rows| {
        let mut keyed = rows
            .iter()
            .enumerate()
            .map(|(index, row)| (index, f(row)))
            .collect::<Vec<_>>();
        keyed.sort_by(|(_, left), (_, right)| left.partial_cmp(right).unwrap_or(Ordering::Equal));
        keyed.into_iter().map(|(index, _)| index).collect()
    })
}

pub fn any_order<T: 'static>() -> Rc<SortFn<T>> {
    Rc::new(|rows| (0..rows.len()).collect())
}

pub fn col_cell<T: 'static, F, V: IntoElement>(f: F, probe: &T) -> AnyElement
where
    F: Fn(&T) -> V,
{
    _ = probe;
    f(probe).into_any_element()
}

pub struct DeclarativeTableDelegate<T>
where
    T: 'static,
{
    pub data: Rc<Vec<T>>,
    order: Option<Vec<usize>>,
    active_sort: Option<(usize, ColumnSort)>,
    pub columns: Vec<Column>,
    pub group_headers: Option<Vec<Vec<ColumnGroup>>>,

    #[allow(clippy::type_complexity)]
    pub render_row: Option<
        Box<dyn Fn(usize, &T, &mut Window, &mut Context<TableState<Self>>) -> Stateful<Div>>,
    >,
    #[allow(clippy::type_complexity)]
    pub render_cell: Box<CellFn<T>>,
    #[allow(clippy::type_complexity)]
    pub on_sort: Option<Rc<dyn Fn(&Column, ColumnSort, &mut App)>>,
    #[allow(clippy::type_complexity)]
    pub on_context_menu: Option<
        Box<dyn Fn(&T, usize, PopupMenu, &mut Window, &mut Context<TableState<Self>>) -> PopupMenu>,
    >,
    pub on_lazy_load: Option<LoadCallback>,
    pub explicit_row_count: Option<usize>,
    pub data_offset: usize,
    #[allow(clippy::type_complexity)]
    pub on_visible_rows_changed: Option<Rc<dyn Fn(Range<usize>, &mut App)>>,
    sorters: Vec<Option<Rc<SortFn<T>>>>,

    loading: bool,
    eof: bool,
    visible_rows: Range<usize>,
    visible_cols: Range<usize>,
}

impl<T: 'static> DeclarativeTableDelegate<T> {
    #[must_use]
    pub fn new<C>(data: Rc<Vec<T>>, columns: Vec<Column>, render_cell: C) -> Self
    where
        C: Fn(&T, &str, &mut Window, &mut Context<TableState<Self>>) -> AnyElement + 'static,
    {
        let sorters = vec![None; columns.len()];
        Self {
            data,
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
            loading: false,
            eof: false,
            visible_rows: 0..0,
            visible_cols: 0..0,
        }
    }

    pub fn update_data(&mut self, data: &Rc<Vec<T>>) -> bool {
        if Rc::ptr_eq(&self.data, data) {
            return false;
        }
        self.data = data.clone();
        if let Some((column, sort)) = self.active_sort {
            self.set_order(column, sort);
        }
        true
    }

    fn set_order(&mut self, column: usize, sort: ColumnSort) {
        if sort == ColumnSort::Default {
            self.order = None;
            self.active_sort = None;
            return;
        }
        let Some(sorter) = self.sorters.get(column).and_then(Option::as_ref) else {
            return;
        };
        let mut order = sorter(self.data.as_slice());
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

    fn get_data(&self, row_ix: usize) -> Option<&T> {
        let row = row_ix.checked_sub(self.data_offset)?;
        self.data.get(self.data_index(row))
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
        self.on_sort = Some(Rc::new(handler));
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
        self.on_visible_rows_changed = Some(Rc::new(handler));
        self
    }

    pub fn set_rows_count(&mut self, count: usize) {
        self.explicit_row_count = Some(count);
    }

    pub fn set_data_offset(&mut self, offset: usize) {
        self.data_offset = offset;
    }

    pub fn set_on_visible_rows_changed(&mut self, handler: impl Fn(Range<usize>, &mut App) + 'static) {
        self.on_visible_rows_changed = Some(Rc::new(handler));
    }

    #[must_use]
    pub fn on_lazy_load(mut self, handler: impl Fn(&mut App) + 'static) -> Self {
        self.on_lazy_load = Some(Rc::new(handler));
        self
    }

    #[must_use]
    pub fn sorters(mut self, sorters: Vec<Option<Rc<SortFn<T>>>>) -> Self {
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
        self.loading = loading;
    }

    pub fn set_eof(&mut self, eof: bool) {
        self.eof = eof;
    }

    #[must_use]
    pub fn loading(&self) -> bool {
        self.loading
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

impl<T: 'static> TableDelegate for DeclarativeTableDelegate<T> {
    fn columns_count(&self, _cx: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _cx: &App) -> usize {
      self.explicit_row_count.unwrap_or(self.data.len())
    }

    fn column(&self, col_ix: usize, _cx: &App) -> Column {
        debug_assert!(
            col_ix < self.columns.len(),
            "column index {col_ix} out of bounds ({} columns)",
            self.columns.len()
        );
        self.columns
            .get(col_ix)
            .cloned()
            .unwrap_or_else(|| Column::new("unknown", "Unknown"))
    }

    fn group_headers(&self, _cx: &App) -> Option<Vec<Vec<ColumnGroup>>> {
        self.group_headers.clone()
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
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> PopupMenu {
        if let (Some(handler), Some(item)) = (&self.on_context_menu, self.get_data(row_ix)) {
            handler(item, row_ix, menu, _window, _cx)
        } else {
            menu
        }
    }

    fn render_tr(
        &mut self,
        row_ix: usize,
        window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> Stateful<Div> {
        let data_ix = self.data_index(row_ix);
        if let Some(handler) = &self.render_row
            && let Some(item) = self.get_data(row_ix)
        {
            return handler(data_ix, item, window, cx);
        }
        div().id(("row", row_ix))
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let Some(col) = self.columns.get(col_ix) else {
            return div().into_any_element();
        };

        if let Some(item) = self.get_data(row_ix) {
            (self.render_cell)(item, col.key.as_ref(), window, cx)
        } else {
            div().into_any_element()
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
        self.set_order(col_ix, sort);
        cx.notify();
    }

    fn has_more(&self, _cx: &App) -> bool {
        self.on_lazy_load.is_some() && !self.loading && !self.eof
    }

    fn load_more(&mut self, _window: &mut Window, cx: &mut Context<TableState<Self>>) {
      if self.loading {
              return;
          }
      self.loading = true;
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
