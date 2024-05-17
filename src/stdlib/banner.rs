use neofiglet::FIGfont;

pub fn banner(s: &String) -> String {
    let standard_font = FIGfont::standard().unwrap();
    let figure = standard_font.convert(&s);
    figure.unwrap().to_string()
}

pub fn bund_banner() -> String {
    let ban = format!("ZBUSDG {}", env!("CARGO_PKG_VERSION"));
    banner(&ban)
}
