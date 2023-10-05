use crossterm::event::KeyCode;
use ratatui::{prelude::*, widgets::*};

use crate::widget::Renderer;

pub enum FieldStatus {
    Valid,
    Invalid,
}

impl Into<String> for Field<'_> {
    fn into(self) -> String {
        self.fd.val.to_string()
    }
}

/// A reference to a specific field's data in a form that also indicates whether or not it's valid.
pub struct Field<'a> {
    fd: FieldData<'a>,
    status: FieldStatus,
}

impl<'a> Field<'a> {
    pub fn valid(name: &'a str, val: &'a str) -> Field<'a> {
        Self {
            fd: FieldData { name, val },
            status: FieldStatus::Valid,
        }
    }

    pub fn invalid(name: &'a str, val: &'a str) -> Field<'a> {
        Self {
            fd: FieldData { name, val },
            status: FieldStatus::Invalid,
        }
    }

    pub fn name(&self) -> &str {
        self.fd.name
    }

    pub fn value(&self) -> &str {
        self.fd.val
    }

    fn inner(&self) -> &FieldData<'_> {
        &self.fd
    }

    pub fn is_valid(&self) -> bool {
        match self.status {
            FieldStatus::Valid => true,
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
#[derive(PartialEq)]
pub enum FormSelection {
    NoSelection,
    Hovered(usize),
    Active(usize),
}

pub(crate) struct FieldBuffer {
    name: String,
    val: String,
}

impl From<Vec<(&str, &str)>> for Form {
    fn from(value: Vec<(&str, &str)>) -> Self {
        Self {
            fields: value
                .into_iter()
                .map(|(d_name, d_val)| FieldBuffer {
                    name: d_name.to_string(),
                    val: d_val.to_string(),
                })
                .collect(),
            ..Default::default()
        }
    }
}

impl From<Vec<&str>> for Form {
    fn from(value: Vec<&str>) -> Self {
        Self {
            fields: value
                .into_iter()
                .map(|d_name| FieldBuffer {
                    name: d_name.to_string(),
                    val: String::new(),
                })
                .collect(),
            ..Default::default()
        }
    }
}

impl From<Vec<FieldBuffer>> for Form {
    fn from(value: Vec<FieldBuffer>) -> Self {
        Self {
            fields: value,
            ..Default::default()
        }
    }
}

/// A widget to display data in a collection of fields
///
/// # Example
///
/// ```rust
/// let form = Form::new(&["A", "B", "C"], |field| !field.is_empty());
///
/// // all fields remain valid until form is submitted.
/// assert_eq!(form.status().iter().all(|field| field.is_valid()), true);
///
/// // fields will now be invalid after submitting.
/// form.submit();
/// assert_eq!(form.status().iter().all(|field| field.is_valid()), false);
///
/// form.select(FormSelection::Active(0));
/// form.append_selection('a');
/// assert!(form.status[0].is_valid());
/// ```
pub struct Form {
    selected: FormSelection,
    pub(crate) fields: Vec<FieldBuffer>,
    pub(crate) submitted: bool,
    validation_fn: Box<dyn Fn(&str) -> bool + 'static>,
    pub(crate) default_field_style: Style,
    pub(crate) invalid_field_style: Style,
    pub(crate) hovered_field_style: Style,
    pub(crate) active_field_style: Style,
}

impl Default for Form {
    fn default() -> Self {
        Self {
            selected: FormSelection::NoSelection,
            fields: Vec::new(),
            submitted: false,
            validation_fn: Box::new(|f| !f.is_empty()),
            default_field_style: Style::default(),
            invalid_field_style: Style::default().red().bold(),
            hovered_field_style: Style::default().cyan(),
            active_field_style: Style::default().cyan().bold(),
        }
    }
}

impl Form {
    /// Create a new [`Form`] from a slice of field titles and a validator function.
    /// `validation_fn` is used to mark fields as either valid or invalid when `.status()` is called.
    pub fn new(fields: &[&str], validation_fn: impl Fn(&str) -> bool + 'static) -> Self {
        let fields = fields
            .iter()
            .map(|&title| FieldBuffer {
                name: title.to_string(),
                val: String::new(),
            })
            .collect();

        Self {
            fields,
            validation_fn: Box::new(validation_fn),
            ..Default::default()
        }
    }

    pub fn widget(&self) -> impl Widget + '_ {
        Renderer::new(self)
    }

    pub fn select(&mut self, s: FormSelection) {
        self.selected = s;
    }

    pub fn selected(&self) -> &FormSelection {
        &self.selected
    }

    /// Submits form and returns status of fields.
    pub fn submit(&mut self) -> FormFieldStatus {
        self.submitted = true;
        self.status()
    }

    /// Returns the state of all fields in the form. Uses a [`Field`] struct to indicate whether or
    /// not each field's buffer is valid.
    pub fn status(&self) -> FormFieldStatus {
        if self.submitted {
            self.fields
                .iter()
                .map(|fb| {
                    if (self.validation_fn)(&fb.val) {
                        Field::valid(&fb.name, &fb.val)
                    } else {
                        Field::invalid(&fb.name, &fb.val)
                    }
                })
                .collect()
        } else {
            self.fields
                .iter()
                .map(|fb| Field::valid(&fb.name, &fb.val))
                .collect()
        }
    }

    pub fn input(&mut self, key: KeyCode) {
        if let FormSelection::Active(i) = self.selected {
            match key {
                KeyCode::Enter => self.next_field(),
                KeyCode::Esc => self.select(FormSelection::Hovered(i)),
                KeyCode::Backspace => self.pop_field(i),
                KeyCode::Char(ch) => self.append_field(ch, i),
                _ => {}
            }
        } else {
            match key {
                KeyCode::Esc => self.select(FormSelection::NoSelection),
                KeyCode::Char('j') => self.next_field(),
                KeyCode::Char('k') => self.prev_field(),
                KeyCode::Enter => {
                    if let FormSelection::Hovered(i) = self.selected {
                        self.selected = FormSelection::Active(i)
                    } else {
                        self.selected = FormSelection::Active(0)
                    }
                }
                _ => {}
            }
        }
    }

    fn pop_field(&mut self, field: usize) {
        self.fields[field].val.pop();
    }

    fn append_field(&mut self, ch: char, field: usize) {
        self.fields[field].val.push(ch)
    }

    pub fn append_selection(&mut self, ch: char) {
        match self.selected() {
            FormSelection::NoSelection => {}
            FormSelection::Hovered(_) => {}
            FormSelection::Active(i) => self.append_field(ch, *i),
        }
    }

    pub fn pop_selection(&mut self) {
        match self.selected() {
            FormSelection::NoSelection => {}
            FormSelection::Hovered(_) => {}
            FormSelection::Active(i) => self.pop_field(*i),
        }
    }

    pub fn deselect(&mut self) {
        self.selected = FormSelection::NoSelection
    }

    pub fn next_field(&mut self) {
        self.selected = match self.selected {
            FormSelection::NoSelection => FormSelection::Hovered(0),
            FormSelection::Hovered(i) => {
                FormSelection::Hovered((i + 1).rem_euclid(self.fields.len()))
            }
            FormSelection::Active(i) => {
                FormSelection::Active((i + 1).rem_euclid(self.fields.len()))
            }
        }
    }

    pub fn prev_field(&mut self) {
        self.selected = match self.selected {
            FormSelection::NoSelection => FormSelection::Hovered(0),
            FormSelection::Hovered(i) => {
                let i = if i == 0 { self.fields.len() - 1 } else { i - 1 };
                FormSelection::Hovered(i)
            }
            FormSelection::Active(i) => {
                let i = if i == 0 { self.fields.len() - 1 } else { i - 1 };
                FormSelection::Active(i)
            }
        }
    }

    /// Set whether the Form has been submitted
    pub fn submitted(&mut self, submitted: bool) {
        self.submitted = submitted;
    }

    pub fn active_field_style(&mut self, style: Style) {
        self.active_field_style = style;
    }

    pub fn invalid_field_style(&mut self, style: Style) {
        self.invalid_field_style = style;
    }

    pub fn hovered_field_style(&mut self, style: Style) {
        self.hovered_field_style = style;
    }

    pub fn default_field_style(&mut self, style: Style) {
        self.default_field_style = style;
    }
}
