use std::fmt;
use std::collections::HashMap;

use iced::widget::{
    button, checkbox, column, container, horizontal_space, pick_list, responsive, scrollable, text,
    text_input,
};
use iced::{Element, Length, Renderer, Task, Theme};
use iced_table::{table, ColumnVisibilityMessage};

fn main() {
    iced::application(App::new, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .run()
        .unwrap()
}

#[derive(Debug, Clone)]
enum Message {
    SyncHeader(scrollable::AbsoluteOffset),
    Resizing(usize, f32),
    Resized,
    ResizeColumnsEnabled(bool),
    FooterEnabled(bool),
    MinWidthEnabled(bool),
    DarkThemeEnabled(bool),
    ColumnVisibilityEnabled(bool),
    Notes(usize, String),
    Category(usize, Category),
    Enabled(usize, bool),
    Delete(usize),
    ColumnVisibility(ColumnVisibilityMessage),
}

struct App {
    columns: Vec<Column>,
    rows: Vec<Row>,
    header: scrollable::Id,
    body: scrollable::Id,
    footer: scrollable::Id,
    resize_columns_enabled: bool,
    footer_enabled: bool,
    min_width_enabled: bool,
    column_visibility_enabled: bool,
    column_visibility: HashMap<String, bool>,
    theme: Theme,
}

impl Default for App {
    fn default() -> Self {
        let mut column_visibility = HashMap::new();
        column_visibility.insert("index".to_string(), true);
        column_visibility.insert("category".to_string(), true);
        column_visibility.insert("enabled".to_string(), true);
        column_visibility.insert("notes".to_string(), true);
        column_visibility.insert("delete".to_string(), false); // Hidden by default

        Self {
            columns: vec![
                Column::new(ColumnKind::Index),
                Column::new(ColumnKind::Category),
                Column::new(ColumnKind::Enabled),
                Column::new(ColumnKind::Notes),
                Column::new(ColumnKind::Delete),
            ],
            rows: (0..50).map(Row::generate).collect(),
            header: scrollable::Id::unique(),
            body: scrollable::Id::unique(),
            footer: scrollable::Id::unique(),
            resize_columns_enabled: true,
            footer_enabled: true,
            min_width_enabled: true,
            column_visibility_enabled: true,
            column_visibility,
            theme: Theme::Light,
        }
    }
}

impl App {
    fn title(&self) -> String {
        "Iced Table - Column Visibility Demo".into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn new() -> Self { 
        App::default() 
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SyncHeader(offset) => {
                return Task::batch(vec![
                    scrollable::scroll_to(self.header.clone(), offset),
                    scrollable::scroll_to(self.footer.clone(), offset),
                ])
            }
            Message::Resizing(index, offset) => {
                if let Some(column) = self.columns.get_mut(index) {
                    column.resize_offset = Some(offset);
                }
            }
            Message::Resized => self.columns.iter_mut().for_each(|column| {
                if let Some(offset) = column.resize_offset.take() {
                    column.width += offset;
                }
            }),
            Message::ResizeColumnsEnabled(enabled) => self.resize_columns_enabled = enabled,
            Message::FooterEnabled(enabled) => self.footer_enabled = enabled,
            Message::MinWidthEnabled(enabled) => self.min_width_enabled = enabled,
            Message::ColumnVisibilityEnabled(enabled) => self.column_visibility_enabled = enabled,
            Message::DarkThemeEnabled(enabled) => {
                if enabled {
                    self.theme = Theme::Dark;
                } else {
                    self.theme = Theme::Light;
                }
            }
            Message::Category(index, category) => {
                if let Some(row) = self.rows.get_mut(index) {
                    row.category = category;
                }
            }
            Message::Enabled(index, is_enabled) => {
                if let Some(row) = self.rows.get_mut(index) {
                    row.is_enabled = is_enabled;
                }
            }
            Message::Notes(index, notes) => {
                if let Some(row) = self.rows.get_mut(index) {
                    row.notes = notes;
                }
            }
            Message::Delete(index) => {
                self.rows.remove(index);
            }
            Message::ColumnVisibility(visibility_msg) => {
                match visibility_msg {
                    ColumnVisibilityMessage::ToggleColumn(column_id) => {
                        if let Some(visible) = self.column_visibility.get_mut(&column_id) {
                            *visible = !*visible;
                            
                            // Update the corresponding column
                            if let Some(column) = self.columns.iter_mut().find(|c| c.id() == column_id) {
                                column.visible = *visible;
                            }
                        }
                    }
                    ColumnVisibilityMessage::HideContextMenu => {
                        // Context menu was closed, no action needed
                        // This could be used to do cleanup if needed
                    }
                }
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let table = responsive(|size| {
            let mut table = table(
                self.header.clone(),
                self.body.clone(),
                &self.columns,
                &self.rows,
                Message::SyncHeader,
            );

            if self.resize_columns_enabled {
                table = table.on_column_resize(Message::Resizing, Message::Resized);
            }
            if self.footer_enabled {
                table = table.footer(self.footer.clone());
            }
            if self.min_width_enabled {
                table = table.min_width(size.width);
            }
            if self.column_visibility_enabled {
                table = table.on_column_visibility(Message::ColumnVisibility);
            }

            table.into()
        });

        let visible_columns_count = self.columns.iter().filter(|c| c.visible).count();

        let content = column![
            text("Table Features:").size(16),
            checkbox("Resize Columns", self.resize_columns_enabled,)
                .on_toggle(Message::ResizeColumnsEnabled),
            checkbox("Footer", self.footer_enabled,).on_toggle(Message::FooterEnabled),
            checkbox("Min Width", self.min_width_enabled,).on_toggle(Message::MinWidthEnabled),
            checkbox("Column Visibility (Right-click headers)", self.column_visibility_enabled,)
                .on_toggle(Message::ColumnVisibilityEnabled),
            checkbox("Dark Theme", matches!(self.theme, Theme::Dark),)
                .on_toggle(Message::DarkThemeEnabled),
            text(format!("Visible columns: {}/{}", visible_columns_count, self.columns.len())).size(14),
            if self.column_visibility_enabled {
                text("💡 Right-click on column headers to show/hide columns!").size(12)
            } else {
                text("Enable column visibility to access the context menu").size(12)
            },
            table,
        ]
        .spacing(6);

        container(container(content).width(Length::Fill).height(Length::Fill))
            .padding(20)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}

struct Column {
    kind: ColumnKind,
    width: f32,
    resize_offset: Option<f32>,
    visible: bool,
}

impl Column {
    fn new(kind: ColumnKind) -> Self {
        let width = match kind {
            ColumnKind::Index => 60.0,
            ColumnKind::Category => 100.0,
            ColumnKind::Enabled => 155.0,
            ColumnKind::Notes => 400.0,
            ColumnKind::Delete => 100.0,
        };

        let visible = match kind {
            ColumnKind::Delete => false, // Hidden by default
            _ => true,
        };

        Self {
            kind,
            width,
            resize_offset: None,
            visible,
        }
    }

    fn id(&self) -> &'static str {
        match self.kind {
            ColumnKind::Index => "index",
            ColumnKind::Category => "category",
            ColumnKind::Enabled => "enabled",
            ColumnKind::Notes => "notes",
            ColumnKind::Delete => "delete",
        }
    }

    fn display_name(&self) -> &'static str {
        match self.kind {
            ColumnKind::Index => "Index",
            ColumnKind::Category => "Category", 
            ColumnKind::Enabled => "Enabled",
            ColumnKind::Notes => "Notes",
            ColumnKind::Delete => "Delete",
        }
    }
}

#[derive(Clone, Copy)]
enum ColumnKind {
    Index,
    Category,
    Enabled,
    Notes,
    Delete,
}

struct Row {
    notes: String,
    category: Category,
    is_enabled: bool,
}

impl Row {
    fn generate(index: usize) -> Self {
        let category = match index % 5 {
            0 => Category::A,
            1 => Category::B,
            2 => Category::C,
            3 => Category::D,
            4 => Category::E,
            _ => unreachable!(),
        };
        let is_enabled = index % 5 < 4;

        Self {
            notes: String::new(),
            category,
            is_enabled,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Category {
    A,
    B,
    C,
    D,
    E,
}

impl Category {
    const ALL: &'static [Self] = &[Self::A, Self::B, Self::C, Self::D, Self::E];
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Category::A => "A",
            Category::B => "B",
            Category::C => "C",
            Category::D => "D",
            Category::E => "E",
        }
        .fmt(f)
    }
}

impl<'a> table::Column<'a, Message, Theme, Renderer> for Column {
    type Row = Row;

    fn header(&'a self, _col_index: usize) -> Element<'a, Message> {
        let content = self.display_name();
        container(text(content)).center_y(24).into()
    }

    fn cell(&'a self, _col_index: usize, row_index: usize, row: &'a Row) -> Element<'a, Message> {
        let content: Element<_> = match self.kind {
            ColumnKind::Index => text(row_index).into(),
            ColumnKind::Category => pick_list(Category::ALL, Some(row.category), move |category| {
                Message::Category(row_index, category)
            })
            .into(),
            ColumnKind::Enabled => checkbox("", row.is_enabled)
                .on_toggle(move |enabled| Message::Enabled(row_index, enabled))
                .into(),
            ColumnKind::Notes => text_input("", &row.notes)
                .on_input(move |notes| Message::Notes(row_index, notes))
                .width(Length::Fill)
                .into(),
            ColumnKind::Delete => button(text("Delete"))
                .on_press(Message::Delete(row_index))
                .into(),
        };

        container(content).width(Length::Fill).center_y(32).into()
    }

    fn footer(&'a self, _col_index: usize, rows: &'a [Row]) -> Option<Element<'a, Message>> {
        let content = if matches!(self.kind, ColumnKind::Enabled) {
            let total_enabled = rows.iter().filter(|row| row.is_enabled).count();
            Element::from(text(format!("Total Enabled: {total_enabled}")))
        } else {
            horizontal_space().into()
        };

        Some(container(content).center_y(24).into())
    }

    fn width(&self) -> f32 {
        self.width
    }

    fn resize_offset(&self) -> Option<f32> {
        self.resize_offset
    }

    // Implement the new trait methods for column visibility
    fn id(&self) -> String {
        Column::id(self).to_string()
    }

    fn title(&self) -> String {
        self.display_name().to_string()
    }

    fn is_visible(&self) -> bool {
        self.visible
    }
}