
use rusqlite::{Connection};
use std::{path::{Path, PathBuf}, any::Any};
use crate::error::{BSResult, BSError};
use crossbeam_channel::{Sender, Receiver, unbounded};

pub mod migrations;

pub struct StatisticsRepository {
  pub connection: Connection,
}

impl StatisticsRepository {
  /// Instantiates repo and opens the SQLite database at the given `db_path`
  pub fn new(db_file: &Path) -> BSResult<StatisticsRepository> {
    let connection = Connection::open(db_file)?;
    Ok(StatisticsRepository { connection })
  }

  pub fn select(&mut self, item: StatisticsEntity) -> BSResult<()> {
    let mut query = r#"
      SELECT * FROM history_selections 
      INNER JOIN selections ON history_selections.selection_id = selections.id 
      WHERE 
    "#.to_string();
    let criteria = [
      ()
    ]


    let stmt = self.connection.prepare(query)?;
    stmt.execute(params)
  }

  // pub fn save(&mut self, item: StatisticsEntity) -> BSResult<()> {
  //   self.connection.prepare("SELECT * FROM history_selections INNER JOIN selections ON history_selections.selection_id = selections.id WHERE ")
  // }
}

#[derive(Default)]
pub struct StatisticsEntity {
  browser_path_hash: Option<u64>,
  base_domain_hash: Option<u64>,
  browser_path: Option<String>,
  hour: Option<u16>,
  weekday: Option<u16>,
  fqdn: Option<[char; 255]>,
  uri: Option<[char; 255]>,
  src: Option<[char; 255]>,
}

impl StatisticsEntity {
  fn make_query_where_clause(&self) -> (String, (&str, &dyn Any)) {
    let sql = "".to_string();
    if let Some(browser_path_hash) = self.base_domain_hash {
      sql += "browser_path_hash = ?"
    }
  }
}

/// All results from Statistics worker operations are served through
/// this structure. Its member fields will be populated based on the 
/// type of request.
pub struct StatisticsResult {
  /// The initiating message for the operation that completed
  /// sucessfully.
  op_msg: StatisticsWorkerMsg,

  /// The resutlting data when querying.
  entity: Option<StatisticsEntity>,
}

pub enum StatisticsWorkerMsg {
  Setup,
  Predict(StatisticsEntity),
  Save(StatisticsEntity),
  Sucess(Box<StatisticsResult>),
  Error(BSError),
  Quit,
}

pub struct StatisticsWorker<'a> {
  sender_main: Sender<StatisticsWorkerMsg>,
  receiver_main: Receiver<StatisticsWorkerMsg>,
  error_cb: &'a ErrorCallback,
  sucess_cb: &'a SucessCallback,
}

pub type ErrorCallback = dyn Fn(BSError) -> ();
pub type SucessCallback = dyn Fn(Box<StatisticsResult>) -> ();

impl<'a> StatisticsWorker<'a> {
  pub fn new(db: &Path) -> StatisticsWorker {
    let (sender_main, receiver_main) = unbounded::<StatisticsWorkerMsg>();
    let receiver_worker = receiver_main.clone();
    let sender_worker = sender_main.clone();

    let db_path = std::path::PathBuf::from(db);
    std::thread::spawn(move || { StatisticsWorker::worker_main(db_path, receiver_worker, sender_worker) });
    sender_main.send(StatisticsWorkerMsg::Setup).unwrap();

    StatisticsWorker { sender_main, receiver_main, error_cb: &|_|{}, sucess_cb: &|_|{} }
  }

  pub fn on_error(&mut self, cb: &'a ErrorCallback) {
    self.error_cb = cb;
  }

  pub fn on_success(&mut self, cb: &'a SucessCallback) {
    self.sucess_cb = cb;
  }

  pub fn save_async(&mut self, item: StatisticsEntity) {
    self.sender_main.send(StatisticsWorkerMsg::Save(item)).unwrap()
  }

  pub fn predict_async(&mut self, input: StatisticsEntity) {
    self.sender_main.send(StatisticsWorkerMsg::Predict(input)).unwrap()
  }

  fn worker_main(db_path: PathBuf, receiver_worker: Receiver<StatisticsWorkerMsg>, sender_worker: Sender<StatisticsWorkerMsg>) -> BSResult<()> {
    let mut statistics = StatisticsRepository::new(&db_path)?;
    
    loop {
      let op_msg = receiver_worker.recv().unwrap();
      match op_msg {
        StatisticsWorkerMsg::Setup => {
          migrations::migrate(&mut statistics.connection)?;
          sender_worker.send(StatisticsWorkerMsg::Sucess(
            Box::new(StatisticsResult {
              op_msg,
              entity: None,
            })
          )).unwrap();
        },
        StatisticsWorkerMsg::Quit => break,
        _ => ()
      };

      // std::thread::sleep(std::time::Duration::from_millis(1));
    }
    
    Ok(())
  }

  fn process_main_thread_message(&self) {
    match self.receiver_main.try_recv().unwrap() {
      StatisticsWorkerMsg::Sucess(result) => (self.sucess_cb)(result),
      StatisticsWorkerMsg::Error(err) => (self.error_cb)(err),
      _ => return
    }
  }
}

 impl<'a> Drop for StatisticsWorker<'a> {
  fn drop(&mut self) {
      self.sender_main.send(StatisticsWorkerMsg::Quit).unwrap()
  }
 }