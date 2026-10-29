use std::ops::Range;

use gpui_kit::component::setting::{self as kit, NumberFieldOptions, RenderOptions, SettingField};
use gpui_kit::component::group_box::GroupBoxVariant;
use gpui_kit::component::{Icon, Sizable, Size};
use gpui_kit::{App, Axis, ElementId, IntoElement, Pixels, SharedString, Window};

use crate::{Children, component};

#[component]
pub fn settings(
    id: Option<ElementId>,
    size: Option<Size>,
    group_variant: Option<GroupBoxVariant>,
    sidebar_width: Option<Pixels>,
    sidebar_size_range: Option<Range<Pixels>>,
    children: Children<kit::SettingPage>,
) -> kit::Settings {
    let mut settings = kit::Settings::new(id.unwrap_or_else(|| "settings".into())).pages(children());
    if let Some(size) = size {
        settings = settings.with_size(size);
    }
    if let Some(variant) = group_variant {
        settings = settings.with_group_variant(variant);
    }
    if let Some(width) = sidebar_width {
        settings = settings.sidebar_width(width);
    }
    if let Some(range) = sidebar_size_range {
        settings = settings.sidebar_size_range(range);
    }
    settings
}

#[component]
pub fn setting_page(
    title: SharedString,
    description: Option<SharedString>,
    icon: Option<Icon>,
    default_open: Option<bool>,
    resettable: Option<bool>,
    children: Children<kit::SettingGroup>,
) -> kit::SettingPage {
    let mut page = kit::SettingPage::new(title)
        .default_open(default_open.unwrap_or_default())
        .groups(children());
    if let Some(description) = description {
        page = page.description(description);
    }
    if let Some(icon) = icon {
        page = page.icon(icon);
    }
    if let Some(resettable) = resettable {
        page = page.resettable(resettable);
    }
    page
}

#[component]
pub fn setting_group(
    title: Option<SharedString>,
    description: Option<SharedString>,
    variant: Option<GroupBoxVariant>,
    children: Children<kit::SettingItem>,
) -> kit::SettingGroup {
    let mut group = kit::SettingGroup::new().items(children());
    if let Some(title) = title {
        group = group.title(title);
    }
    if let Some(description) = description {
        group = group.description(description);
    }
    if let Some(variant) = variant {
        group = group.variant(variant);
    }
    group
}

pub enum Field {
    Toggle(SettingField<bool>),
    Text(SettingField<SharedString>),
    Number(SettingField<f64>),
}

impl From<SettingField<bool>> for Field {
    fn from(field: SettingField<bool>) -> Self {
        Self::Toggle(field)
    }
}

impl From<SettingField<SharedString>> for Field {
    fn from(field: SettingField<SharedString>) -> Self {
        Self::Text(field)
    }
}

impl From<SettingField<f64>> for Field {
    fn from(field: SettingField<f64>) -> Self {
        Self::Number(field)
    }
}

#[component]
pub fn setting_item(
    title: SharedString,
    description: Option<SharedString>,
    layout: Option<Axis>,
    keywords: Option<Vec<SharedString>>,
    disabled: Option<bool>,
    children: Children<Field>,
) -> kit::SettingItem {
    let field = children()
        .into_iter()
        .next()
        .expect("a <SettingItem> holds one field, e.g. <SettingDropdown>");
    let mut item = match field {
        Field::Toggle(field) => kit::SettingItem::new(title, field),
        Field::Text(field) => kit::SettingItem::new(title, field),
        Field::Number(field) => kit::SettingItem::new(title, field),
    }
    .disabled(disabled.unwrap_or_default());
    if let Some(description) = description {
        item = item.description(description);
    }
    if let Some(layout) = layout {
        item = item.layout(layout);
    }
    if let Some(keywords) = keywords {
        item = item.keywords(keywords);
    }
    item
}

#[component]
pub fn setting_custom_item<R, E>(content: R) -> kit::SettingItem
where
    R: Fn(&RenderOptions, &mut Window, &mut App) -> E + 'static,
    E: IntoElement + 'static,
{
    kit::SettingItem::render(content)
}

#[component]
pub fn setting_switch<V, S>(value: V, on_change: S, default_value: Option<bool>) -> Field
where
    V: Fn(&App) -> bool + 'static,
    S: Fn(bool, &mut App) + 'static,
{
    with_default(SettingField::switch(value, on_change), default_value)
}

#[component]
pub fn setting_checkbox<V, S>(value: V, on_change: S, default_value: Option<bool>) -> Field
where
    V: Fn(&App) -> bool + 'static,
    S: Fn(bool, &mut App) + 'static,
{
    with_default(SettingField::checkbox(value, on_change), default_value)
}

#[component]
pub fn setting_text_input<V, S>(value: V, on_change: S, default_value: Option<SharedString>) -> Field
where
    V: Fn(&App) -> SharedString + 'static,
    S: Fn(SharedString, &mut App) + 'static,
{
    with_default(SettingField::input(value, on_change), default_value)
}

#[component]
pub fn setting_dropdown<V, S>(
    options: Vec<(SharedString, SharedString)>,
    value: V,
    on_change: S,
    scrollable: Option<bool>,
    default_value: Option<SharedString>,
) -> Field
where
    V: Fn(&App) -> SharedString + 'static,
    S: Fn(SharedString, &mut App) + 'static,
{
    let field = if scrollable.unwrap_or_default() {
        SettingField::scrollable_dropdown(options, value, on_change)
    } else {
        SettingField::dropdown(options, value, on_change)
    };
    with_default(field, default_value)
}

#[component]
pub fn setting_number_input<V, S>(
    options: NumberFieldOptions,
    value: V,
    on_change: S,
    default_value: Option<f64>,
) -> Field
where
    V: Fn(&App) -> f64 + 'static,
    S: Fn(f64, &mut App) + 'static,
{
    with_default(SettingField::number_input(options, value, on_change), default_value)
}

#[component]
pub fn setting_custom_field<R, E>(content: R) -> Field
where
    R: Fn(&RenderOptions, &mut Window, &mut App) -> E + 'static,
    E: IntoElement + 'static,
{
    SettingField::render(content).into()
}

fn with_default<T>(field: SettingField<T>, default_value: Option<T>) -> Field
where
    SettingField<T>: Into<Field>,
{
    match default_value {
        Some(value) => field.default_value(value).into(),
        None => field.into(),
    }
}

crate::typed_child!(Field, kit::SettingPage, kit::SettingGroup, kit::SettingItem);
