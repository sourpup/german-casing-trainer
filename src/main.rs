use inquire::{Confirm, Select, Text};

enum TrainingMode {
    Easy,
    Hard,
    Sentence,
}

#[allow(dead_code)]
enum Gender {
    Masculine,
    Feminine,
    Neuter,
}

#[allow(dead_code)]
enum Case {
    Nominative,
    Accusative,
    Dative,
    Genitive,
}

// lemma is the base word, not including the article
struct Noun {
    gender: Gender,
    case: Case,
    lemma: String,
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

fn main() {
    println!("welcome to german casing trainer!");

    let options = vec!["single word - easy", "single word - hard", "full sentences"];
    let mode_query = Select::new("please select training mode.", options.clone()).prompt();

    let mut mode = TrainingMode::Easy;
    match mode_query {
        Ok(mode_query) => match options.into_iter().position(|x| x.contains(mode_query)) {
            Some(0) => {
                println!("easy mode selected.");
                mode = TrainingMode::Easy;
            }
            Some(1) => {
                println!("hard mode selected.");
                mode = TrainingMode::Hard;
            }
            Some(2) => {
                println!("sentence mode selected.");
                mode = TrainingMode::Sentence;
            }
            _ => println!("error while selecting mode: unexpected mode selected"),
        },
        Err(_) => {
            eprintln!("error while selecting mode");
        }
    }
    // let example: Vec<Noun> = Vec::new();

    let mut cont = true;

    if let TrainingMode::Sentence = mode {
        todo!()
    } else {
        while cont {
            let example = Noun {
                gender: Gender::Masculine,
                case: Case::Nominative,
                lemma: "Tisch".to_owned(),
            };
            println!();
            println!("given the following");
            println!("word: {0}", example.lemma);
            if let TrainingMode::Easy = mode {
                match example.gender {
                    Gender::Masculine => println!("gender: masculine"),
                    Gender::Feminine => println!("gender: feminine"),
                    Gender::Neuter => println!("gender: neuter"),
                }
            }
            match example.case {
                Case::Nominative => println!("case: nominative"),
                Case::Accusative => println!("case: accusative"),
                Case::Dative => println!("case: dative"),
                Case::Genitive => println!("case: genitive"),
            }

            println!();
            let response = Text::new("what is the definite article + word?").prompt();

            match response {
                Ok(response) => {
                    let mut answer = "der".to_owned();
                    answer.push_str(" ");
                    answer.push_str(example.lemma.as_str());
                    if answer == response {
                        println!("good answer!");
                    } else {
                        println!("incorrect");
                        println!("your answer:    {response}");
                        println!("correct answer: {answer}");
                    }
                }
                Err(_) => println!("error"),
            }

            cont = Confirm::new("Continue?")
                .with_default(true)
                .prompt()
                .unwrap();
        }
    }
}
