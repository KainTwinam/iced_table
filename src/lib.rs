//! A table widget for iced
#![deny(missing_debug_implementations, missing_docs)]
pub use style::Catalog;
pub use table::{table, Table};

mod divider;
mod style;

pub use divider::ColumnVisibilityMessage;

pub mod table {
    //! Display rows of data into columns
    use iced::{Element, Length, Padding};
    use iced::widget::{column, container, row, scrollable, Space};

    use super::divider::Divider;
    use super::style;

    /// Creates a new [`Table`] with the provided [`Column`] definitions
    /// and [`Row`](Column::Row) data.
    ///
    /// `on_sync` is needed to keep the header & footer scrollables in sync with
    /// the body scrollable. It is up to the consumer to emit a [`scroll_to`](iced_widget::scrollable::scroll_to) operation
    /// from `update` when this message is received.
    pub fn table<'a, Column, Row, Message, Theme>(
        header: scrollable::Id,
        body: scrollable::Id,
        columns: &'a [Column],
        rows: &'a [Row],
        on_sync: fn(scrollable::AbsoluteOffset) -> Message,
    ) -> Table<'a, Column, Row, Message, Theme>
    where
        Theme: style::Catalog + container::Catalog,
    {
        Table {
            header,
            body,
            footer: None,
            columns,
            rows,
            on_sync,
            on_column_drag: None,
            on_column_release: None,
            on_column_visibility: None,
            min_width: 0.0,
            min_column_width: 4.0,
            divider_width: 2.0,
            cell_padding: 4.into(),
            style: Default::default(),
            scrollbar: scrollable::Scrollbar::default(),
        }
    }

    /// Defines what a column looks like for each [`Row`](Self::Row) of data.
    pub trait Column<'a, Message, Theme, Renderer> {
        /// A row of data.
        type Row;

        /// Define the header [`Element`] for this column.
        fn header(&'a self, col_index: usize) -> Element<'a, Message, Theme, Renderer>;

        /// Define the cell [`Element`] for this column.
        fn cell(
            &'a self,
            col_index: usize,
            row_index: usize,
            row: &'a Self::Row,
        ) -> Element<'a, Message, Theme, Renderer>;

        /// Define the footer [`Element`] for this column.
        fn footer(
            &'a self,
            _col_index: usize,
            _rows: &'a [Self::Row],
        ) -> Option<Element<'a, Message, Theme, Renderer>> {
            None
        }

        /// Return the fixed width for this column.
        fn width(&self) -> f32;

        /// Return the offset of an on-going resize of this column.
        fn resize_offset(&self) -> Option<f32>;

        /// Return the unique identifier for this column (used for visibility tracking).
        fn id(&self) -> String {
            // Default implementation - users should override this with meaningful IDs
            format!("column_{}", std::any::type_name::<Self>())
        }

        /// Return the display title for this column (used in context menu).
        fn title(&self) -> String {
            // Default implementation - users should override this with meaningful titles
            format!("Column")
        }

        /// Return whether this column is currently visible.
        fn is_visible(&self) -> bool {
            true
        }
    }

    /// An element to display rows of data into columns.
    #[allow(missing_debug_implementations)]
    pub struct Table<'a, Column, Row, Message, Theme>
    where
        Theme: style::Catalog + container::Catalog,
    {
        header: scrollable::Id,
        body: scrollable::Id,
        footer: Option<scrollable::Id>,
        columns: &'a [Column],
        rows: &'a [Row],
        on_sync: fn(scrollable::AbsoluteOffset) -> Message,
        on_column_drag: Option<fn(usize, f32) -> Message>,
        on_column_release: Option<Message>,
        on_column_visibility: Option<fn(super::divider::ColumnVisibilityMessage) -> Message>,
        min_width: f32,
        min_column_width: f32,
        divider_width: f32,
        cell_padding: Padding,
        style: <Theme as style::Catalog>::Style,
        scrollbar: scrollable::Scrollbar,
    }

    impl<'a, Column, Row, Message, Theme> Table<'a, Column, Row, Message, Theme>
    where
        Theme: style::Catalog + container::Catalog,
    {
        /// Sets the message that will be produced when a [`Column`] is resizing. Setting this
        /// will enable the resizing interaction.
        ///
        /// `on_drag` will emit a message during an on-going resize. It is up to the consumer to return
        /// this value for the associated column in [`Column::resize_offset`].
        ///
        /// `on_release` is emited when the resize is finished. It is up to the consumer to apply the last
        /// `on_drag` offset to the column's stored width.
        pub fn on_column_resize(
            self,
            on_drag: fn(usize, f32) -> Message,
            on_release: Message,
        ) -> Self {
            Self {
                on_column_drag: Some(on_drag),
                on_column_release: Some(on_release),
                ..self
            }
        }

        /// Sets the message that will be produced when column visibility is changed.
        /// This enables the right-click context menu for hiding/showing columns.
        pub fn on_column_visibility(
            self,
            on_visibility: fn(super::divider::ColumnVisibilityMessage) -> Message,
        ) -> Self {
            Self {
                on_column_visibility: Some(on_visibility),
                ..self
            }
        }

        /// Show the footer returned by [`Column::footer`].
        pub fn footer(self, footer: scrollable::Id) -> Self {
            Self {
                footer: Some(footer),
                ..self
            }
        }

        /// Sets the minimum width of table.
        ///
        /// This is useful to use in conjuction with [`responsive`](iced_widget::responsive) to ensure
        /// the table always fills the width of it's parent container.
        pub fn min_width(self, min_width: f32) -> Self {
            Self { min_width, ..self }
        }

        /// Sets the minimum width a column can be resized to.
        pub fn min_column_width(self, min_column_width: f32) -> Self {
            Self {
                min_column_width,
                ..self
            }
        }

        /// Sets the width of the column dividers.
        pub fn divider_width(self, divider_width: f32) -> Self {
            Self {
                divider_width,
                ..self
            }
        }

        /// Sets the [`Padding`] used inside each cell of the [`Table`].
        pub fn cell_padding(self, cell_padding: impl Into<Padding>) -> Self {
            Self {
                cell_padding: cell_padding.into(),
                ..self
            }
        }

        /// Sets the style variant of this [`Table`].
        pub fn style(self, style: impl Into<<Theme as style::Catalog>::Style>) -> Self {
            Self {
                style: style.into(),
                ..self
            }
        }

        ///  Sets the [`Scrollbar`](iced_widget::scrollable::Scrollbar) used for the table's body scrollable.
        pub fn scrollbar(self, scrollbar: scrollable::Scrollbar) -> Self {
            Self { scrollbar, ..self }
        }
    }

    impl<'a, Column, Row, Message, Theme, Renderer> From<Table<'a, Column, Row, Message, Theme>>
        for Element<'a, Message, Theme, Renderer>
    where
        Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer + 'a,
        Theme: style::Catalog + container::Catalog + scrollable::Catalog + 'a,
        Column: self::Column<'a, Message, Theme, Renderer, Row = Row>,
        Message: 'a + Clone,
    {
        fn from(table: Table<'a, Column, Row, Message, Theme>) -> Self {
            let Table {
                header,
                body,
                footer,
                columns,
                rows,
                on_sync,
                on_column_drag,
                on_column_release,
                on_column_visibility,
                min_width,
                min_column_width,
                divider_width,
                cell_padding,
                style,
                scrollbar,
            } = table;

            let header = scrollable(style::wrapper::header(
                row(columns
                    .iter()
                    .enumerate()
                    .filter(|(_, column)| column.is_visible())
                    .map(|(index, column)| {
                        header_container(
                            index,
                            column,
                            columns,
                            on_column_drag,
                            on_column_release.clone(),
                            on_column_visibility.clone(),
                            min_column_width,
                            divider_width,
                            cell_padding,
                            style.clone(),
                        )
                    })
                    .chain(dummy_container(columns, min_width, min_column_width))),
                style.clone(),
            ))
            .id(header)
            .direction(scrollable::Direction::Both {
                vertical: scrollable::Scrollbar::new()
                    .width(0)
                    .margin(0)
                    .scroller_width(0),
                horizontal: scrollable::Scrollbar::new()
                    .width(0)
                    .margin(0)
                    .scroller_width(0),
            });

            let body = scrollable(column(rows.iter().enumerate().map(|(row_index, _row)| {
                style::wrapper::row(
                    row(columns
                        .iter()
                        .enumerate()
                        .filter(|(_, column)| column.is_visible())
                        .map(|(col_index, column)| {
                            body_container(
                                col_index,
                                row_index,
                                column,
                                _row,
                                min_column_width,
                                divider_width,
                                cell_padding,
                            )
                        })
                        .chain(dummy_container(columns, min_width, min_column_width))),
                    style.clone(),
                    row_index,
                )
                .into()
            })))
            .id(body)
            .on_scroll(move |viewport| {
                let offset = viewport.absolute_offset();

                (on_sync)(scrollable::AbsoluteOffset { y: 0.0, ..offset })
            })
            .direction(scrollable::Direction::Both {
                horizontal: scrollbar,
                vertical: scrollbar,
            })
            .height(Length::Fill);

            let footer = footer.map(|footer| {
                scrollable(style::wrapper::footer(
                    row(columns
                        .iter()
                        .enumerate()
                        .filter(|(_, column)| column.is_visible())
                        .map(|(index, column)| {
                            footer_container(
                                index,
                                column,
                                columns,
                                rows,
                                on_column_drag,
                                on_column_release.clone(),
                                on_column_visibility.clone(),
                                min_column_width,
                                divider_width,
                                cell_padding,
                                style.clone(),
                            )
                        })
                        .chain(dummy_container(columns, min_width, min_column_width))),
                    style,
                ))
                .id(footer)
                .direction(scrollable::Direction::Both {
                    vertical: scrollable::Scrollbar::new()
                        .width(0)
                        .margin(0)
                        .scroller_width(0),
                    horizontal: scrollable::Scrollbar::new()
                        .width(0)
                        .margin(0)
                        .scroller_width(0),
                })
            });

            let mut column = column![header, body];

            if let Some(footer) = footer {
                column = column.push(footer);
            }

            column.height(Length::Fill).into()
        }
    }

    fn header_container<'a, Column, Row, Message, Theme, Renderer>(
        index: usize,
        column: &'a Column,
        all_columns: &'a [Column],
        on_drag: Option<fn(usize, f32) -> Message>,
        on_release: Option<Message>,
        on_column_visibility: Option<fn(super::divider::ColumnVisibilityMessage) -> Message>,
        min_column_width: f32,
        divider_width: f32,
        cell_padding: Padding,
        style: <Theme as style::Catalog>::Style,
    ) -> Element<'a, Message, Theme, Renderer>
    where
        Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer + 'a,
        Theme: style::Catalog + container::Catalog + 'a,
        Column: self::Column<'a, Message, Theme, Renderer, Row = Row>,
        Message: 'a + Clone,
    {
        let content = container(column.header(index))
            .width(Length::Fill)
            .padding(cell_padding)
            .into();

        with_divider(
            index,
            column,
            all_columns,
            content,
            on_drag,
            on_release,
            on_column_visibility,
            min_column_width,
            divider_width,
            style,
        )
    }

    fn body_container<'a, Column, Row, Message, Theme, Renderer>(
        col_index: usize,
        row_index: usize,
        column: &'a Column,
        row: &'a Row,
        min_column_width: f32,
        divider_width: f32,
        cell_padding: Padding,
    ) -> Element<'a, Message, Theme, Renderer>
    where
        Renderer: iced::advanced::Renderer + 'a,
        Theme: style::Catalog + container::Catalog + 'a,
        Column: self::Column<'a, Message, Theme, Renderer, Row = Row>,
        Message: 'a + Clone,
    {
        let width = column.width() + column.resize_offset().unwrap_or_default();

        let content = container(column.cell(col_index, row_index, row))
            .width(Length::Fill)
            .padding(cell_padding);

        let spacing = Space::new(divider_width, Length::Shrink);

        row![content, spacing]
            .width(width.max(min_column_width))
            .into()
    }

    fn footer_container<'a, Column, Row, Message, Theme, Renderer>(
        index: usize,
        column: &'a Column,
        all_columns: &'a [Column],
        rows: &'a [Row],
        on_drag: Option<fn(usize, f32) -> Message>,
        on_release: Option<Message>,
        on_column_visibility: Option<fn(super::divider::ColumnVisibilityMessage) -> Message>,
        min_column_width: f32,
        divider_width: f32,
        cell_padding: Padding,
        style: <Theme as style::Catalog>::Style,
    ) -> Element<'a, Message, Theme, Renderer>
    where
        Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer + 'a,
        Theme: style::Catalog + container::Catalog + 'a,
        Column: self::Column<'a, Message, Theme, Renderer, Row = Row>,
        Message: 'a + Clone,
    {
        let content = if let Some(footer) = column.footer(index, rows) {
            container(footer)
                .width(Length::Fill)
                .padding(cell_padding)
                .into()
        } else {
            Element::from(Space::with_width(Length::Fill))
        };

        with_divider(
            index,
            column,
            all_columns,
            content,
            on_drag,
            on_release,
            on_column_visibility,
            min_column_width,
            divider_width,
            style,
        )
    }

    fn with_divider<'a, Column, Row, Message, Theme, Renderer>(
        index: usize,
        column: &'a Column,
        all_columns: &'a [Column],
        content: Element<'a, Message, Theme, Renderer>,
        on_drag: Option<fn(usize, f32) -> Message>,
        on_release: Option<Message>,
        on_column_visibility: Option<fn(super::divider::ColumnVisibilityMessage) -> Message>,
        min_column_width: f32,
        divider_width: f32,
        style: <Theme as style::Catalog>::Style,
    ) -> Element<'a, Message, Theme, Renderer>
    where
        Renderer: iced::advanced::Renderer + iced::advanced::text::Renderer + 'a,
        Theme: style::Catalog + container::Catalog + 'a,
        Column: self::Column<'a, Message, Theme, Renderer, Row = Row>,
        Message: 'a + Clone,
    {
        let width =
            (column.width() + column.resize_offset().unwrap_or_default()).max(min_column_width);

        if let Some((on_drag, on_release)) = on_drag.zip(on_release) {
            let old_width = column.width();

            let mut divider = Divider::new(
                content,
                divider_width,
                column.id(),
                column.title(),
                move |offset| {
                    let new_width = (old_width + offset).max(min_column_width);
                    (on_drag)(index, new_width - old_width)
                },
                on_release,
                style,
            );

            // Add column visibility if enabled
            if let Some(on_visibility) = on_column_visibility {
                let other_columns: Vec<(String, String, bool)> = all_columns
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != index)
                    .map(|(_, col)| (col.id(), col.title(), col.is_visible()))
                    .collect();

                divider = divider.with_column_visibility(on_visibility, other_columns);
            }

            container(divider)
                .width(width)
                .into()
        } else {
            row![content, Space::new(divider_width, Length::Shrink)]
                .width(width)
                .into()
        }
    }

    // Used to enforce "min_width"
    fn dummy_container<'a, Column, Row, Message, Theme, Renderer>(
        columns: &'a [Column],
        min_width: f32,
        min_column_width: f32,
    ) -> Option<Element<'a, Message, Theme, Renderer>>
    where
        Renderer: iced::advanced::Renderer + 'a,
        Theme: style::Catalog + container::Catalog + 'a,
        Column: self::Column<'a, Message, Theme, Renderer, Row = Row>,
        Message: 'a + Clone,
    {
        let total_width: f32 = columns
            .iter()
            .map(|column| {
                (column.width() + column.resize_offset().unwrap_or_default()).max(min_column_width)
            })
            .sum();

        let remaining = min_width - total_width;

        (remaining > 0.0).then(|| container(Space::with_width(remaining)).into())
    }
}
