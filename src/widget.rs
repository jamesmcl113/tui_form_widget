use std::rc::Rc;

use crate::{Form, FormSelection};
use ratatui::{prelude::*, widgets::*};

pub struct Renderer<'a>(&'a Form);

impl<'a> Renderer<'a> {
    pub fn new(form: &'a Form) -> Self {
        Renderer(form)
    }
}

impl<'a> Widget for Renderer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Block::new().title("Form").render(area, buf);
        let constraints: Vec<Constraint> = self
            .0
            .fields
            .iter()
            .map(|_| Constraint::Max(3))
            .chain([Constraint::Max(1)])
            .collect();

        let area = Layout::default()
            .direction(Direction::Vertical)
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

impl<'a> Renderer<'a> {
    fn render_fields(&self, area: Rc<[Rect]>, buf: &mut Buffer) {
        let fields = self.0.status();
        fields.iter().enumerate().for_each(|(i, field)| {
            let is_invalid = !field.is_valid() && self.0.submitted;
            let hovered = if let FormSelection::Hovered(f) = self.0.selected() {
                *f == i
            } else {
                false
            };

            let active = if let FormSelection::Active(f) = self.0.selected() {
                *f == i
            } else {
                false
            };

            let render_type = match (hovered, active, is_invalid) {
                (_, true, _) => FieldRenderType::Active,
                (true, false, _) => FieldRenderType::Hovered,
                (false, false, true) => FieldRenderType::Invalid,
                (false, false, false) => FieldRenderType::Normal,
            };
            self.render_field_gen(area[i], buf, field.value(), Some(field.name()), render_type);
        });
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
                    .border_style(self.0.hovered_field_style)
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
        Paragraph::new(Line::from(vec![
            Span::raw(content),
            Span::styled(" ", Style::default().reversed()),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(self.0.active_field_style)
                .border_type(BorderType::Rounded)
                .title_style(self.0.active_field_style)
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
                    .border_style(self.0.invalid_field_style)
                    .border_type(BorderType::Rounded)
                    .title_style(self.0.invalid_field_style)
                    .title(match title {
                        Some(t) => t,
                        None => "",
                    }),
            )
            .render(area, buf)
    }
}
