use std::borrow::Cow ;

fn my_normalize(s: &str) ->Cow<'_, str> {
    if s.contains("\n") {
        Cow::Owned(s.replace("\n", ""))
    } else {
        Cow::Borrowed(s)
    }
}
fn main() {
    println!("{}", my_normalize("Hello, world!")) ; // Out: Hello, world!
    println!("{}", my_normalize("Hello\nworld!")) ; // Out: Helloworld!
}
