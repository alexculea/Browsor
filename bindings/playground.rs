

fn main() {
  use regex::Regex;
  let re = Regex::new(r"pub mod ([A-z0-9_]+);").unwrap();

  for re_match in re.captures_iter(&"pub mod Hello;") {
    let module_name = &re_match[1];
     println!("{:?}", module_name);
    //  println!("{:?}", tree);
  }

}