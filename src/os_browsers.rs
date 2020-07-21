use simple_error::{SimpleResult as Result};

#[derive(Debug)]
#[derive(Clone)]
pub struct Browser {
  pub exe_path: String,
  pub name: String,
  pub version: String,
}

impl Default for Browser {
  fn default() -> Browser { 
    Browser {
      exe_path: "".into(),
      name: "".into(),
      version: "".into(),
    }
  }
}

pub fn read_system_browsers_sync() -> Result<Vec<Browser>> {
  let error = false;

  if error {
    bail!("Error.");
  } 
  
  return Ok(vec![
    Browser { exe_path : "".into(), name: "Test 1".into(), version: "1.0".into()},
    Browser { exe_path : "".into(), name: "Test 2".into(), version: "1.0".into()},
    Browser { exe_path : "".into(), name: "Test 3".into(), version: "1.0".into()}
  ]);
}

pub fn open_url(url: &String, browser: &Browser) {
  println!("URL Open requested with {:?}\nUrl: {}", browser, url);
}
