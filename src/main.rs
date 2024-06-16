use rand::seq::SliceRandom;
use serde_json::Value;
use std::env;
use std::fs::File;
use std::{io, io::BufReader};

#[derive(Debug)]
struct Answer {
    answer: String,
    is_correct: bool,
}

// get zero-based index of the correct answer
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
struct Quiz {
    questions: Vec<Question>,
}

impl From<&str> for Quiz {
    fn from(file_name: &str) -> Self {
        let file = File::open(file_name).unwrap();
        let data = serde_json::from_reader(BufReader::new(file));

        let mut questions: Vec<Question> = vec![];
        match data {
            Ok(Value::Array(question_list)) => {
                for question in question_list {
                    match question {
                        Value::Object(obj) => {
                            let answer_num: usize = obj
                                .get("Answer")
                                .unwrap()
                                .as_u64()
                                .unwrap()
                                .try_into()
                                .unwrap();

                            let answers: Vec<Answer> = obj
                                .get("Options")
                                .unwrap()
                                .as_array()
                                .unwrap()
                                .iter()
                                .enumerate()
                                .map(|(index, option)| Answer {
                                    answer: option.to_string(),
                                    is_correct: index + 1 == answer_num,
                                })
                                .collect();

                            questions.push(Question {
                                question: obj.get("Question").unwrap().to_string(),
                                answers,
                            });
                        }
                        _ => panic!("Unknown issue with the question in the JSON"),
                    }
                }
            }
            Err(e) => panic!("Could not parse JSON: {}", e),
            _ => panic!("Unknown error"),
        }
        Quiz { questions }
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
        panic!("Please input just one parameter for json filename");
    }

    let quiz = Quiz::from(args[1].as_str());

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
}
