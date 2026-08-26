use atomic_write_file::AtomicWriteFile;
use chrono::Datelike;
use chrono::NaiveDate;
use chrono::Weekday;
use clap::Parser;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use tera::Context;
use tera::Tera;

#[derive(Debug, Parser)]
#[command(version)]
struct Args {
    #[arg(short, long, default_value = "config.toml")]
    config: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    year: i32,
    term: String,
    instructor: Vec<Instructor>,
    meets: Vec<String>,
    starts: String,
    ends: String,
    location: String,
    first_day: NaiveDate,
    last_day: NaiveDate,
    #[serde(default)]
    holiday: Vec<Holiday>,
    #[serde(default)]
    exam: Vec<Exam>,
    lecture: Vec<Lecture>,
    #[serde(default)]
    post_class_event: Vec<PostClassEvent>,
}

#[derive(Debug, Deserialize)]
struct PostClassEvent {
    date: NaiveDate,
    title: String,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Exam {
    name: String,
    date: NaiveDate,
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
    section_header: Option<String>,
    instructor: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Instructor {
    name: Option<String>,
    email: Option<String>,
    webpage: Option<String>,
    office: Option<String>,
    hours: Option<String>,
}

fn weekday_from_str(s: &str) -> Result<Weekday, String> {
    match s.to_lowercase().as_str() {
        "mon" | "monday" => Ok(Weekday::Mon),
        "tue" | "tues" | "tuesday" => Ok(Weekday::Tue),
        "wed" | "weds" | "wednesday" => Ok(Weekday::Wed),
        "thu" | "thur" | "thurs" | "thursday" => Ok(Weekday::Thu),
        "fri" | "friday" => Ok(Weekday::Fri),
        "sat" | "saturday" => Ok(Weekday::Sat),
        "sun" | "sunday" => Ok(Weekday::Sun),
        _ => Err(format!(
            "config `meets`: unrecognized weekday {s:?} (expected e.g. \"mon\" or \"monday\")"
        )),
    }
}

fn instructor_html(instructors: &[Instructor]) -> Result<String, std::fmt::Error> {
    let mut html = String::from("<ul>");

    for instructor in instructors {
        write!(&mut html, "<li>")?;
        if let Some(name) = &instructor.name {
            write!(&mut html, "<strong>{name}</strong>")?;
        }
        if let Some(email) = &instructor.email {
            write!(&mut html, " <a href=\"mailto:{email}\">{email}</a>")?;
        }
        if let Some(webpage) = &instructor.webpage {
            write!(&mut html, " <a href=\"{webpage}\">webpage</a>")?;
        }
        if let Some(office) = &instructor.office {
            write!(&mut html, "<br />Office: {office}")?;
        }
        if let Some(hours) = &instructor.hours {
            write!(&mut html, "<br />Office hours: {hours}")?;
        }
        writeln!(&mut html, "</li>")?;
    }

    html.push_str("</ul>");
    Ok(html)
}

fn holiday_map(holidays: &[Holiday]) -> Result<HashMap<NaiveDate, String>, String> {
    let mut holidays_by_date = HashMap::new();

    for holiday in holidays {
        for date in &holiday.dates {
            if let Some(previous_name) = holidays_by_date.insert(*date, holiday.name.clone()) {
                return Err(format!(
                    "duplicate holiday date {date}: {previous_name:?} and {:?}",
                    holiday.name
                ));
            }
        }
    }

    Ok(holidays_by_date)
}

fn schedule_html(
    config: &Config,
    holidays: &HashMap<NaiveDate, String>,
    meets: &HashSet<Weekday>,
    exams: &HashMap<NaiveDate, &Exam>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut schedule = String::new();
    let mut lecture_idx = 0;

    for day in config
        .first_day
        .iter_days()
        .take_while(|d| *d <= config.last_day)
    {
        if !meets.contains(&day.weekday()) {
            continue;
        }
        let dow = day.weekday().to_string();

        if let Some(holiday) = holidays.get(&day) {
            writeln!(
                &mut schedule,
                "<tr class=\"noclass\"><td>{dow} {}/{}</td><td colspan=\"3\">No Class - {holiday}</td></tr>",
                day.month(),
                day.day()
            )?;
            continue;
        }
        if let Some(exam) = exams.get(&day) {
            writeln!(
                &mut schedule,
                "<tr class=\"lecture\"><td>{} {}/{} </td>",
                dow,
                day.month(),
                day.day(),
            )?;
            writeln!(&mut schedule, "<td>{}</td><td></td>", exam.name)?;
            writeln!(&mut schedule, "<td>")?;
            writeln!(&mut schedule, "</td>")?;
            writeln!(&mut schedule, "</tr>")?;
            continue;
        }

        if lecture_idx >= config.lecture.len() {
            writeln!(
                &mut schedule,
                "<tr class=\"lecture\"><td>{} {}/{}</td><td></td><td></td><td></td></tr>",
                dow,
                day.month(),
                day.day()
            )?;
            continue;
        }
        let lecture = &config.lecture[lecture_idx];
        if let Some(section_header) = &lecture.section_header {
            writeln!(
                &mut schedule,
                "<tr class=\"lechead\"><td class=\"lechead\" colspan=\"4\">{}</td></tr>",
                section_header
            )?;
        }

        writeln!(
            &mut schedule,
            "<tr class=\"lecture\"><td>{} {}/{} {}</td>",
            dow,
            day.month(),
            day.day(),
            lecture.instructor.as_deref().unwrap_or("")
        )?;
        writeln!(
            &mut schedule,
            "<td>{}</td><td>{}</td>",
            lecture.title,
            lecture.notes.as_deref().unwrap_or("")
        )?;
        writeln!(&mut schedule, "<td>")?;
        if let Some(papers) = &lecture.papers {
            for (index, paper) in papers.iter().enumerate() {
                if index > 0 {
                    write!(&mut schedule, ", ")?;
                }
                let link = if paper.link.starts_with("http") {
                    paper.link.as_str()
                } else {
                    write!(
                        &mut schedule,
                        "<a href=\"papers/{}\">{}</a>",
                        paper.link, paper.title
                    )?;
                    continue;
                };
                write!(&mut schedule, "<a href=\"{link}\">{}</a>", paper.title)?;
            }
        }
        writeln!(&mut schedule, "</td>")?;
        writeln!(&mut schedule, "</tr>")?;
        lecture_idx += 1;
    }

    if lecture_idx < config.lecture.len() {
        let dropped_lectures = config.lecture.len() - lecture_idx;
        return Err(format!(
            "{dropped_lectures} lecture(s) extend past the semester end; first unrendered lecture: {:?}",
            config.lecture[lecture_idx].title
        )
        .into());
    }

    for event in &config.post_class_event {
        let dow = event.date.weekday();
        writeln!(
            &mut schedule,
            "<tr class=\"deadline\"><td>{} {}/{} </td>",
            dow,
            event.date.month(),
            event.date.day(),
        )?;
        writeln!(
            &mut schedule,
            "<td>{}</td><td>{}</td><td></td></tr>",
            event.title,
            event.notes.as_deref().unwrap_or("")
        )?;
    }

    Ok(schedule)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();
    let contents = std::fs::read_to_string(&args.config)
        .map_err(|error| format!("reading config {}: {error}", args.config))?;
    let config: Config = toml::from_str(&contents)?;

    let holidays = holiday_map(&config.holiday)?;

    let exams: HashMap<NaiveDate, &Exam> =
        config.exam.iter().map(|exam| (exam.date, exam)).collect();

    let meets: HashSet<Weekday> = config
        .meets
        .iter()
        .map(|meeting| weekday_from_str(meeting))
        .collect::<Result<_, _>>()?;
    let schedule = schedule_html(&config, &holidays, &meets, &exams)?;
    let semester = format!("{} {}", config.term, config.year);
    let meeting_times = format!(
        "{} {}-{}",
        config.meets.join("/"),
        config.starts,
        config.ends
    );
    let generated_at = chrono::Local::now().format("%Y-%m-%d").to_string();
    let syllabus_instructors = instructor_html(&config.instructor)?;
    let syllabus_template = std::fs::read_to_string("syllabus_template.html")?;
    let index_template = match std::fs::read_to_string("index_template.html") {
        Ok(template) => Some(template),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };

    let mut context = Context::new();
    context.insert("semester", &semester);
    context.insert("meeting_times", &meeting_times);
    context.insert("location", &config.location);
    context.insert("instructors", &config.instructor);
    context.insert("syllabus_instructors", &syllabus_instructors);
    context.insert("schedule", &schedule);
    context.insert("generated_at", &generated_at);

    let mut tera = Tera::default();
    tera.add_raw_template("syllabus_template.html", &syllabus_template)?;
    if let Some(template) = &index_template {
        tera.add_raw_template("index_template.html", template)?;
    }

    let syllabus = tera.render("syllabus_template.html", &context)?;
    let index = index_template
        .as_ref()
        .map(|_| tera.render("index_template.html", &context))
        .transpose()?;

    let mut syllabus_output = AtomicWriteFile::options().open("syllabus.html")?;
    syllabus_output.write_all(syllabus.as_bytes())?;
    syllabus_output.commit()?;

    if let Some(index) = index {
        let mut index_output = AtomicWriteFile::options().open("index.html")?;
        index_output.write_all(index.as_bytes())?;
        index_output.commit()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use chrono::Weekday;
    use clap::Parser;
    use std::collections::HashMap;
    use std::collections::HashSet;
    use tera::Context;
    use tera::Tera;

    fn test_config(
        first_day: NaiveDate,
        last_day: NaiveDate,
        lecture: Vec<super::Lecture>,
    ) -> super::Config {
        super::Config {
            year: 2026,
            term: "Fall".to_owned(),
            instructor: Vec::new(),
            meets: vec!["fri".to_owned(), "mon".to_owned()],
            starts: "12:30".to_owned(),
            ends: "13:50".to_owned(),
            location: "Room".to_owned(),
            first_day,
            last_day,
            holiday: Vec::new(),
            exam: Vec::new(),
            lecture,
            post_class_event: Vec::new(),
        }
    }

    fn lecture(title: &str) -> super::Lecture {
        super::Lecture {
            title: title.to_owned(),
            notes: None,
            papers: None,
            section_header: None,
            instructor: None,
        }
    }

    #[test]
    fn tera_autoescapes_values_and_allows_explicit_safe_html() {
        let mut context = Context::new();
        context.insert("semester", "Fall & 2026");
        context.insert("schedule", "<tr><td>Lecture</td></tr>");
        let mut tera = Tera::default();
        let rendered = tera
            .add_raw_template(
                "syllabus_template.html",
                "<p>{{ semester }}</p>{{ schedule | safe }}",
            )
            .and_then(|()| tera.render("syllabus_template.html", &context));

        assert!(rendered.is_ok());
        assert_eq!(
            rendered.ok().as_deref(),
            Some("<p>Fall &amp; 2026</p><tr><td>Lecture</td></tr>")
        );
    }

    #[test]
    fn tera_renders_serialized_instructors_for_index_templates() {
        let instructors = [super::Instructor {
            name: Some("Ada Lovelace".to_owned()),
            email: Some("ada@example.test".to_owned()),
            webpage: None,
            office: Some("Room 101".to_owned()),
            hours: None,
        }];
        let mut context = Context::new();
        context.insert("instructors", &instructors);
        let mut tera = Tera::default();
        let rendered = tera
            .add_raw_template(
                "index_template.html",
                "{% for instructor in instructors %}{{ instructor.name }}: {{ instructor.office }}{% endfor %}",
            )
            .and_then(|()| tera.render("index_template.html", &context));

        assert_eq!(rendered.ok().as_deref(), Some("Ada Lovelace: Room 101"));
    }

    #[test]
    fn tera_rejects_malformed_template_syntax() {
        assert!(Tera::default()
            .add_raw_template("syllabus_template.html", "{{")
            .is_err());
        assert!(Tera::one_off("{{ missing }}", &Context::new(), true).is_err());
    }

    #[test]
    fn configuration_defaults_to_current_directory() {
        let default_config = super::Args::try_parse_from(["coursegen2"]);
        let explicit_config = super::Args::try_parse_from(["coursegen2", "--config", "other.toml"]);

        assert_eq!(
            default_config.ok().map(|args| args.config),
            Some("config.toml".to_owned())
        );
        assert_eq!(
            explicit_config.ok().map(|args| args.config),
            Some("other.toml".to_owned())
        );
    }

    #[test]
    fn configured_exam_occupies_its_meeting_slot() {
        let exam_date = NaiveDate::from_ymd_opt(2026, 10, 9).unwrap();
        let config = super::Config {
            year: 2026,
            term: "Fall".to_owned(),
            instructor: Vec::new(),
            meets: vec!["fri".to_owned(), "mon".to_owned()],
            starts: "12:30".to_owned(),
            ends: "13:50".to_owned(),
            location: "Room".to_owned(),
            first_day: exam_date,
            last_day: NaiveDate::from_ymd_opt(2026, 10, 12).unwrap(),
            holiday: Vec::new(),
            exam: Vec::new(),
            lecture: vec![super::Lecture {
                title: "After Exam".to_owned(),
                notes: None,
                papers: None,
                section_header: None,
                instructor: None,
            }],
            post_class_event: vec![super::PostClassEvent {
                date: NaiveDate::from_ymd_opt(2026, 10, 12).unwrap(),
                title: "Final Report Due".to_owned(),
                notes: None,
            }],
        };
        let exam = super::Exam {
            name: "Midterm 1".to_owned(),
            date: exam_date,
        };
        let exams = HashMap::from([(exam_date, &exam)]);
        let meetings = HashSet::from([Weekday::Fri, Weekday::Mon]);
        let schedule = super::schedule_html(&config, &HashMap::new(), &meetings, &exams);

        assert_eq!(
            schedule.ok().as_deref(),
            Some(concat!(
                "<tr class=\"lecture\"><td>Fri 10/9 </td>\n",
                "<td>Midterm 1</td><td></td>\n<td>\n</td>\n</tr>\n",
                "<tr class=\"lecture\"><td>Mon 10/12 </td>\n",
                "<td>After Exam</td><td></td>\n<td>\n</td>\n</tr>\n",
                "<tr class=\"deadline\"><td>Mon 10/12 </td>\n",
                "<td>Final Report Due</td><td></td><td></td></tr>\n"
            ))
        );
    }

    #[test]
    fn weekday_aliases_are_case_insensitive_and_invalid_values_fail() {
        assert_eq!(super::weekday_from_str("MONDAY"), Ok(Weekday::Mon));
        assert_eq!(super::weekday_from_str("tues"), Ok(Weekday::Tue));
        assert_eq!(super::weekday_from_str("ThUr"), Ok(Weekday::Thu));
        assert!(super::weekday_from_str("thrusday").is_err());
    }

    #[test]
    fn holidays_do_not_consume_lecture_slots() {
        let friday = NaiveDate::from_ymd_opt(2026, 10, 9).unwrap();
        let monday = NaiveDate::from_ymd_opt(2026, 10, 12).unwrap();
        let config = test_config(friday, monday, vec![lecture("First Lecture")]);
        let holidays = HashMap::from([(friday, "Break".to_owned())]);
        let meetings = HashSet::from([Weekday::Fri, Weekday::Mon]);
        let schedule = super::schedule_html(&config, &holidays, &meetings, &HashMap::new());

        assert_eq!(
            schedule.ok().as_deref(),
            Some(concat!(
                "<tr class=\"noclass\"><td>Fri 10/9</td><td colspan=\"3\">No Class - Break</td></tr>\n",
                "<tr class=\"lecture\"><td>Mon 10/12 </td>\n",
                "<td>First Lecture</td><td></td>\n<td>\n</td>\n</tr>\n"
            ))
        );
    }

    #[test]
    fn excess_lectures_fail_instead_of_being_dropped() {
        let friday = NaiveDate::from_ymd_opt(2026, 10, 9).unwrap();
        let config = test_config(friday, friday, vec![lecture("First"), lecture("Dropped")]);
        let meetings = HashSet::from([Weekday::Fri]);
        let result = super::schedule_html(&config, &HashMap::new(), &meetings, &HashMap::new());

        assert_eq!(
            result.err().map(|error| error.to_string()),
            Some(
                "1 lecture(s) extend past the semester end; first unrendered lecture: \"Dropped\""
                    .to_owned()
            )
        );
    }

    #[test]
    fn example_configuration_parses_generic_exams() {
        let config: Result<super::Config, toml::de::Error> =
            toml::from_str(include_str!("../ex/cmu-15712/config.toml"));

        assert_eq!(
            config.ok().map(|config| {
                config
                    .exam
                    .into_iter()
                    .map(|exam| (exam.name, exam.date.to_string()))
                    .collect::<Vec<_>>()
            }),
            Some(vec![
                ("Midterm 1".to_owned(), "2026-10-09".to_owned()),
                (
                    "Midterm 2, Date Time And Location TBA".to_owned(),
                    "2026-12-04".to_owned()
                ),
            ])
        );
    }

    #[test]
    fn omitted_schedule_collections_deserialize_empty() {
        let config: Result<super::Config, toml::de::Error> = toml::from_str(
            r#"
                year = 2026
                term = "Fall"
                instructor = []
                meets = ["mon"]
                starts = "12:30"
                ends = "13:50"
                location = "Room"
                first_day = "2026-08-24"
                last_day = "2026-12-04"
                lecture = []
            "#,
        );

        assert_eq!(
            config.ok().map(|config| (
                config.holiday.len(),
                config.exam.len(),
                config.post_class_event.len()
            )),
            Some((0, 0, 0))
        );
    }

    #[test]
    fn duplicate_holiday_dates_fail_with_both_names() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 12).unwrap();
        let holidays = [
            super::Holiday {
                dates: vec![date],
                name: "Fall Break".to_owned(),
            },
            super::Holiday {
                dates: vec![date],
                name: "University Holiday".to_owned(),
            },
        ];

        assert_eq!(
            super::holiday_map(&holidays),
            Err(
                "duplicate holiday date 2026-10-12: \"Fall Break\" and \"University Holiday\""
                    .to_owned()
            )
        );
    }
}
