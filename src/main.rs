use std::fs;
mod alice;

fn main() {
    let result = fs::read_to_string("test_2.html");
    match result {
        Ok(content) => {
            // here's where the main code is;
            let mut tokenizer = alice::HTMLTokenizer::new(&content.chars().collect());
            tokenizer.run();
        }
        Err(_err) => {
            println!("probably couldn't read the file");
        }
    };
}
