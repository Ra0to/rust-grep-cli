use std::{
    env::args,
    fs::File,
    io::{self, BufRead, BufReader, Lines},
    path::PathBuf,
};

struct MatchInfo {
    line: String,
    line_number: u32,
    file_path: String,
}

fn grep(args: GrepArgs) {
    let mut results = Vec::<MatchInfo>::new();

    if args.path.is_dir() {
        process_dir(&args, &mut results);
    } else {
        process_file(&args, &mut results);
    }

    print_results(&results)
}

fn print_results(results: &Vec<MatchInfo>) {
    results
        .iter()
        .for_each(|info| println!("{}({}): {}", info.file_path, info.line_number, info.line));
}

fn process_dir(args: &GrepArgs, mut results: &mut Vec<MatchInfo>) {
    std::fs::read_dir(&args.path)
        .unwrap()
        .into_iter()
        .flatten()
        .map(|entry| GrepArgs {
            path: entry.path(),
            pattern: String::from(&args.pattern),
        })
        .for_each(|entry| {
            if entry.path.is_dir() {
                process_dir(&entry, &mut results);
            } else {
                process_file(&entry, &mut results);
            }
        });
}

fn read_lines(path: &PathBuf) -> Lines<BufReader<File>> {
    // TODO return Error and don't unwrap directly
    let file = File::open(path).unwrap();
    io::BufReader::new(file).lines()
}

fn process_file(args: &GrepArgs, results: &mut Vec<MatchInfo>) {
    // TODO can be reused in all files
    let formated_pattern = format_matched_line_with_colors(&args.pattern);

    results.extend(
        read_lines(&args.path)
            .enumerate()
            .filter_map(|(line_number, res)| {
                res.ok()
                    .filter(|x| x.contains(&args.pattern))
                    .map(|line| (line_number, line))
            })
            .map(|(line_number, line)| MatchInfo {
                line: line.replace(&args.pattern, &formated_pattern),
                line_number: line_number as u32,
                file_path: args.path.to_string_lossy().to_string(),
            }),
    );
}

fn format_matched_line_with_colors(line: &str) -> String {
    format!("\x1b[6;30;42m{}\x1b[0m", line)
}

enum GrepArgsParseError {
    NoPattern,
    NoPath,
    InvalidPath,
}

struct GrepArgs {
    pattern: String,
    path: PathBuf,
}

fn get_args() -> Result<GrepArgs, GrepArgsParseError> {
    let mut args = args();
    let _script = args.next();
    let pattern = args.next().ok_or(GrepArgsParseError::NoPattern)?;

    let path_str = args.next().ok_or(GrepArgsParseError::NoPath)?;
    let path = PathBuf::from(path_str);

    if !path.exists() {
        return Err(GrepArgsParseError::InvalidPath);
    }

    Ok(GrepArgs { pattern, path })
}

fn main() {
    let args_res = get_args();
    match args_res {
        Ok(args) => grep(args),
        Err(err) => match err {
            GrepArgsParseError::NoPattern => println!("Error! No pattern provided"),
            GrepArgsParseError::NoPath => println!("Error! No path provided"),
            GrepArgsParseError::InvalidPath => println!("Error! Invalid path"),
        },
    }
}
