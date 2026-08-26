use atomic_write_file::AtomicWriteFile;
use chrono::Datelike;
use chrono::NaiveDate;
use chrono::Weekday;
use clap::Parser;
use serde::Deserialize;
use serde::Serialize;
use tera::Context;
use tera::Tera;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;

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
    meets: Vec<String>,
    starts: String,
    ends: String,
    location: String,
    first_day: NaiveDate,
    last_day: NaiveDate,
    holiday: Option<Vec<Holiday>>,
    lecture: Vec<Lecture>,
    post_class_event: Option<Vec<PostClassEvent>>,
}

#[derive(Debug, Deserialize)]
struct PostClassEvent {
    date: NaiveDate,
    title: String,
    notes: Option<String>,
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

fn schedule_html(
    config: &Config,
    holidays: &HashMap<NaiveDate, String>,
    meets: &HashSet<Weekday>,
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
                    write!(&mut schedule, "<a href=\"papers/{}\">{}</a>", paper.link, paper.title)?;
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

    if let Some(events) = &config.post_class_event {
        for event in events {
            let dow = event.date.weekday();
            writeln!(
                &mut schedule,
                "<tr class=\"lecture\"><td>{} {}/{} </td>",
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
    }

    Ok(schedule)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();
    let contents = std::fs::read_to_string(&args.config)?;
    let config: Config = toml::from_str(&contents)?;

    let mut holidays: HashMap<NaiveDate, String> = HashMap::new();
    if let Some(holidaylist) = &config.holiday {
        for holiday in holidaylist {
            for date in &holiday.dates {
                holidays.insert(*date, holiday.name.clone());
            }
        }
    }

    let meets: HashSet<Weekday> = config
        .meets
        .iter()
        .map(|meeting| weekday_from_str(meeting))
        .collect::<Result<_, _>>()?;
    let schedule = schedule_html(&config, &holidays, &meets)?;
    let semester = format!("{} {}", config.term, config.year);
    let meeting_times = format!("{} {}-{}", config.meets.join("/"), config.starts, config.ends);
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
    use tera::Context;
    use tera::Tera;

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
}
