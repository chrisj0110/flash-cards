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
struct JsonQuestion {
    question: String,
    answer: usize,
    options: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct JsonQuiz {
    questions: Vec<JsonQuestion>,
}

#[derive(Clone, Debug)]
enum Answer {
    CorrectAnswer(String),
    IncorrectAnswer(String),
}

impl Answer {
    fn as_str(&self) -> &str {
        match self {
            Answer::CorrectAnswer(s) | Answer::IncorrectAnswer(s) => s,
        }
    }
}

#[derive(Debug)]
struct Question {
    question: String,
    answers: Vec<Answer>,
}

#[derive(Debug)]
struct Quiz {
    questions: Vec<Question>,
}

impl TryFrom<&str> for Quiz {
    type Error = QuizParseError;

    fn try_from(file_name: &str) -> Result<Self, QuizParseError> {
        let file =
            File::open(file_name).map_err(|e| QuizParseError::FileNotFound(e.to_string()))?;

        let json_quiz: JsonQuiz = serde_json::from_reader(BufReader::new(file))
            .map_err(|e| QuizParseError::ParseError(e.to_string()))?;

        Ok(Quiz {
            questions: json_quiz
                .questions
                .into_iter()
                .map(|json_question| Question {
                    question: json_question.question,
                    answers: json_question
                        .options
                        .iter()
                        .enumerate()
                        .map(|(index, option)| {
                            if index + 1 == json_question.answer {
                                Answer::CorrectAnswer(option.clone())
                            } else {
                                Answer::IncorrectAnswer(option.clone())
                            }
                        })
                        .collect(),
                })
                .collect(),
        })
    }
}

#[derive(Debug, Default)]
struct Results {
    correct: usize,
    incorrect: usize,
}

fn display_question(question: &str, answers: &[Answer]) -> String {
    format!("---\n\n{}\n\n", &question)
        + &answers
            .iter()
            .enumerate()
            .map(|(index, answer)| format!("{} - {}\n\n", index + 1, answer.as_str()))
            .collect::<Vec<_>>()
            .join("")
}

fn get_user_answer_index(answers: &[Answer]) -> usize {
    loop {
        println!("Answer: ");

        let mut user_answer = String::new();
        io::stdin()
            .read_line(&mut user_answer)
            .expect("Could not read input");

        match user_answer.trim().parse::<usize>() {
            Ok(index) if index > 0 && index <= answers.len() => return index - 1,
            _ => println!("Invalid answer"),
        }
    }
}

fn get_correct_answer_index(answers: &[Answer]) -> usize {
    answers
        .iter()
        .position(|a| matches!(a, Answer::CorrectAnswer(_)))
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
        let mut answers = question.answers.clone();
        answers.shuffle(&mut rand::thread_rng());

        println!("{}", display_question(&question.question, &answers));

        let correct_answer_index = get_correct_answer_index(&answers);

        if get_user_answer_index(&answers) == correct_answer_index {
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
            get_correct_answer_index(&vec![
                Answer::CorrectAnswer("answer 1".to_string()),
                Answer::IncorrectAnswer("answer 2".to_string()),
                Answer::IncorrectAnswer("answer 3".to_string()),
            ],),
            0
        );

        assert_eq!(
            get_correct_answer_index(&vec![
                Answer::IncorrectAnswer("answer 2".to_string()),
                Answer::CorrectAnswer("answer 1".to_string()),
                Answer::IncorrectAnswer("answer 3".to_string()),
            ],),
            1
        );

        assert_eq!(
            get_correct_answer_index(&vec![
                Answer::IncorrectAnswer("answer 2".to_string()),
                Answer::IncorrectAnswer("answer 3".to_string()),
                Answer::CorrectAnswer("answer 1".to_string()),
            ],),
            2
        );
    }

    #[test]
    fn test_quiz_try_from() {
        let quiz = Quiz::try_from("example_json.txt").unwrap();
        let question = quiz.questions.first().unwrap();
        assert_eq!(question.question, "What is 10+10?");

        let answers = &question.answers;
        assert_eq!(answers[0].as_str(), "10");
        assert_eq!(answers[1].as_str(), "20");
        assert_eq!(answers[2].as_str(), "30");
        assert!(matches!(answers[0], Answer::IncorrectAnswer(_)));
        assert!(matches!(answers[1], Answer::CorrectAnswer(_)));
        assert!(matches!(answers[2], Answer::IncorrectAnswer(_)));

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
                Answer::CorrectAnswer("answer 1".to_string()),
                Answer::IncorrectAnswer("answer 2".to_string()),
                Answer::IncorrectAnswer("answer 3".to_string()),
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
        let result =
            get_file_name_from_args(vec!["script".to_string(), "filename.json".to_string()]);
        assert!(result.is_ok());
        assert_eq!(result, Ok("filename.json".to_string()));

        let result = get_file_name_from_args(vec![
            "script".to_string(),
            "filename.json".to_string(),
            "extra".to_string(),
        ]);
        assert!(result.is_ok());
        assert_eq!(result, Ok("filename.json".to_string()));

        let result = get_file_name_from_args(vec!["script".to_string()]);
        assert!(!result.is_ok());
    }
}
