use std::ops::Range;

use gpui_kit::component::menu::PopupMenu;
use gpui_kit::component::table::{Column, ColumnGroup, ColumnSort, TableDelegate, TableState};
use gpui_kit::{AnyElement, App, Context, Div, IntoElement, Stateful, Window, div};
use gpui_kit::prelude::*;

pub struct DeclarativeTableDelegate<T>
where
    T: Clone + 'static,
{
    pub data: Vec<T>,
    pub columns: Vec<Column>,
    pub group_headers: Option<Vec<Vec<ColumnGroup>>>,

    #[allow(clippy::type_complexity)]
    pub render_row: Option<Box<dyn Fn(usize, &T, &mut Window, &mut Context<TableState<Self>>) -> Stateful<Div>>>,
    #[allow(clippy::type_complexity)]
    pub render_cell: Box<dyn Fn(&T, &str, &mut Window, &mut Context<TableState<Self>>) -> AnyElement>,
    #[allow(clippy::type_complexity)]
    pub on_sort: Option<Box<dyn Fn(&str, ColumnSort)>>,
    #[allow(clippy::type_complexity)]
    pub on_context_menu: Option<Box<dyn Fn(&T, usize, PopupMenu, &mut Window, &mut Context<TableState<Self>>) -> PopupMenu>>,
    #[allow(clippy::type_complexity)]
    pub on_lazy_load: Option<Box<dyn Fn()>>,

    // Internal virtualized state tracking
    pub loading: bool,
    pub eof: bool,
    pub visible_rows: Range<usize>,
    pub visible_cols: Range<usize>,
}

impl<T: Clone + 'static> DeclarativeTableDelegate<T> {
    #[must_use]
    pub fn new(
        data: Vec<T>,
        columns: Vec<Column>,
        render_cell: impl Fn(&T, &str, &mut Window, &mut Context<TableState<Self>>) -> AnyElement + 'static,
    ) -> Self {
        Self {
            data,
            columns,
            group_headers: None,
            render_row: None,
            render_cell: Box::new(render_cell),
            on_sort: None,
            on_context_menu: None,
            on_lazy_load: None,
            loading: false,
            eof: false,
            visible_rows: 0..0,
            visible_cols: 0..0,
        }
    }

    #[must_use]
    pub fn render_row(
        mut self,
        handler: impl Fn(usize, &T, &mut Window, &mut Context<TableState<Self>>) -> Stateful<Div> + 'static,
    ) -> Self {
        self.render_row = Some(Box::new(handler));
        self
    }

    /// Chains a sorting callback
    #[must_use]
    pub fn on_sort(mut self, handler: impl Fn(&str, ColumnSort) + 'static) -> Self {
        self.on_sort = Some(Box::new(handler));
        self
    }

    /// Chains a context menu callback
    #[must_use]
    pub fn on_context_menu(mut self, handler: impl Fn(&T, usize, PopupMenu, &mut Window, &mut Context<TableState<Self>>) -> PopupMenu + 'static) -> Self {
        self.on_context_menu = Some(Box::new(handler));
        self
    }

    /// Chains a lazy load callback
    #[must_use]
    pub fn on_lazy_load(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_lazy_load = Some(Box::new(handler));
        self
    }

    /// Applies custom group headers
    #[must_use]
    pub fn group_headers(mut self, headers: Vec<Vec<ColumnGroup>>) -> Self {
        self.group_headers = Some(headers);
        self
    }
}

impl<T: Clone + 'static> TableDelegate for DeclarativeTableDelegate<T> {
    fn columns_count(&self, _cx: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _cx: &App) -> usize {
        self.data.len()
    }

    fn column(&self, col_ix: usize, _cx: &App) -> Column {
        self.columns.get(col_ix).cloned().unwrap_or_else(|| Column::new("unknown", "Unknown"))
    }

    fn group_headers(&self, _cx: &App) -> Option<Vec<Vec<ColumnGroup>>> {
        self.group_headers.clone()
    }

    fn render_th(
        &mut self,
        col_ix: usize,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let col = self.column(col_ix, cx);
        div().child(col.name)
    }

    fn context_menu(
        &mut self,
        row_ix: usize,
        menu: PopupMenu,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> PopupMenu {
        if let (Some(handler), Some(item)) = (&self.on_context_menu, self.data.get(row_ix)) {
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
        if let Some(handler) = &self.render_row
            && let Some(item) = self.data.get(row_ix) {
                return handler(row_ix, item, window, cx);
            }
        div().id(row_ix)
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let col_key = self.columns.get(col_ix).map(|c| c.key.as_ref()).unwrap_or("");
        
        if let Some(item) = self.data.get(row_ix) {
            (self.render_cell)(item, col_key, window, cx)
        } else {
            div().into_any_element()
        }
    }

    fn perform_sort(
        &mut self,
        col_ix: usize,
        sort: ColumnSort,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) {
        if let (Some(handler), Some(col)) = (&self.on_sort, self.columns.get(col_ix)) {
            handler(col.key.as_ref(), sort);
        }
    }

    fn loading(&self, _cx: &App) -> bool {
        self.loading
    }

    fn has_more(&self, _cx: &App) -> bool {
        self.on_lazy_load.is_some() && !self.loading && !self.eof
    }

    fn load_more(&mut self, _window: &mut Window, _cx: &mut Context<TableState<Self>>) {
        if let Some(handler) = &self.on_lazy_load {
            handler();
        }
    }

    fn visible_rows_changed(
        &mut self,
        visible_range: Range<usize>,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) {
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

