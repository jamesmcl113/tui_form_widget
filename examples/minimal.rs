use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};

struct State {
    form: Form,
    submissions: Option<Vec<String>>,
    should_quit: bool,
}

use tui_form_widget::{Form, FormSelection};

fn main() -> io::Result<()> {
    let mut state = State {
        form: Form::from(vec!["Account", "Username / Email", "Password"]),
        should_quit: false,
        submissions: None,
    };
    let mut terminal = setup_terminal()?;
    run(&mut terminal, &mut state)?;
    restore_terminal(&mut terminal)?;
    Ok(())
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, state: &mut State) -> io::Result<()> {
    loop {
        terminal.draw(|f| render_app(f, state))?;
        handle_input(state)?;
        if state.should_quit {
            break;
        }
    }
    Ok(())
}

fn render_app(frame: &mut Frame<CrosstermBackend<Stdout>>, state: &State) {
    match &state.submissions {
        Some(fields) => frame.render_widget(Paragraph::new(fields.join("\n")), frame.size()),
        None => frame.render_widget(state.form.widget(), frame.size()),
    }
}

fn handle_input(state: &mut State) -> io::Result<()> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            match state.form.selected() {
                FormSelection::NoSelection => match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => state.should_quit = true,
                    KeyCode::Char('s') => {
                        let fields = state.form.submit();
                        if fields.iter().any(|f| !f.is_valid()) {
                        } else {
                            // Field impls Into<String>
                            state.submissions = Some(fields.into_iter().map(Into::into).collect());

                            state.form.deselect();
                        }
                    }
                    _ => {}
                },
                _ => {}
            }

            state.form.input(key.code);
        }
    }

    Ok(())
}
