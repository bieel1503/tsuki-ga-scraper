use std::{fs::{File, OpenOptions, self}, io::{Read, Write}, thread, time::Duration};

use scraper::{Html, Selector};

fn main() {
    loop {
        let mut title: Option<String> = None;
        let mut chapter_content: Vec<&str> = Vec::new();
        let mut next_chapter_url: Option<&str> = None;
        let mut url = get_last_chapter_url();
        if let Some(next) = next_chapter_url {
            url = next.to_string();
        }

        let html = reqwest::blocking::get(&url).unwrap();
        title = Some(html.url().path().to_owned());

        let document = Html::parse_document(&html.text().unwrap());
        let entry_title_selector = Selector::parse("h1.entry-title").unwrap();
        let entry_content_selector = Selector::parse("div.entry-content").unwrap();
        let p_selector = Selector::parse("p").unwrap();

        if let Some(t) = document.select(&entry_title_selector).next() {
            title = Some(t.inner_html().to_owned());
        }

        chapter_content.push(title.as_ref().unwrap());
        chapter_content.push(" ");

        let mut htmls: Vec<Html> = Vec::new();
        for doc in document.select(&entry_content_selector).next().unwrap().select(&p_selector) {
            htmls.push(Html::parse_fragment(&doc.inner_html()));
        }

        let mut nodes = Vec::new();

        htmls.iter()
            .for_each(|h| {
                h.tree.values()
                    .into_iter()
                    .for_each(|n| nodes.push(n));
            });

        let mut was_href = false;
        let mut found = false;
        nodes.iter()
            .for_each(|n| {
                if let Some(element) = n.as_element() {
                    if found {return;}

                    if let Some(url) = element.attr("href") {
                        was_href = true;
                        next_chapter_url = Some(url);
                    }
                }

                if let Some(line) = n.as_text() {
                    if line.eq_ignore_ascii_case("next chapter") {
                        found = true;
                    }

                    chapter_content.push(line);
                    chapter_content.push(" ");
                }
            });

        write_chapter(title.as_ref().unwrap(), &chapter_content);
        write_last_chapter(next_chapter_url.unwrap());
        println!("{}\n{}\n", title.unwrap(), next_chapter_url.unwrap());

        thread::sleep(Duration::from_millis(1000));
    }
}

fn get_last_chapter_url() -> String {
    let mut file = File::open("last_chapter_url.txt").unwrap();
    let mut url = String::new();
    file.read_to_string(&mut url).unwrap();

    url
}

fn write_last_chapter(name: &str) {
    /* let mut file = OpenOptions::new()
        .write(true)
        .append(false)
        .create(true)
        .open("last_chapter_url.txt")
        .unwrap(); */

    fs::write("last_chapter_url.txt", name.as_bytes()).unwrap();
}

fn write_chapter(name: &str, content: &Vec<&str>) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("chapters/".to_string() + name + ".txt")
        .unwrap();

    content.into_iter()
        .for_each(|line| { writeln!(file, "{line}").unwrap(); });
}
