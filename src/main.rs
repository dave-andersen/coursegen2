use chrono::Datelike;
use chrono::NaiveDate;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use toml::value::Datetime;

#[derive(Debug, Parser)]
#[command(version)]
struct Args {
    #[arg(short, long)]
    config: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    year: i32,
    term: String,
    instructor: Vec<Instructor>,
    meets: Vec<String>, // eventually need to parse this to DOW
    starts: String,
    ends: String,
    first_day: NaiveDate,
    last_day: NaiveDate,
    holiday: Option<Vec<Holiday>>,
    lecture: Vec<Lecture>,
}

#[derive(Debug, Deserialize)]
struct Holiday {
    dates: Vec<NaiveDate>,
    name: String,
}

#[derive(Debug, Deserialize)]
struct Paper {
    title: String,
    link: String,
}

#[derive(Debug, Deserialize)]
struct Lecture {
    title: String,
    notes: Option<String>,
    papers: Option<Vec<Paper>>,
}

mod toml_date_format {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(serde::de::Error::custom)?)
    }
}

#[derive(Debug, Deserialize)]
struct Instructor {
    name: Option<String>,
    email: Option<String>,
    webpage: Option<String>,
    office: Option<String>,
    hours: Option<String>,
}

fn main() {
    let args: Args = Args::parse();
    let mut file = File::open(args.config).expect("Failed to open config file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read config file");

    let config: Config = toml::from_str(&contents).expect("Failed to parse config file");

    let mut holidays: HashMap<NaiveDate, String> = HashMap::new();
    if let Some(holidaylist) = &config.holiday {
        for h in holidaylist {
            for d in &h.dates {
                holidays.insert(*d, h.name.clone());
            }
        }
    }

    let meets: HashSet<String> = config.meets.iter().cloned().collect();

    let output_html_file_name = "syllabus.html";
    std::fs::copy("syllabus_head.html", output_html_file_name).expect("Failed to copy head file");
    let mut output = File::options()
        .append(true)
        .open(output_html_file_name)
        .expect("Failed to open output file");

    let mut lecture_idx = 0;
    for day in config
        .first_day
        .iter_days()
        .take_while(|d| *d <= config.last_day)
    {
        let dow = day.weekday().to_string();

        if !meets.contains(&dow.to_lowercase()) {
            continue;
        }
        // How do we want to handle lecture headings?
        if let Some(h) = holidays.get(&day) {
            writeln!(
                &mut output,
                "<tr class=\"noclass\"><td>{dow} {}/{}<td colspan=\"3\">No Class - {h}</td></tr>",
                day.month(),
                day.day()
            );
            continue;
        }
        writeln!(
            &mut output,
            "<tr class=\"lecture\"><td>{} {}/{}</td>",
            dow,
            day.month(),
            day.day()
        );

        if lecture_idx >= config.lecture.len() {
            writeln!(&mut output, "<td></td><td></td><td></td></tr>");
            continue;
        }
        let lecture = &config.lecture[lecture_idx];
        writeln!(
            &mut output,
            "<td>{}</td><td>{}</td>",
            lecture.title,
            lecture.notes.as_deref().unwrap_or("")
        );
        if let Some(papers) = &lecture.papers {
            writeln!(&mut output, "<td>");
            for p in papers {
                let link = match p.link.starts_with("http") {
                    true => p.link.clone(),
                    false => format!("papers/{}", p.link),
                };
                write!(&mut output, "<a href=\"{}\">{}</a>, ", link, p.title);
            }
            writeln!(&mut output, "</td>");
        }
        writeln!(&mut output, "</tr>");
        lecture_idx += 1;
    }

    let mut tail = File::open("syllabus_tail.html").expect("Failed to open tail file");
    let mut tail_contents = Vec::new();
    tail.read_to_end(&mut tail_contents)
        .expect("Failed to read tail file");
    output
        .write(&tail_contents)
        .expect("Failed to write tail file");
}
