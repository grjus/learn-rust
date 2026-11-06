use std::collections::HashMap;
use std::fs;
use std::io;

struct Document {
    name: String,
    word_counts: HashMap<String, usize>,
    word_count: usize,
}

fn count_words(contents: &str) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();

    for word in contents.split_whitespace() {
        let word = word
            .trim_matches(|c: char| !c.is_alphabetic())
            .to_lowercase();
        if word.is_empty() {
            continue;
        }
        let count = counts.entry(word).or_insert(0);
        *count += 1;
    }
    return counts;
}

fn main() {
    println!("Enter the path of the file: ");
    let mut path = String::new();

    io::stdin()
        .read_line(&mut path)
        .expect("Failed to read input");

    let path = path.trim();
    println!("Selected file {path}");
    let contents = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("Could not load {path},{error}");
            return;
        }
    };

    let counts = count_words(&contents);
    let word_count = counts.values().sum::<usize>();
    let document = Document {
        name: path.to_string(),
        word_counts: counts,
        word_count,
    };

    let mut words: Vec<_> = document.word_counts.iter().collect();
    words.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));

    println!("Document: {}", document.name);
    println!("Total words: {}", document.word_count);

    println!("{:<20} {:>8}", "Word", "Count");
    println!("{}", "-".repeat(29));

    for (word, count) in words.iter().take(10) {
        println!("{word:<20} {count:>8}")
    }
}

#[cfg(test)]
mod tests {
    use crate::count_words;

    #[test]
    fn count_words_and_ignores_case_and_punctuations() {
        let counts = count_words("Rust, rust!, !!!");
        assert_eq!(counts.get("rust"), Some(&2));
        assert_eq!(counts.len(), 1)
    }

    #[test]
    fn empty_input_has_no_words() {
        assert!(count_words("").is_empty())
    }
}
