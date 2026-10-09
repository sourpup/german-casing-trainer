use inquire::{Confirm, Select, Text};
use rand::RngExt;
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

fn main() {
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
