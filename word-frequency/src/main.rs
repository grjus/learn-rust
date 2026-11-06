use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;

const FILE_EXTENSION: &str = "md";

struct Document {
    name: String,
    word_counts: HashMap<String, usize>,
    word_count: usize,
}

fn load_document(path: &str) -> Result<Document, io::Error> {
    let contents = fs::read_to_string(path)?;
    let word_counts = count_words(&contents);
    let word_count: usize = word_counts.values().sum();

    Ok(Document {
        name: path.to_string(),
        word_counts,
        word_count,
    })
}

fn load_documents(directory_path: &str, file_extension: &str) -> Result<Vec<Document>, io::Error> {
    let mut documents = Vec::new();

    for entry in fs::read_dir(directory_path)? {
        let path = entry?.path();

        if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some(file_extension)
        {
            continue;
        }

        let path = path.to_str().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "File path is not valid UTF-8")
        })?;

        documents.push(load_document(path)?);
    }

    Ok(documents)
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
    let mut args = env::args().skip(1);
    let directory_path = match (args.next(), args.next(), args.next()) {
        (Some(flag), Some(path), None) if flag == "--document-directory" => path,
        _ => {
            eprintln!("Usage: word-frequency --document-directory <path>");
            return;
        }
    };
    println!("Document directory: {directory_path}");

    println!("Enter the search query: ");
    let mut query = String::new();

    io::stdin()
        .read_line(&mut query)
        .expect("Failed to read query");

    let query_words = count_words(&query);

    let documents = match load_documents(&directory_path, FILE_EXTENSION) {
        Ok(documents) => documents,
        Err(error) => {
            eprintln!("Could not load documents: {error}");
            return;
        }
    };

    for word in query_words.keys() {
        let document_frequency = documents
            .iter()
            .filter(|document| document.word_counts.contains_key(word))
            .count();
        println!("{word}: found in {document_frequency} documents")
    }

    if documents.is_empty() {
        println!("No {FILE_EXTENSION} document found. Exiting....");
        return;
    }

    println!("Document count: {}", documents.len());

    let total_words: usize = documents.iter().map(|document| document.word_count).sum();
    let average_length = total_words as f64 / documents.len() as f64;
    println!("Average document length: {average_length:.2}");

    for document in documents {
        let matches = query_words
            .keys()
            .filter(|word| document.word_counts.contains_key(*word))
            .count();
        if matches > 0 {
            println!("{}: {matches} matching query words", document.name);
        }
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
