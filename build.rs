use fancy_regex::Regex as Fancy;
use regex::Regex;
use serde::Deserialize;
use std::fmt::Write;
use std::{env, fmt, fs, path::Path};

#[allow(dead_code)]
#[derive(Deserialize)]
struct PatternData {
    name: String,
    regex: String,
    #[serde(skip_deserializing)]
    regex_no_anchor: String,
    plural_name: bool,
    description: Option<&'static str>,
    exploit: Option<String>,
    rarity: f32,
    url: Option<&'static str>,
    tags: Vec<&'static str>,
    #[serde(skip_deserializing)]
    uses_non_standard_regex: bool,
}


impl fmt::Debug for PatternData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PatternData")
            .field("name", &self.name)
            .field("plural_name", &self.plural_name)
            .field("description", &self.description)
            .field("exploit", &self.exploit)
            .field("rarity", &self.rarity)
            .field("url", &self.url)
            .field("tags", &self.tags)
            // .field("uses_non_standard_regex", &self.uses_non_standard_regex) TODO: use it
            .finish()
    }
}

fn main() {
    let mut data: Vec<PatternData> = serde_json::from_str(include_str!("./data/regex.json")).unwrap();

    data.iter_mut().for_each(|d| {
        d.regex_no_anchor = Fancy::new(r"(?<!\\)\^(?![^\[\]]*(?<!\\)\])")
            .expect("can't compile for regex_no_anchor")
            .replace(&d.regex, "")
            .to_string();
        d.regex_no_anchor = Fancy::new(r"(?<!\\)\$(?![^\[\]]*(?<!\\)\])")
            .expect("can't compile for regex_no_anchor")
            .replace(&d.regex_no_anchor, "")
            .to_string();
        match Regex::new(&d.regex) {
            Ok(_) => {
                d.uses_non_standard_regex = false;
            }
            Err(_) => {
                d.uses_non_standard_regex = true;
            }
        }
    });

    data.sort_by(|a, b| {
        a.name.cmp(&b.name)
    });

    // TODO support non standard regex, currently crashes sometimes if we dont filter
    data.retain(|r| Regex::new(&r.regex).is_ok() && Regex::new(&r.regex_no_anchor).is_ok());

    let mut data_str = format!("{:?}", data);
    // we want reference to [], i.e. &[]
    data_str = data_str.replace("tags: [", "tags: &[");

    let regex_str: String = data.iter().fold(String::new(), |mut output, d| {
        let _ = write!(
            output,
            "\tLazy::new(|| Regex::new({:?}).unwrap()),\n",
            d.regex
        );
        output
    });

    let regex_no_anchor_str: String = data.iter().fold(String::new(), |mut output, d| {
        let _ = write!(
            output,
            "\tLazy::new(|| Regex::new({:?}).unwrap()),\n",
            d.regex_no_anchor
        );
        output
    });

    let count = data.len();
    let final_str = format!(
        "pub const PATTERN_DATA: [PatternData; {count}] = {data_str};"
    );
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("pattern_data.rs");
    fs::write(dest_path, final_str).unwrap();

    let mut final_str = format!(
        "pub static REGEX: [Lazy<Regex>; {count}] = [\n{regex_str}];\n"
    );
    final_str += "\n";
    final_str += format!(
        "pub static REGEX_NO_ANCHOR: [Lazy<Regex>; {count}] = [\n{regex_no_anchor_str}];"
    ).as_str();
    let regex_dest_path = Path::new(&out_dir).join("regex_data.rs");
    fs::write(regex_dest_path, final_str).unwrap();
}
