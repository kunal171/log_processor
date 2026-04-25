use std::fs;


fn main(){
    let logs = fs::read_to_string("logs.txt").expect("Failed to read log file");

    let mut error_count = 0;

    for line in logs.lines() {
        if line.contains("ERROR") {
            error_count += 1;
        }
    }

    println!("Errors: {}", error_count);

}