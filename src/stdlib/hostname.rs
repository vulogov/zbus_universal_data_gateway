extern crate hostname;

pub fn get_hostname() -> String {
    match hostname::get() {
        Ok(h) => {
            match h.into_string() {
                Ok(hs) => return hs,
                Err(_) => return "localhost".to_string(),
            }
        }
        Err(_) => return "localhost".to_string(),
    }
}
