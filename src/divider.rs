use iced::advanced::layout::{self, Layout};
use iced::mouse::Cursor;
use iced::advanced::widget::{self, Widget};
use iced::wgpu::naga::back;
use iced::{
    event, mouse, overlay, padding, Color, Element, Length, Point, Rectangle, Size, Vector,
};
use iced::advanced::{renderer, Clipboard, Overlay, Shell};

use crate::style;

#[derive(Clone, Copy, Debug, Default)]
struct State {
    drag_origin: Option<Point>,
    is_divider_hovered: bool,
    show_context_menu: bool,
    context_menu_position: Point,
}

/// Messages for column visibility management
#[derive(Debug, Clone)]
pub enum ColumnVisibilityMessage {
    /// Toggle visibility of a column by ID
    ToggleColumn(String),
    /// Hide the context menu
    HideContextMenu,
}

pub(crate) struct Divider<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
    Theme: style::Catalog,
{
    content: Element<'a, Message, Theme, Renderer>,
    width: f32,
    column_id: String,
    column_title: String,
    on_drag: Box<dyn Fn(f32) -> Message + 'a>,
    on_release: Message,
    on_column_visibility: Option<Box<dyn Fn(ColumnVisibilityMessage) -> Message + 'a>>,
    style: <Theme as style::Catalog>::Style,
    // List of other columns that can be toggled
    other_columns: Vec<(String, String, bool)>, // (id, title, visible)
    // New field to control divider visibility
    always_show_divider: bool,
}

