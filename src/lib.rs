#![doc = include_str!("../README.md")]

use std::rc::Rc;

use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Widget};

enum FieldStatus {
    None,
    Valid,
    Invalid,
}

pub struct Field<'a> {
    fd: FieldData<'a>,
    status: FieldStatus,
}

// pass me to Form rather than tuples
impl<'a> Field<'a> {
    pub fn new(name: &'a str, val: &'a str) -> Field<'a> {
        Self {
            fd: FieldData { name, val },
            status: FieldStatus::None,
        }
    }

    fn inner(&self) -> &FieldData<'_> {
        &self.fd
    }

    fn valid(&self) -> bool {
        match self.status {
            FieldStatus::Valid | FieldStatus::None => true,
            FieldStatus::Invalid => false,
        }
    }
}

#[derive(Clone)]
struct FieldData<'a> {
    name: &'a str,
    val: &'a str,
}

type FormFieldStatus<'a> = Vec<Field<'a>>;
pub enum FormSelection {
    NoSelection,
    Hovered(usize),
    Active(usize),
}

pub struct FormContext {
    selected: FormSelection,
}

impl FormContext {
    pub fn new() -> Self {
        Self {
            selected: FormSelection::Hovered(1),
        }
    }

    pub fn select(&mut self, selection: FormSelection) {
        self.selected = selection;
    }

    pub fn selected(&self) -> &FormSelection {
        &self.selected
    }
}

/// A widget to display data in a collection of [`Field`]s
pub struct Form<'a> {
    context: &'a FormContext,
    fields: FormFieldStatus<'a>,
    submitted: bool,
    default_field_style: Style,
    invalid_field_style: Style,
    hovered_field_style: Style,
    active_field_style: Style,
}

impl<'a> Form<'a> {
    pub fn new(context: &'a FormContext) -> Self {
        Self {
            context,
            fields: vec![],
            submitted: false,
            default_field_style: Style::default(),
            invalid_field_style: Style::default().red().bold(),
            hovered_field_style: Style::default().cyan(),
            active_field_style: Style::default().cyan().bold(),
        }
    }

    pub fn fields(mut self, fields: &'a [Field]) -> Self {
        self.fields = fields
            .iter()
            .map(|f| Field::new(f.inner().name, f.inner().val))
            .collect();
        self
    }

    /// Set whether the Form has been submitted
    pub fn submitted(mut self, submitted: bool) -> Self {
        self.submitted = submitted;
        self
    }

    /// Check fields based on `validation_fn`
    /// Fields for which `validation_fn` returns false will be rendered differently when the Form
    /// is submitted
    pub fn validate<F>(mut self, validation_fn: F) -> Self
    where
        F: Fn(&str) -> bool + 'static,
    {
        let field_validated = self
            .fields
            .iter()
            .map(|f| {
                let status = if validation_fn(f.inner().val) {
                    FieldStatus::Valid
                } else {
                    FieldStatus::Invalid
                };

                Field {
                    fd: f.fd.clone(),
                    status,
                }
            })
            .collect::<Vec<_>>();

        self.fields = field_validated;
        self
    }

    pub fn active_field_style(mut self, style: Style) -> Self {
        self.active_field_style = style;
        self
    }

    pub fn invalid_field_style(mut self, style: Style) -> Self {
        self.invalid_field_style = style;
        self
    }

    pub fn hovered_field_style(mut self, style: Style) -> Self {
        self.hovered_field_style = style;
        self
    }

    pub fn default_field_style(mut self, style: Style) -> Self {
        self.default_field_style = style;
        self
    }
}

impl Widget for Form<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Block::new().title("Form").render(area, buf);
        let constraints: Vec<Constraint> = self
            .fields
            .iter()
            .map(|_| Constraint::Max(3))
            .chain([Constraint::Max(1)])
            .collect();

        Block::default()
            .borders(Borders::ALL)
            .title("Form")
            .render(area, buf);

        let area = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(constraints)
            .split(area);

        self.render_fields(area, buf);
    }
}

enum FieldRenderType {
    Normal,
    Invalid,
    Hovered,
    Active,
}
impl Form<'_> {
    fn render_fields(&self, area: Rc<[Rect]>, buf: &mut Buffer) {
        self.fields.iter().enumerate().for_each(|(i, field)| {
            let is_invalid = !field.valid() && self.submitted;
            let hovered = if let FormSelection::Hovered(f) = self.context.selected {
                f == i
            } else {
                false
            };

            let active = if let FormSelection::Active(f) = self.context.selected {
                f == i
            } else {
                false
            };

            let render_type = match (hovered, active, is_invalid) {
                (_, true, _) => FieldRenderType::Active,
                (true, false, _) => FieldRenderType::Hovered,
                (false, false, true) => FieldRenderType::Invalid,
                (false, false, false) => FieldRenderType::Normal,
            };
            self.render_field_gen(
                area[i],
                buf,
                field.inner().val,
                Some(field.inner().name),
                render_type,
            );
        })
    }

    fn render_field_gen(
        &self,
        area: Rect,
        buf: &mut Buffer,
        content: &str,
        title: Option<&str>,
        fr: FieldRenderType,
    ) {
        match fr {
            FieldRenderType::Normal => self.render_field(area, buf, content, title),
            FieldRenderType::Invalid => self.render_field_invalid(area, buf, content, title),
            FieldRenderType::Hovered => self.render_field_hovered(area, buf, content, title),
            FieldRenderType::Active => self.render_field_active(area, buf, content, title),
        }
    }

    fn render_field(&self, area: Rect, buf: &mut Buffer, content: &str, title: Option<&str>) {
        Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(match title {
                        Some(t) => t,
                        None => "",
                    }),
            )
            .render(area, buf)
    }

    fn render_field_hovered(
        &self,
        area: Rect,
        buf: &mut Buffer,
        content: &str,
        title: Option<&str>,
    ) {
        Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.hovered_field_style)
                    .border_type(BorderType::Rounded)
                    .title(match title {
                        Some(t) => t,
                        None => "",
                    }),
            )
            .render(area, buf)
    }

    fn render_field_active(
        &self,
        area: Rect,
        buf: &mut Buffer,
        content: &str,
        title: Option<&str>,
    ) {
        Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.active_field_style)
                    .border_type(BorderType::Rounded)
                    .title_style(self.active_field_style)
                    .title(match title {
                        Some(t) => t,
                        None => "",
                    }),
            )
            .render(area, buf)
    }

    fn render_field_invalid(
        &self,
        area: Rect,
        buf: &mut Buffer,
        content: &str,
        title: Option<&str>,
    ) {
        Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.invalid_field_style)
                    .border_type(BorderType::Rounded)
                    .title_style(self.invalid_field_style)
                    .title(match title {
                        Some(t) => t,
                        None => "",
                    }),
            )
            .render(area, buf)
    }
}
