fn translit_char(c: char) -> Option<&'static str> {
    Some(match c {
        'а' => "a",
        'б' => "b",
        'в' => "v",
        'г' => "g",
        'д' => "d",
        'е' => "e",
        'ё' => "e",
        'ж' => "zh",
        'з' => "z",
        'и' => "i",
        'й' => "y",
        'к' => "k",
        'л' => "l",
        'м' => "m",
        'н' => "n",
        'о' => "o",
        'п' => "p",
        'р' => "r",
        'с' => "s",
        'т' => "t",
        'у' => "u",
        'ф' => "f",
        'х' => "kh",
        'ц' => "ts",
        'ч' => "ch",
        'ш' => "sh",
        'щ' => "shch",
        'ъ' => "",
        'ы' => "y",
        'ь' => "",
        'э' => "e",
        'ю' => "yu",
        'я' => "ya",
        _ => return None,
    })
}

pub fn slugify(title: &str) -> String {
    let mut out = String::new();
    for c in title.to_lowercase().chars() {
        let mapped = match translit_char(c) {
            Some(s) => s,
            None => {
                if c.is_ascii_alphanumeric() {
                    out.push(c);
                    continue;
                }
                "-"
            }
        };
        out.push_str(mapped);
    }
    let mut slug = String::new();
    let mut prev_hyphen = true;
    for c in out.chars() {
        if c == '-' {
            if !prev_hyphen {
                slug.push('-');
                prev_hyphen = true;
            }
        } else {
            slug.push(c);
            prev_hyphen = false;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "workout".to_string()
    } else {
        slug
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugifies_ascii_titles() {
        assert_eq!(slugify("6x3min steady"), "6x3min-steady");
        assert_eq!(slugify("September park race"), "september-park-race");
        assert_eq!(slugify("Easy run!"), "easy-run");
    }

    #[test]
    fn slugifies_cyrillic_titles() {
        assert_eq!(slugify("Лёгкий бег"), "legkiy-beg");
        assert_eq!(slugify("Разминка 2 км"), "razminka-2-km");
    }

    #[test]
    fn slug_falls_back_when_empty() {
        assert_eq!(slugify("---"), "workout");
        assert_eq!(slugify("!!!"), "workout");
    }
}
