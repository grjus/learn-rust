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

fn main() {
    let mut args = env::args().skip(1);
    let directory_path = match (args.next(), args.next(), args.next()) {
        (Some(flag), Some(path), None) if flag == "--document-directory" => path,
        _ => {
            eprintln!("Usage: word-frequency --document-directory <path>");
            return;
        }
    };

    let documents = match load_documents(&directory_path, FILE_EXTENSION) {
        Ok(documents) => documents,
        Err(error) => {
            eprintln!("Could not load documents: {error}");
            return;
        }
    };

    if documents.is_empty() {
        println!("No {FILE_EXTENSION} document found. Exiting....");
        return;
    }

    println!("Document count: {}", documents.len());
    let total_words: usize = documents.iter().map(|document| document.word_count).sum();
    let average_length = total_words as f64 / documents.len() as f64;
    println!("Average document length: {average_length:.2}");

    loop {
        println!("Enter the search query: ");
        let mut query = String::new();

        match io::stdin().read_line(&mut query) {
            Ok(0) => break,
            Ok(_) => {}
            Err(error) => {
                eprintln!("Could not read query :{error}");
                break;
            }
        }

        if query.trim().is_empty() {
            break;
        }

        let query_words = count_words(&query);

        if query_words.is_empty() {
            println!("Please enter a query containing at least one word...");
            return;
        }

        let mut results = Vec::new();
        for document in &documents {
            let score = bm25_score(document, &documents, &query_words, average_length);
            if score > 0.0 {
                results.push((document, score));
            }
        }
        results.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.name.cmp(&b.0.name)));
        if results.is_empty() {
            println!("No matching documents found.");
            return;
        }
        for (document, score) in results {
            println!("{},{score:.3}", document.name)
        }
    }
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
fn document_frequency(documents: &[Document], word: &str) -> usize {
    documents
        .iter()
        .filter(|document| document.word_counts.contains_key(word))
        .count()
}

fn inverse_document_frequency(documents: &[Document], word: &str) -> f64 {
    let total = documents.len() as f64;
    let frequency = document_frequency(documents, word) as f64;
    (1.0 + (total - frequency + 0.5) / (frequency + 0.5)).ln()
}

fn bm25_score(
    document: &Document,
    documents: &[Document],
    query_words: &HashMap<String, usize>,
    average_length: f64,
) -> f64 {
    let k1 = 1.2;
    let b = 0.75;
    let mut score = 0.0;

    for word in query_words.keys() {
        let frequency = document.word_counts.get(word).copied().unwrap_or(0) as f64;
        if frequency == 0.0 {
            continue;
        }

        let idf = inverse_document_frequency(documents, word);
        let length_ratio = document.word_count as f64 / average_length;

        let numerator = frequency * (k1 + 1.0);
        let denominator = frequency + k1 * (1.0 - b + b * length_ratio);

        score += idf * numerator / denominator;
    }

    score
}

#[cfg(test)]
mod tests {
    use crate::{Document, bm25_score, count_words};

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

    #[test]
    fn matching_document_scores_above_nonmatching_document() {
        let documents = vec![
            Document {
                name: "matching".to_string(),
                word_counts: count_words("rust code"),
                word_count: 2,
            },
            Document {
                name: "other".to_string(),
                word_counts: count_words("python code"),
                word_count: 2,
            },
        ];
        let query = count_words("rust");
        let matching = bm25_score(&documents[0], &documents, &query, 2.0);
        let other = bm25_score(&documents[1], &documents, &query, 2.0);
        assert!(matching > other);
        assert_eq!(other, 0.0);
    }

    #[test]
    fn shorter_document_scores_higher_for_equal_word_frequency() {
        let documents = vec![
            Document {
                name: "short".to_string(),
                word_counts: count_words("rust"),
                word_count: 1,
            },
            Document {
                name: "long".to_string(),
                word_counts: count_words("rust code code code code"),
                word_count: 5,
            },
        ];

        let query = count_words("rust");
        let short = bm25_score(&documents[0], &documents, &query, 3.0);
        let long = bm25_score(&documents[1], &documents, &query, 3.0);

        assert!(short > long);
    }
}
