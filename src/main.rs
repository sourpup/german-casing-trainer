use color_eyre::Result;
use crossterm::event::{self, KeyCode, KeyEventKind};
use inquire::{Confirm, Select, Text};
use rand::RngExt;
use ratatui::layout::{Constraint, Layout, Position};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text as RatatuiText};
use ratatui::widgets::{Block, List, ListItem, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use serde::{Deserialize, Serialize};
use serde_json::from_reader;
use std::fs::File;
use std::io::{BufReader, Write};
mod throbber;

#[derive(Serialize, Deserialize, Debug)]
enum TrainingMode {
    Easy,
    Hard,
    Sentence,
}

#[derive(Serialize, Deserialize, Debug)]
struct Settings {
    mode: TrainingMode,
    frequency: f32,
    genders: Vec<Gender>,
    cases: Vec<Case>,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            mode: TrainingMode::Easy,
            frequency: 0.005,
            genders: vec![Gender::Masculine, Gender::Feminine, Gender::Neuter],
            cases: vec![
                Case::Nominative,
                Case::Accusative,
                Case::Dative,
                Case::Genitive,
            ],
        }
    }
}

fn load_settings() -> Result<Settings, serde_json::Error> {
    let file = File::open("settings.json");
    match file {
        Ok(file) => {
            let reader = BufReader::new(file);
            let data = from_reader::<_, Settings>(reader);
            match data {
                Ok(data) => {
                    let resume = Confirm::new("Resume previous session?")
                        .with_default(true)
                        .prompt()
                        .unwrap();

                    match resume {
                        true => Ok(data),
                        false => settings_select(),
                    }
                }

                Err(e) => {
                    println!("error loading previous settings: {e}");
                    settings_select()
                }
            }
        }
        Err(e) => {
            println!("error loading settings: {e}");
            settings_select()
        }
    }
}

fn settings_select() -> Result<Settings, serde_json::Error> {
    let mut settings = Settings::default();
    // determine what mode the user wants to be in
    let options = vec!["single word - easy", "single word - hard", "full sentences"];
    let mode_query = Select::new("please select training mode.", options.clone()).prompt();

    settings.mode = match mode_query {
        Ok(mode_query) => match options.into_iter().position(|x| x.contains(mode_query)) {
            Some(0) => TrainingMode::Easy,
            Some(1) => TrainingMode::Hard,
            Some(2) => TrainingMode::Sentence,
            _ => {
                println!("error while selecting mode: unexpected mode selected");
                TrainingMode::Easy
            }
        },
        Err(_) => {
            eprintln!("error while selecting mode");
            TrainingMode::Easy
        }
    };

    let freq_options = vec!["Easy", "Medium", "Difficult", "Nightmare"];
    let freq_query = Select::new("please select word difficulty", freq_options.clone()).prompt();
    settings.frequency = match freq_query {
        Ok(freq_query) => match freq_options
            .into_iter()
            .position(|x| x.contains(freq_query))
        {
            Some(0) => 0.005,
            Some(1) => 0.001,
            Some(2) => 0.00001,
            Some(3) => 0.,
            _ => {
                println!("error while selecting frequency: unexpected frequency selected");
                0.005
            }
        },
        Err(_) => {
            eprintln!("error while selecting mode");
            0.005
        }
    };

    let j = serde_json::to_string(&settings)?;
    let buffer = File::create("settings.json");
    match buffer {
        Ok(mut buffer) => {
            let resp = buffer.write_fmt(format_args!("{}", j));
            match resp {
                Ok(_) => println!("successfully wrote to disk."),
                Err(e) => println!("error writing to disk: {e}"),
            }
        }
        Err(e) => println!("error writing settings to disk: {e}"),
    }

    Ok(settings)
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum Gender {
    #[serde(rename(deserialize = "m", serialize = "m"))]
    Masculine,
    #[serde(rename(deserialize = "f", serialize = "f"))]
    Feminine,
    #[serde(rename(deserialize = "n", serialize = "n"))]
    Neuter,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
enum Case {
    Nominative,
    Accusative,
    Dative,
    Genitive,
}

#[derive(Deserialize, Debug)]
struct Translations {
    en: Vec<String>,
}

// lemma is the base word, not including the article
#[derive(Deserialize, Debug)]
struct Noun {
    gender: Option<Gender>,
    lemma: String,
    frequency: f32,
    translations: Translations,
}

#[allow(dead_code)]
struct Verb {
    sing_1: String,
    sing_2: String,
    sing_3: String,
    plur_1: String,
    plur_2: String,
    plur_3: String,
}

fn parse_json(file_name: &str) -> std::result::Result<Vec<Noun>, serde_json::Error> {
    use throbber::{CIRCLE_F, Throbber};
    let mut throbber = Throbber::default()
        .message("loading words...")
        .frames(&CIRCLE_F);

    throbber.start();
    let file = File::open(file_name).unwrap();
    let reader = BufReader::new(file);
    from_reader::<_, Vec<Noun>>(reader)
}

fn get_nouns(
    mut data: Vec<Noun>,
    frequency: f32,
) -> std::result::Result<Vec<Noun>, serde_json::Error> {
    data.retain(|x| x.gender.is_some() && x.frequency > frequency);
    Ok(data)
}

fn get_article(case: Case, gender: &Gender) -> String {
    match case {
        Case::Nominative => {
            println!("case: nominative");
            match gender {
                Gender::Masculine => "der".to_string(),
                Gender::Feminine => "die".to_string(),
                Gender::Neuter => "das".to_string(),
            }
        }
        Case::Accusative => {
            println!("case: accusative");
            match gender {
                Gender::Masculine => "den".to_string(),
                Gender::Feminine => "die".to_string(),
                Gender::Neuter => "das".to_string(),
            }
        }
        Case::Dative => {
            println!("case: dative");
            match gender {
                Gender::Masculine => "dem".to_string(),
                Gender::Feminine => "der".to_string(),
                Gender::Neuter => "dem".to_string(),
            }
        }
        Case::Genitive => {
            println!("case: genitive");
            match gender {
                Gender::Masculine => "des".to_string(),
                Gender::Feminine => "der".to_string(),
                Gender::Neuter => "des".to_string(),
            }
        }
    }
}

/// App holds the state of the application
struct App {
    /// Current value of the input box
    input: String,
    /// Position of cursor in the editor area.
    character_index: usize,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    messages: Vec<String>,
}

enum InputMode {
    Normal,
    Editing,
}

impl App {
    const fn new() -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            character_index: 0,
        }
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    const fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn submit_message(&mut self) {
        self.messages.push(self.input.clone());
        self.input.clear();
        self.reset_cursor();
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;

            if let Some(key) = event::read()?.as_key_press_event() {
                match self.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('e') => {
                            self.input_mode = InputMode::Editing;
                        }
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        _ => {}
                    },
                    InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Enter => self.submit_message(),
                        KeyCode::Char(to_insert) => self.enter_char(to_insert),
                        KeyCode::Backspace => self.delete_char(),
                        KeyCode::Left => self.move_cursor_left(),
                        KeyCode::Right => self.move_cursor_right(),
                        KeyCode::Esc => self.input_mode = InputMode::Normal,
                        _ => {}
                    },
                    InputMode::Editing => {}
                }
            }
        }
    }

    fn render(&self, frame: &mut Frame) {
        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let [help_area, input_area, messages_area] = frame.area().layout(&layout);

        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec![
                    "Press ".into(),
                    "q".bold(),
                    " to exit, ".into(),
                    "e".bold(),
                    " to start editing.".bold(),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            InputMode::Editing => (
                vec![
                    "Press ".into(),
                    "Esc".bold(),
                    " to stop editing, ".into(),
                    "Enter".bold(),
                    " to record the message".into(),
                ],
                Style::default(),
            ),
        };
        let text = RatatuiText::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, help_area);

        let input = Paragraph::new(self.input.as_str())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, input_area);
        match self.input_mode {
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            InputMode::Normal => {}

            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            #[expect(clippy::cast_possible_truncation)]
            InputMode::Editing => frame.set_cursor_position(Position::new(
                // Draw the cursor at the current position in the input field.
                // This position can be controlled via the left and right arrow key
                input_area.x + self.character_index as u16 + 1,
                // Move one line down, from the border to the input line
                input_area.y + 1,
            )),
        }

        let messages: Vec<ListItem> = self
            .messages
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let content = Line::from(Span::raw(format!("{i}: {m}")));
                ListItem::new(content)
            })
            .collect();
        let messages = List::new(messages).block(Block::bordered().title("Messages"));
        frame.render_widget(messages, messages_area);
    }
}

