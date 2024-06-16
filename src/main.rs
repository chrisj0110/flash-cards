use rand::seq::SliceRandom;
use serde_json::Value;
use std::env;
use std::fmt;
use std::fs::File;
use std::{io, io::BufReader};

#[derive(Debug)]
struct Answer {
    answer: String,
    is_correct: bool,
}

// get zero-based index of the correct answer
// TODO: return a Result
fn get_correct_answer_index(answers: &[Answer]) -> usize {
    answers
        .iter()
        .position(|answer| answer.is_correct)
        .unwrap_or_else(|| panic!("Correct answer not found"))
}

#[derive(Debug)]
struct Question {
    question: String,
    answers: Vec<Answer>,
}

#[derive(Debug)]
enum QuizParseError {
    FileNotFound(String),
    ParseError(String),
    UnexpectedJsonStructure,
}

impl fmt::Display for QuizParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::FileNotFound(ref s) => write!(f, "Error reading json file: {}", s),
            Self::ParseError(ref s) => write!(f, "Parse Error: {}", s),
            Self::UnexpectedJsonStructure => write!(f, "Unepected JSON structure"),
        }
    }
}

#[derive(Debug)]
struct Quiz {
    questions: Vec<Question>,
}

impl TryFrom<&str> for Quiz {
    type Error = QuizParseError;

    fn try_from(file_name: &str) -> Result<Self, QuizParseError> {
        let file = match File::open(file_name) {
            Ok(f) => f,
            Err(e) => return Err(QuizParseError::FileNotFound(e.to_string())),
        };

        let question_list = match serde_json::from_reader(BufReader::new(file)) {
            Ok(Value::Array(q)) => q,
            Err(e) => return Err(QuizParseError::ParseError(e.to_string())),
            _ => return Err(QuizParseError::UnexpectedJsonStructure),
        };

        let mut questions: Vec<Question> = vec![];
        for question in question_list {
            let answer_num: usize = question
                .get("Answer")
                .unwrap()
                .as_u64()
                .unwrap()
                .try_into()
                .unwrap();

            let answers: Vec<Answer> = question
                .get("Options")
                .unwrap()
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(index, option)| Answer {
                    answer: option.as_str().unwrap().to_string(),
                    is_correct: index + 1 == answer_num,
                })
                .collect();

            questions.push(Question {
                question: question
                    .get("Question")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string(),
                answers,
            });
        }
        Ok(Quiz { questions })
    }
}

#[derive(Debug, Default)]
struct Results {
    correct: usize,
    incorrect: usize,
}

fn display_question(question: &str, answers: &[Answer]) -> String {
    let mut output = String::new();
    output.push_str(format!("---\n\n{}\n\n", &question).as_ref());

    for (index, answer) in answers.iter().enumerate() {
        output.push_str(format!("{} - {}\n\n", index + 1, answer.answer).as_ref());
    }

    output.trim().to_string()
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

fn display_results(results: &Results) -> String {
    let total = results.correct + results.incorrect;
    let percent = match total {
        0 => 0,
        _ => 100 * results.correct / (total),
    };
    format!("{}% correct ({} of {})", percent, results.correct, total).to_string()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Error: Please input just one parameter for json filename");
        return;
    }

    let quiz = match Quiz::try_from(args[1].as_str()) {
        Ok(q) => q,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    let mut results = Results::default();

    let mut questions = quiz.questions;
    questions.shuffle(&mut rand::thread_rng());

    for question in questions {
        let mut answers = question.answers;
        answers.shuffle(&mut rand::thread_rng());

        let correct_answer_index = get_correct_answer_index(&answers);

        println!("{}\n", display_question(&question.question, &answers));

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
            get_correct_answer_index(
                vec![
                    Answer {
                        answer: "answer 1".to_string(),
                        is_correct: true
                    },
                    Answer {
                        answer: "answer 2".to_string(),
                        is_correct: false
                    },
                    Answer {
                        answer: "answer 2".to_string(),
                        is_correct: false
                    },
                ]
                .as_ref()
            ),
            0
        );

        assert_eq!(
            get_correct_answer_index(
                vec![
                    Answer {
                        answer: "answer 1".to_string(),
                        is_correct: false
                    },
                    Answer {
                        answer: "answer 2".to_string(),
                        is_correct: true
                    },
                    Answer {
                        answer: "answer 2".to_string(),
                        is_correct: false
                    },
                ]
                .as_ref()
            ),
            1
        );

        assert_eq!(
            get_correct_answer_index(
                vec![
                    Answer {
                        answer: "answer 1".to_string(),
                        is_correct: false
                    },
                    Answer {
                        answer: "answer 2".to_string(),
                        is_correct: false
                    },
                    Answer {
                        answer: "answer 2".to_string(),
                        is_correct: true
                    },
                ]
                .as_ref()
            ),
            2
        );
    }

    #[test]
    fn test_quiz_try_from() {
        let quiz = Quiz::try_from("example_json.txt").unwrap();
        let question = quiz.questions.first().unwrap();
        assert_eq!(question.question, "What is 1+1?");

        let answers = &question.answers;
        assert_eq!(answers[0].answer, "2");
        assert!(answers[0].is_correct);
        assert_eq!(answers[1].answer, "3");
        assert!(!answers[1].is_correct);
        assert_eq!(answers[2].answer, "4");
        assert!(!answers[2].is_correct);

        // error scenario:
        match Quiz::try_from("does_not_exist.txt") {
            Ok(_) => panic!("This should have failed to find the file"),
            Err(e) => assert!(matches!(e, QuizParseError::FileNotFound(_))),
        };
    }
}
