fn main() {
    let content = "Please compare user_guide.pdf with the technical_manual.docx";
    
    if let Ok(file_regex) = regex::Regex::new(r"\b([^\s]+\.(docx?|pdf|txt|md))\b") {
        println!("Regex created successfully");
        for captures in file_regex.captures_iter(content) {
            if let Some(filename) = captures.get(1) {
                println!("Found: {}", filename.as_str());
            }
        }
    } else {
        println!("Regex failed to compile");
    }
}