fn main() {
    color_eyre::install();
    ratatui::run(|terminal| App::new().run(terminal));

    println!("welcome to german casing trainer!");

    let settings = match load_settings() {
        Ok(settings) => settings,
        Err(e) => {
            println!("error while selecting settings: {e}");
            Settings::default()
        }
    };

    let all_words = parse_json("all.json");

    let all_nouns = match all_words {
        Ok(data) => get_nouns(data, settings.frequency),
        Err(e) => panic!("error while parsing json: {e}"),
    };

    let nouns = match all_nouns {
        Ok(data) => data,
        Err(e) => panic!("error while removing non-nouns: {e}"),
    };

    let mut cont = true;

    if let TrainingMode::Sentence = settings.mode {
        todo!()
    } else {
        let mut rng = rand::rng();
        while cont {
            let index = rng.random_range(0..nouns.len());
            let word = &nouns[index];
            let case = match rng.random_range(0..4) {
                1 => Case::Accusative,
                2 => Case::Dative,
                3 => Case::Genitive,
                _ => Case::Nominative,
            };

            println!();
            println!(
                "given the word '{}', with definitions and info: ",
                word.lemma
            );
            // println!("‾‾‾‾‾‾");
            for definitions in word.translations.en.clone() {
                println!("- {}", definitions)
            }
            println!();
            if let TrainingMode::Easy = settings.mode {
                match word.gender {
                    Some(Gender::Masculine) => println!("gender: masculine"),
                    Some(Gender::Feminine) => println!("gender: feminine"),
                    Some(Gender::Neuter) => println!("gender: neuter"),
                    None => panic!("no gender???"),
                }
            }

            let article = get_article(case, word.gender.as_ref().unwrap());

            println!();
            let response = Text::new("what is the definite article + word?").prompt();

            match response {
                Ok(response) => {
                    let mut answer = article;
                    answer.push_str(" ");
                    answer.push_str(word.lemma.as_str());
                    if answer == response {
                        println!("correct answer!");
                    } else {
                        println!("incorrect");
                        println!("your answer:    {response}");
                        println!("correct answer: {answer}");
                    }
                }
                Err(e) => println!("error matching response to answer: {e}"),
            }

            cont = Confirm::new("Continue?")
                .with_default(true)
                .prompt()
                .unwrap();
        }
    }
}
