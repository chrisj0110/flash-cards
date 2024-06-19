use rand::seq::SliceRandom;
use serde_derive::Deserialize;
use std::env;
use std::fmt;
use std::fs::File;
use std::{io, io::BufReader};

#[derive(Debug)]
enum QuizParseError {
    FileNotFound(String),
    ParseError(String),
}

impl fmt::Display for QuizParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::FileNotFound(ref s) => write!(f, "Error reading json file: {}", s),
            Self::ParseError(ref s) => write!(f, "Parse Error: {}", s),
        }
    }
}

#[derive(Debug, Deserialize)]
struct Question {
    question: String,
    answer: usize,
    options: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Quiz {
    questions: Vec<Question>,
}

impl TryFrom<&str> for Quiz {
    type Error = QuizParseError;

    fn try_from(file_name: &str) -> Result<Self, QuizParseError> {
        let file =
            File::open(file_name).map_err(|e| QuizParseError::FileNotFound(e.to_string()))?;

        serde_json::from_reader(BufReader::new(file))
            .map_err(|e| QuizParseError::ParseError(e.to_string()))
    }
}

#[derive(Debug, Default)]
struct Results {
    correct: usize,
    incorrect: usize,
}

fn display_question(question: &str, options: &[String]) -> String {
    format!("---\n\n{}\n\n", &question)
        + &options
            .iter()
            .enumerate()
            .map(|(index, option)| format!("{} - {}\n\n", index + 1, option))
            .collect::<Vec<_>>()
            .join("")
}

fn get_user_answer_index(options: &[String]) -> usize {
    loop {
        println!("Answer: ");

        let mut user_answer = String::new();
        io::stdin()
            .read_line(&mut user_answer)
            .expect("Could not read input");

        match user_answer.trim().parse::<usize>() {
            Ok(index) if index > 0 && index <= options.len() => return index - 1,
            _ => println!("Invalid answer"),
        }
    }
}

fn get_correct_answer_index(question: &Question, options: &[String]) -> usize {
    let correct_answer = &question.options.get(question.answer - 1).unwrap();

    options
        .iter()
        .position(|option| option == *correct_answer)
        .unwrap_or_else(|| panic!("Correct answer not found"))
}

fn display_results(results: &Results) -> String {
    let total = results.correct + results.incorrect;
    let percent = match total {
        0 => 0,
        _ => 100 * results.correct / (total),
    };
    format!("{}% correct ({} of {})", percent, results.correct, total).to_string()
}

fn get_file_name_from_args(args: Vec<String>) -> Result<String, String> {
    args.get(1)
        .map(|s| s.to_string())
        .ok_or_else(|| "Error: Please input one parameter for json filename".to_string())
}

fn main() {
    let questions_file = get_file_name_from_args(env::args().collect()).unwrap_or_else(|e| {
        println!("{}", e);
        std::process::exit(1);
    });

    let quiz = Quiz::try_from(questions_file.as_str()).unwrap_or_else(|e| {
        println!("{}", e);
        std::process::exit(1);
    });

    let mut results = Results::default();

    let mut questions = quiz.questions;
    questions.shuffle(&mut rand::thread_rng());

    for question in questions {
        let mut options = question.options.clone();
        options.shuffle(&mut rand::thread_rng());

        println!("{}", display_question(&question.question, &options));

        let correct_answer_index = get_correct_answer_index(&question, &options);

        if get_user_answer_index(&options) == correct_answer_index {
            results.correct += 1;
            println!("Correct!");
        } else {
            results.incorrect += 1;
            println!(
                "Incorrect! Correct answer was #{}",
                correct_answer_index + 1
            );
        }

        println!("\n{}\n\n", display_results(&results));
    }
    println!("Done!");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_correct_answer_index() {
        assert_eq!(
            get_correct_answer_index(
                &Question {
                    question: "".to_string(),
                    answer: 1,
                    options: vec![
                        "answer 1".to_string(),
                        "answer 2".to_string(),
                        "answer 3".to_string(),
                    ],
                },
                &vec![
                    "answer 1".to_string(),
                    "answer 2".to_string(),
                    "answer 3".to_string(),
                ],
            ),
            0
        );

        assert_eq!(
            get_correct_answer_index(
                &Question {
                    question: "".to_string(),
                    answer: 1,
                    options: vec![
                        "answer 1".to_string(),
                        "answer 2".to_string(),
                        "answer 3".to_string(),
                    ],
                },
                &vec![
                    "answer 2".to_string(),
                    "answer 1".to_string(),
                    "answer 3".to_string(),
                ],
            ),
            1
        );

        assert_eq!(
            get_correct_answer_index(
                &Question {
                    question: "".to_string(),
                    answer: 1,
                    options: vec![
                        "answer 1".to_string(),
                        "answer 2".to_string(),
                        "answer 3".to_string(),
                    ],
                },
                &vec![
                    "answer 2".to_string(),
                    "answer 3".to_string(),
                    "answer 1".to_string(),
                ],
            ),
            2
        );
    }

    #[test]
    fn test_quiz_try_from() {
        let quiz = Quiz::try_from("example_json.txt").unwrap();
        let question = quiz.questions.first().unwrap();
        assert_eq!(question.question, "What is 1+1?");

        let options = &question.options;
        assert_eq!(options[0], "2");
        assert_eq!(options[1], "3");
        assert_eq!(options[2], "4");

        // error scenario:
        match Quiz::try_from("does_not_exist.txt") {
            Ok(_) => panic!("This should have failed to find the file"),
            Err(e) => assert!(matches!(e, QuizParseError::FileNotFound(_))),
        };
    }

    #[test]
    fn test_display_question() {
        let display_str = display_question(
            "test question",
            &vec![
                "answer 1".to_string(),
                "answer 2".to_string(),
                "answer 3".to_string(),
            ],
        );
        assert!(display_str.contains("---"));
        assert!(display_str.contains("test question"));
        assert!(display_str.contains("1 - answer 1"));
        assert!(display_str.contains("2 - answer 2"));
        assert!(display_str.contains("3 - answer 3"));
    }

    #[test]
    fn test_display_results() {
        assert_eq!(
            display_results(&Results {
                correct: 3,
                incorrect: 2
            }),
            "60% correct (3 of 5)"
        );
        assert_eq!(
            display_results(&Results {
                correct: 0,
                incorrect: 0
            }),
            "0% correct (0 of 0)"
        );
    }

    #[test]
    fn test_get_file_name_from_args() {
        let result = get_file_name_from_args(vec!["script".to_string(), "filename.json".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result, Ok("filename.json".to_string()));

        let result = get_file_name_from_args(vec!["script".to_string(), "filename.json".to_string(), "extra".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result, Ok("filename.json".to_string()));

        let result = get_file_name_from_args(vec!["script".to_string()]);
        assert!(!result.is_ok());
    } 
}
