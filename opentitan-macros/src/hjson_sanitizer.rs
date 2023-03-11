pub fn sanitize(s: String) -> String {
    let r1 = regex::Regex::new(r"'''([^\n]*)'''").unwrap();
    let s = r1.replace(&s, "'''$1\n'''");
    s.into_owned()
}