impl<'a, Message, Theme, Renderer> Divider<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
    Theme: style::Catalog,
{
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        width: f32,
        column_id: String,
        column_title: String,
        on_drag: impl Fn(f32) -> Message + 'a,
        on_release: Message,
        style: <Theme as style::Catalog>::Style,
    ) -> Self {
        Self {
            content: content.into(),
            width,
            column_id,
            column_title,
            on_drag: Box::new(on_drag),
            on_release,
            on_column_visibility: None,
            style,
            other_columns: Vec::new(),
            always_show_divider: false,
        }
    }

    pub fn with_column_visibility(
        mut self,
        on_column_visibility: impl Fn(ColumnVisibilityMessage) -> Message + 'a,
        other_columns: Vec<(String, String, bool)>,
    ) -> Self {
        self.on_column_visibility = Some(Box::new(on_column_visibility));
        self.other_columns = other_columns;
        self
    }

    // New method to control divider visibility
    pub fn always_show_divider(mut self, always_show: bool) -> Self {
        self.always_show_divider = always_show;
        self
    }

    fn divider_bounds(&self, bounds: Rectangle) -> Rectangle {
        Rectangle {
            x: bounds.x + bounds.width - self.width,
            width: self.width,
            ..bounds
        }
    }

    // FIXED: Expand hover bounds to cover the entire column width
    fn divider_hover_bounds(&self, bounds: Rectangle) -> Rectangle {
        Rectangle {
            x: bounds.x, // Start from the beginning of the column
            y: bounds.y,
            width: bounds.width, // Cover the entire column width
            height: bounds.height,
        }
    }

    fn is_content_hovered(&self, bounds: Rectangle, cursor: Cursor) -> bool {
        cursor.is_over(bounds) // Use full bounds for content hover
    }

    // Helper method to count visible columns
    fn count_visible_columns(&self) -> usize {
        1 + self.other_columns.iter().filter(|(_, _, visible)| *visible).count()
    }

    // Helper method to check if a column can be hidden
    fn can_hide_column(&self, column_id: &str) -> bool {
        let visible_count = self.count_visible_columns();
        
        // Don't allow hiding if it would result in 0 visible columns
        if visible_count <= 1 {
            return false;
        }

        // If hiding current column, check if others are visible
        if column_id == self.column_id {
            return self.other_columns.iter().any(|(_, _, visible)| *visible);
        }

        // If hiding another column, always allow if we have more than 1 visible
        true
    }

    fn context_menu_bounds(&self, position: Point) -> Rectangle {
        // Calculate menu size based on actual content
        let item_height = 30.0;
        let padding = 10.0;
        let separator_height = 6.0;
        
        // Count items: current column + separator (if other columns exist) + other columns
        let item_count = 1 + // current column
            if self.other_columns.is_empty() { 0 } else { 1 } + // separator
            self.other_columns.len(); // other columns
            
        let content_height = (item_count as f32 * item_height) + 
            if self.other_columns.is_empty() { 0.0 } else { separator_height };
            
        let menu_height = content_height + (padding * 2.0);
        
        // Calculate width based on longest text
        let current_title_width = self.column_title.len() as f32 * 8.0;
        let max_other_width = self.other_columns
            .iter()
            .map(|(_, title, _)| title.len() as f32 * 8.0)
            .fold(0.0, f32::max);
            
        let min_width = 180.0;
        let content_width = current_title_width.max(max_other_width).max(min_width);
        let menu_width = content_width + (padding * 2.0);
        
        Rectangle {
            x: position.x,
            y: position.y,
            width: menu_width,
            height: menu_height,
        }
    }

    // FIXED: Use theme colors instead of hardcoded values
    fn draw_context_menu(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: Cursor,
    ) where
        Renderer: iced::advanced::text::Renderer,
    {
        // Get theme colors - you may need to adjust these based on your theme implementation
        let appearance = theme.divider(&self.style, false);
        let hovered_appearance = theme.header(&self.style);
        let background_color = appearance.background.unwrap_or_else(|| Color::TRANSPARENT.into());
        let border_color = appearance.border.color;
        let text_color = hovered_appearance.text_color.unwrap_or_else(|| Color::from_rgb(0.2, 0.2, 0.2));
        let hover_color = hovered_appearance.background.unwrap_or_else(|| Color::TRANSPARENT.into());
        let separator_color = Color::scale_alpha(text_color, 0.6);
        let disabled_text_color = Color::scale_alpha(text_color, 0.6);
        
        

        // Draw menu background
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: iced::Border {
                    color: border_color,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                    offset: Vector::new(0.0, 4.0),
                    blur_radius: 12.0,
                },
            },
            background_color,
        );

        let mut y_offset = bounds.y + 8.0;
        let item_height = 30.0;
        let padding_x = 12.0;

        // Current column item - "Hide [Column]"
        let item_bounds = Rectangle {
            x: bounds.x,
            y: y_offset,
            width: bounds.width,
            height: item_height,
        };

        // Check if this column can be hidden
        let can_hide_current = self.can_hide_column(&self.column_id);
        let current_text_color = if can_hide_current { text_color } else { disabled_text_color };

        // Highlight on hover (only if clickable)
        if can_hide_current && cursor.is_over(item_bounds) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: item_bounds.x + 2.0,
                        y: item_bounds.y,
                        width: item_bounds.width - 4.0,
                        height: item_bounds.height,
                    },
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                },
                hover_color,
            );
        }

        // Draw text for current column
        let hide_text = format!("Hide {}", self.column_title);
        renderer.fill_text(
            iced::advanced::text::Text {
                content: hide_text,
                bounds: Size::new(item_bounds.width - padding_x * 2.0, item_height),
                size: iced::Pixels(14.0),
                line_height: iced::advanced::text::LineHeight::Relative(1.2),
                font: renderer.default_font(),
                align_x: iced::advanced::text::Alignment::Left,
                align_y: iced::alignment::Vertical::Center,
                wrapping: iced::advanced::text::Wrapping::Word,
                shaping: iced::advanced::text::Shaping::Basic,
            },
            Point::new(item_bounds.x + padding_x, item_bounds.y + 14.0),
            current_text_color,
            bounds,
        );

        y_offset += item_height;

        // Draw separator if there are other columns
        if !self.other_columns.is_empty() {
            let separator_y = y_offset + 2.0;
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: bounds.x + 8.0,
                        y: separator_y,
                        width: bounds.width - 16.0,
                        height: 1.0,
                    },
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                },
                separator_color,
            );
            y_offset += 6.0;
        }

        // Draw other columns with checkmarks
        for (column_id, title, visible) in &self.other_columns {
            let item_bounds = Rectangle {
                x: bounds.x,
                y: y_offset,
                width: bounds.width,
                height: item_height,
            };

            // Check if this column can be toggled
            let can_toggle = if *visible {
                self.can_hide_column(column_id)
            } else {
                true // Can always show hidden columns
            };

            let item_text_color = if can_toggle { text_color } else { disabled_text_color };

            // Draw hover highlight if clickable
            if can_toggle && cursor.is_over(item_bounds) {
                let hover_rect = Rectangle {
                    x: item_bounds.x + 2.0,
                    y: item_bounds.y,
                    width: item_bounds.width - 4.0,
                    height: item_bounds.height,
                };
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: hover_rect,
                        border: iced::Border::default(),
                        shadow: iced::Shadow::default(),
                    },
                    hover_color,
                );
            }

            // Compute checkbox position & draw the box
            let checkbox_size = 16.0;
            let checkbox_x = item_bounds.x + padding_x;  
            let checkbox_y = item_bounds.y + (item_bounds.height - checkbox_size) / 2.0;

            // Draw the checkbox
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: checkbox_x,
                        y: checkbox_y,
                        width: checkbox_size,
                        height: checkbox_size,
                    },
                    border: iced::Border {
                        color: if can_toggle { Color::from_rgb(0.6, 0.6, 0.6) } else { disabled_text_color },
                        width: 1.0,
                        radius: 3.0.into(),
                    },
                    shadow: iced::Shadow::default(),
                },
                if can_toggle { Color::from_rgb(0.2, 0.6, 1.0) } else { disabled_text_color }
            );

            // Draw checkmark if visible
            if *visible {
                renderer.fill_text(
                    iced::advanced::text::Text {
                        content: String::from("✓"),
                        bounds: Size::new(checkbox_size, checkbox_size),
                        size: iced::Pixels(12.0),
                        line_height: iced::advanced::text::LineHeight::Relative(1.0),
                        font: renderer.default_font(),
                        align_x: iced::advanced::text::Alignment::Center,
                        align_y: iced::alignment::Vertical::Center,
                        wrapping: iced::advanced::text::Wrapping::Word,
                        shaping: iced::advanced::text::Shaping::Advanced,
                    },
                    Point::new(checkbox_x + 8.0 , checkbox_y + 9.0),
                    Color::WHITE,
                    bounds,
                );
            }

            // Draw the column title
            let gap_between = 8.0;
            let text_x = checkbox_x + checkbox_size + gap_between;  
            let left_padding = text_x - bounds.x;     
            let text_clip_width = bounds.width - left_padding - padding_x;

            renderer.fill_text(
                iced::advanced::text::Text {
                    content: title.to_owned(),
                    bounds: Size::new(text_clip_width, item_height),
                    size: iced::Pixels(14.0),
                    line_height: iced::advanced::text::LineHeight::Relative(1.0),
                    font: renderer.default_font(),
                    align_x: iced::advanced::text::Alignment::Left,
                    align_y: iced::alignment::Vertical::Center,
                    wrapping: iced::advanced::text::Wrapping::Word,
                    shaping: iced::advanced::text::Shaping::Basic,
                },
                Point::new(text_x, item_bounds.y + 14.0),
                item_text_color,
                bounds,
            );

            y_offset += item_height;
        }
    }

    fn handle_context_menu_click(
        &self,
        cursor_position: Point,
        menu_bounds: Rectangle,
        shell: &mut Shell<'_, Message>,
    ) -> bool {
        if cursor_position.x < menu_bounds.x 
            || cursor_position.x >= menu_bounds.x + menu_bounds.width
            || cursor_position.y < menu_bounds.y 
            || cursor_position.y >= menu_bounds.y + menu_bounds.height {
            return false;
        }

        let relative_y = cursor_position.y - menu_bounds.y - 8.0;
        let item_height = 30.0;
        let separator_offset = if self.other_columns.is_empty() { 0.0 } else { 6.0 };
        
        if relative_y < item_height {
            // Clicked on current column - only allow if it can be hidden
            if self.can_hide_column(&self.column_id) {
                if let Some(on_column_visibility) = &self.on_column_visibility {
                    shell.publish((on_column_visibility)(ColumnVisibilityMessage::ToggleColumn(self.column_id.clone())));
                    return true;
                }
            }
        } else if !self.other_columns.is_empty() && relative_y > item_height + separator_offset {
            // Clicked on other column
            let other_column_y = relative_y - item_height - separator_offset;
            let other_index = (other_column_y / item_height) as usize;
            
            if let Some((id, _, visible)) = self.other_columns.get(other_index) {
                // Check if this action is allowed
                let can_toggle = if *visible {
                    self.can_hide_column(id)
                } else {
                    true // Can always show hidden columns
                };

                if can_toggle {
                    if let Some(on_column_visibility) = &self.on_column_visibility {
                        shell.publish((on_column_visibility)(ColumnVisibilityMessage::ToggleColumn(id.clone())));
                        return true;
                    }
                }
            }
        }

        false
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Divider<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: renderer::Renderer + iced::advanced::text::Renderer,
    Theme: style::Catalog,
{
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::default())
    }

    fn children(&self) -> Vec<widget::Tree> {
        vec![widget::Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(&[&self.content]);
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let padding = padding::all(0).right(self.width);

        layout::padded(limits, Length::Fill, Length::Shrink, padding, |limits| {
            self.content
                .as_widget()
                .layout(&mut tree.children[0], renderer, limits)
        })
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &event::Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let divider_hover_bounds = self.divider_hover_bounds(layout.bounds());

        // Always update hover state for smooth transitions
        state.is_divider_hovered = cursor.is_over(divider_hover_bounds);

        // Handle mouse events
        if let event::Event::Mouse(mouse_event) = event {
            match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    // Always close context menu on left click anywhere
                    if state.show_context_menu {
                        state.show_context_menu = false;
                        shell.invalidate_layout();
                        shell.invalidate_widgets();
                    }
                    
                    if let Some(origin) = cursor.position_over(divider_hover_bounds) {
                        state.drag_origin = Some(origin);
                        shell.invalidate_layout();
                        shell.invalidate_widgets();
                        return;
                    }
                }
                mouse::Event::ButtonPressed(mouse::Button::Right) => {
                    // Show context menu on right click (only if column visibility is enabled)
                    if self.on_column_visibility.is_some() && cursor.is_over(layout.bounds()) {
                        if let Some(position) = cursor.position() {
                            state.context_menu_position = position;
                            state.show_context_menu = true;
                            shell.invalidate_layout();
                            shell.invalidate_widgets();
                            return;
                        }
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    if state.drag_origin.take().is_some() {
                        shell.publish(self.on_release.clone());
                        return;
                    }
                }
                mouse::Event::CursorMoved { .. } => {
                    if let Some(position) = cursor.position() {
                        if let Some(origin) = state.drag_origin {
                            shell.publish((self.on_drag)((position - origin).x));
                            shell.invalidate_layout();
                            shell.invalidate_widgets();
                            return;
                        }
                        
                        // Force updates for hover state changes
                        let divider_hover_bounds = self.divider_hover_bounds(layout.bounds());
                        if cursor.is_over(divider_hover_bounds) || cursor.is_over(layout.bounds()) {
                            shell.invalidate_layout();
                            shell.invalidate_widgets();
                            return;
                        }
                    }
                }
                _ => {}
            }
        }

        // Always delegate to content for normal events
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();

        if state.drag_origin.is_some() || state.is_divider_hovered {
            mouse::Interaction::ResizingHorizontally
        } else {
            self.content.as_widget().mouse_interaction(
                &tree.children[0],
                layout.children().next().unwrap(),
                cursor,
                viewport,
                renderer,
            )
        }
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );

        // FIXED: Show divider based on always_show_divider flag or hover state
        let should_show_divider = self.always_show_divider || 
            self.is_content_hovered(layout.bounds(), cursor) ||
            state.is_divider_hovered ||
            state.drag_origin.is_some();

        if should_show_divider {
            let appearance = theme.divider(
                &self.style,
                state.is_divider_hovered || state.drag_origin.is_some(),
            );

            let snap = |bounds: Rectangle| {
                let position = bounds.position();

                Rectangle {
                    x: position.x.floor(),
                    y: position.y.floor(),
                    width: self.width,
                    ..bounds
                }
            };

            renderer.fill_quad(
                renderer::Quad {
                    bounds: snap(self.divider_bounds(layout.bounds())),
                    border: appearance.border,
                    shadow: Default::default(),
                },
                appearance
                    .background
                    .unwrap_or_else(|| Color::TRANSPARENT.into()),
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        viewport: &iced::Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'_, Message, Theme, Renderer>> {
        let state = tree.state.downcast_ref::<State>();

        // Only create overlay if THIS specific divider has the context menu open
        if state.show_context_menu {
            let menu_overlay = ContextMenuOverlay {
                divider: self,
                position: state.context_menu_position + translation,
                tree,
            };

            Some(overlay::Element::new(Box::new(menu_overlay)))
        } else {
            // Always delegate to content's overlay
            self.content.as_widget_mut().overlay(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                viewport,
                translation,
            )
        }
    }

    fn operate(
        &self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content.as_widget().operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
    }
}

