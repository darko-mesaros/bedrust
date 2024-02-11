use figlet_rs::FIGfont;

pub fn hello_header(s: &str) {

    let standard_font = FIGfont::standard().unwrap();
    let ansi_font = FIGfont::from_file("resources/ansishadow.flf").unwrap();
    let hello = ansi_font.convert(s);
    println!("{}", hello.unwrap());

}
