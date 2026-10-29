//! gpui-kit widgets as zopra components, in the spirit of shadcn/ui: every tag takes its
//! content as children and accepts a `class`.
//!
//! ```ignore
//! use zopra::ui::*;
//!
//! view! {
//!     <TabBar selected_index={tab()} on_click={|index, _, cx| tab(*index)} underline>
//!         <Tab label="Intercept" />
//!         <Tab label="History" />
//!     </TabBar>
//! }
//! ```

mod dropdown_button;
mod resizable;
mod settings;
mod sidebar;
mod tabs;
mod title_bar;

pub use dropdown_button::*;
pub use resizable::*;
pub use settings::*;
pub use sidebar::*;
pub use tabs::*;
pub use title_bar::*;

use gpui_kit::{Refineable, StyleRefinement, Styled};

/// Applies a component's `class` on top of a widget's own style.
fn styled<E: Styled>(mut element: E, class: Option<StyleRefinement>) -> E {
    if let Some(class) = class {
        element.style().refine(&class);
    }
    element
}