impl<'a, Message, Theme, Renderer> From<Divider<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: renderer::Renderer + iced::advanced::text::Renderer + 'a,
    Theme: style::Catalog + 'a,
{
    fn from(divider: Divider<'a, Message, Theme, Renderer>) -> Self {
        Element::new(divider)
    }
}

// Context menu overlay implementation
struct ContextMenuOverlay<'a, 'b, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + iced::advanced::text::Renderer,
    Theme: style::Catalog,
{
    divider: &'a Divider<'a, Message, Theme, Renderer>,
    position: Point,
    tree: &'b mut widget::Tree,
}

impl<'a, 'b, Message, Theme, Renderer> Overlay<Message, Theme, Renderer>
    for ContextMenuOverlay<'a, 'b, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: renderer::Renderer + iced::advanced::text::Renderer,
    Theme: style::Catalog,
{
    fn layout(&mut self, _renderer: &Renderer, bounds: Size) -> layout::Node {
        let menu_bounds = self.divider.context_menu_bounds(self.position);
        
        // Ensure menu doesn't go off screen
        let adjusted_x = if menu_bounds.x + menu_bounds.width > bounds.width {
            bounds.width - menu_bounds.width
        } else {
            menu_bounds.x
        }.max(0.0);
        
        let adjusted_y = if menu_bounds.y + menu_bounds.height > bounds.height {
            bounds.height - menu_bounds.height
        } else {
            menu_bounds.y
        }.max(0.0);

        layout::Node::new(Size::new(menu_bounds.width, menu_bounds.height))
            .move_to(Point::new(adjusted_x, adjusted_y))
    }

    fn update(
        &mut self,
        event: &iced::Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let menu_bounds = layout.bounds();

        match &event {
            iced::Event::Mouse(mouse_event) => {
                match mouse_event {
                    mouse::Event::ButtonPressed(mouse::Button::Left) => {
                        if let Some(cursor_pos) = cursor.position() {
                            if cursor.is_over(menu_bounds) {
                                // Handle click inside menu
                                if self.divider.handle_context_menu_click(cursor_pos, menu_bounds, shell) {
                                    // Close menu after successful click
                                    let state = self.tree.state.downcast_mut::<State>();
                                    state.show_context_menu = false;
                                }
                                shell.invalidate_layout();
                                shell.invalidate_widgets();
                                shell.capture_event();
                                return;
                            } else {
                                // Clicked outside menu, close it
                                let state = self.tree.state.downcast_mut::<State>();
                                state.show_context_menu = false;
                                shell.capture_event();
                                return;
                            }
                        }
                    }
                    mouse::Event::ButtonPressed(mouse::Button::Right) => {
                        // Close on right click
                        let state = self.tree.state.downcast_mut::<State>();
                        state.show_context_menu = false;
                        shell.invalidate_layout();
                        shell.invalidate_widgets();
                        shell.capture_event();
                        return;
                    }
                    mouse::Event::CursorMoved { .. } => {
                        // Always capture mouse moves for hover updates
                        shell.invalidate_layout();
                        shell.invalidate_widgets();
                        return;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
    ) {
        self.divider.draw_context_menu(renderer, theme, layout.bounds(), cursor);
    }

    fn is_over(&self, layout: Layout<'_>, _renderer: &Renderer, cursor_position: Point) -> bool {
        layout.bounds().contains(cursor_position)
    }
}