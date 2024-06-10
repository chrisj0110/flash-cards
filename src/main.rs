use rand::seq::SliceRandom;
use serde_json::Value;
use std::fs::File;
use std::{io, io::BufReader};

#[derive(Debug)]
struct Answer {
    answer: String,
    is_correct: bool,
}

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
        if let Ok(Value::Array(question_list)) = data {
            for question in question_list {
                if let Value::Object(obj) = question {
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
            }
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
    let mut user_answer_index = 0;
    let mut is_valid_input = false;

    while !is_valid_input {
        let mut user_answer = String::new();

        println!("Answer: ");

        io::stdin()
            .read_line(&mut user_answer)
            .expect("Could not read input");

        if let Ok(index) = user_answer.trim().parse::<usize>() {
            if index > 0 && index <= answers.len() {
                user_answer_index = index;
                is_valid_input = true;
            }
        }
    }
    user_answer_index - 1
}

fn display_results(results: &Results) -> String {
    let total = results.correct + results.incorrect;
    let percent = match total {
        0 => 0,
        _ => 100 * results.correct / (total),
    };
    format!("{}% correct ({} of {})", percent, results.correct, total).to_string()
}

// for each question in random order:
// print question, answers in random order, prompt with a/b/c/d
// if the inputted letter that matches with the nth answer has is_correct=true, then correct

fn main() {
    let quiz = Quiz::from("datadog_logs.json");

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
        } else {
            results.incorrect += 1;
        }

        println!("\n{}\n\n---\n\n", display_results(&results));
    }
    println!("Done!");
}
