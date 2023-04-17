pub mod migrations;
pub mod statistics_repository;
pub mod thread_worker;

pub use self::statistics_repository::{SelectionEntity, StatisticsRepository};
use self::thread_worker::ThreadWorker;
use crate::error::BSResult;
use std::path::{Path, PathBuf};

pub struct Statistics {
    repo: StatisticsRepository,
    worker: Option<ThreadWorker>,
}

impl Statistics {
    pub fn new() -> Self {
        Self {
            repo: StatisticsRepository::new(),
            worker: None,
        }
    }

    pub fn set_db_path(&mut self, path: &Path) {
        self.repo.set_db_path(path);
    }

    pub fn migrate_async(&mut self, result_cb: impl Fn(Box<BSResult<()>>) + 'static) {
        let db_path = self.repo.get_db_path();
        self.get_worker().run_async(
            move || {
                let mut conn = rusqlite::Connection::open(db_path).expect(format!("Unable to open DB.").as_str());
                migrations::migrate(&mut conn)
            },
            Self::unwrap_result_callback(result_cb),
        );
    }

    pub fn update_selections(
        &mut self,
        list: Vec<SelectionEntity>,
        result_cb: impl Fn(Box<BSResult<()>>) + 'static,
    ) {
        let mut repo_clone = self.repo.clone();
        self.get_worker().run_async(
            move || repo_clone.update_selections(list),
            Self::unwrap_result_callback(result_cb),
        );
    }

    pub fn save_choice(
        &mut self,
        source: Option<PathBuf>,
        url: &str,
        browser_path_hash: &str,
        browser_path: &str,
        result_cb: impl Fn(Box<BSResult<()>>) + 'static,
    ) {
        let mut repo_clone = self.repo.clone();
        let url_s = String::from(url);
        let browser_path_hash_str = String::from(browser_path_hash);
        let browser_path_str = String::from(browser_path);
        self.get_worker().run_async(
            move || {
                repo_clone.save_choice(source, &url_s, &browser_path_hash_str, &browser_path_str)
            },
            Self::unwrap_result_callback(result_cb),
        );
    }

    
    pub fn predict(
        &mut self,
        source: Option<PathBuf>,
        url: &str,
        result_cb: impl Fn(Box<BSResult<Vec<SelectionEntity>>>) + 'static,
    ) {
        let mut repo_clone = self.repo.clone();
        let url_s = String::from(url);
        self.get_worker().run_async(
            move || {
                repo_clone.predict(source, &url_s)
            },
            Self::unwrap_result_callback(result_cb),
        );
    }

    pub fn tick(&mut self) -> bool {
        self.get_worker().tick()
    }

    fn unwrap_result_callback<T: 'static>(
        result_cb: impl Fn(Box<T>) + 'static,
    ) -> impl Fn(Box<dyn std::any::Any + Send>) + 'static {
        move |incoming: Box<dyn std::any::Any + Send>| {
            if let Ok(res) = incoming.downcast::<T>() {
                result_cb(res);
            } else {
                panic!("Type mismatch for result from worker.");
            }
        }
    }

    fn get_worker(&mut self) -> &mut ThreadWorker {
        if self.worker.is_none() {
            self.worker = Some(ThreadWorker::new(|| {}));
        }

        self.worker.as_mut().unwrap()
    }

    pub fn is_finished(&self) -> bool {
        if self.worker.is_some() {
            self.worker.as_ref().unwrap().is_finished()
        } else {
            true
        }
    }

    pub fn stop(&mut self) {
        if let Some(ref mut worker) = self.worker.as_mut() {
            worker.stop();
        }
    }
}
