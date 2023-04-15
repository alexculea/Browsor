use crate::error::{BSError, BSResult};
use chrono::prelude::*;
use chrono::Datelike;
use rusqlite::Connection;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_rusqlite::{columns_from_statement, from_row_with_columns};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use url::Url;

static TLD_SUFFIXES: &str = include_str!("../../assets/data/tld-suffixes.dat");

#[derive(Deserialize, Serialize)]
pub struct StatisticsEntity {
    pub id: Option<usize>,
    pub url: Option<String>,
    pub src: Option<String>,
    pub tld: Option<String>,
    pub weekday: Option<u16>,
    pub hour: Option<u16>,
    pub selection_id: Option<usize>,
    // pub date: Option<TODO>
    pub count: Option<usize>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SelectionEntity {
    pub id: Option<usize>,
    pub path_hash: Option<String>,
    pub path: Option<String>,
}

#[derive(Clone)]
pub struct StatisticsRepository {
    db_path: Option<PathBuf>,
}

impl StatisticsRepository {
    pub fn new() -> StatisticsRepository {
        StatisticsRepository { db_path: None }
    }

    pub fn set_db_path(&mut self, db_file: &Path) {
        self.db_path = Some(PathBuf::from(db_file));
    }

    pub fn get_db_path(&self) -> PathBuf {
        if let Some(path) = self.db_path.clone() {
            return path;
        }

        Default::default()
    }

    pub fn select<T: DeserializeOwned + Serialize>(
        &mut self,
        table: &str,
        con: Option<Connection>,
    ) -> BSResult<Vec<T>> {
        let con = self.open_conn(con)?;
        let query = format!("SELECT * FROM {}", table);
        let mut stmt = con.prepare(&query)?;
        let cols = columns_from_statement(&stmt);
        let res = stmt.query_and_then((), |row| from_row_with_columns::<T>(row, &cols))?;

        let list: Vec<T> = res.map(|item| item.unwrap()).collect();
        Ok(list)
    }

    pub fn select_by_params<'a, T: DeserializeOwned + Serialize>(
        &mut self,
        table: &str,
        params_entity: &T,
        existing_conn: Option<Connection>,
    ) -> BSResult<Vec<T>> {
        if let serde_json::Value::Object(record) = serde_json::to_value(params_entity).unwrap() {
            let keys: Vec<_> = record
                .keys()
                .filter(|key| !record.get(*key).unwrap().is_null())
                .collect();
            let sql_keys = keys
                .clone()
                .into_iter()
                .map(|key| format!("{} = :{}", key, key))
                .collect::<Vec<String>>()
                .join(",");
            let query = format!(r#"SELECT * FROM {table} WHERE {sql_keys}"#);
            let conn = self.open_conn(existing_conn)?;
            let mut stmt = conn.prepare(&query)?;
            let cols = columns_from_statement(&stmt);

            let fields = keys
                .clone()
                .into_iter()
                .map(|item| item.as_str())
                .collect::<Vec<&str>>();
            let params =
                serde_rusqlite::to_params_named_with_fields(params_entity, fields.as_slice())
                    .unwrap();
            let res = stmt.query_and_then(params.to_slice().as_slice(), |row| {
                from_row_with_columns::<T>(row, &cols)
            })?;

            let list: Vec<T> = res.map(|item| item.unwrap()).collect();
            Ok(list)
        } else {
            Err(BSError::new(
                "Expected an object for making up the query WHERE clause.",
            ))
        }
    }

    pub fn get_selection_from_browser(
        &mut self,
        browser_path_hash: &str,
        existing_conn: Option<Connection>,
    ) -> BSResult<Option<SelectionEntity>> {
        let params = SelectionEntity {
            id: None,
            path_hash: Some(browser_path_hash.into()),
            path: None,
        };
        let mut result = self.select_by_params("selections", &params, existing_conn)?;
        if result.len() > 0 {
            Ok(Some(result.remove(0)))
        } else {
            Ok(None)
        }
    }

    pub fn update_selections(&mut self, mut list: Vec<SelectionEntity>) -> BSResult<()> {
        let conn = self.open_conn(None)?;
        let query =
            "INSERT INTO selections (path_hash, path) VALUES(?1, ?2) ON CONFLICT(path) DO UPDATE SET path_hash=?1;";
        let mut stmt = conn.prepare(query)?;
        let _results: Vec<Result<usize, rusqlite::Error>> = list
            .drain(0..)
            .map(|entry| stmt.execute((entry.path_hash, entry.path)))
            .collect();

        Ok(())
    }

    pub fn save_choice(
        &mut self,
        source: Option<PathBuf>,
        url: &str,
        browser_path_hash: &str,
        _browser_path: &str,
    ) -> BSResult<()> {
        let conn = self.open_conn(None)?;
        let selection_opt = self.get_selection_from_browser(browser_path_hash, None)?;

        if let Some(selection) = selection_opt {
            let query = r#"INSERT INTO selections_history (url, src, tld, weekday, hour, selection_id)
            VALUES(?, ?, ?, ?, ?, ?);"#;
            let mut stmt = conn.prepare(query)?;
            let src_path = source.unwrap_or_default();
            let src = src_path.to_string_lossy();
            let local: DateTime<Local> = Local::now();
            let weekday = local.weekday().number_from_monday();
            let hour = local.hour();
            let dns_tld: String = Self::find_tld_from_url(url)?;
            stmt.execute((url, src, dns_tld, weekday, hour, selection.id.unwrap()))?;

            Ok(())
        } else {
            Err(format!(
                "Couldn't find selection by path_hash {} to save the choice.",
                browser_path_hash,
            )
            .as_str()
            .into())
        }
    }

    pub fn predict(
        &mut self,
        source: Option<PathBuf>,
        url: &str,
    ) -> BSResult<Vec<SelectionEntity>> {
        // TODO: quick and dirty MVP POC, refactor asap
        let choices: Vec<SelectionEntity> = self.select("selections", None)?;
        if choices.len() == 0 {
            bail!("Selections table is empty.");
        }
        let src_path = source.unwrap_or_default();
        let src = src_path.to_string_lossy();
        let local: DateTime<Local> = Local::now();
        let weekday = local.weekday().number_from_monday();
        let hour = local.hour();
        let dns_tld: String = Self::find_tld_from_url(url)?;
        let max_age = local
            .checked_sub_months(chrono::Months::new(1))
            .unwrap()
            .timestamp();
        let mut choice_map =
            choices
                .iter()
                .fold(BTreeMap::<usize, (u32, f64)>::new(), |mut map, item| {
                    map.insert(item.id.unwrap(), (0, 0.0));
                    map
                });
        let con = self.open_conn(None)?;
        let query = r#"SELECT id, url, src, tld, weekday, hour, selection_id, date, count
            FROM selections_history
            WHERE date > ?"#;
        let mut stmt = con.prepare(&query)?;
        let cols = columns_from_statement(&stmt);
        let rows = stmt.query_and_then(params!(max_age), |row| {
            from_row_with_columns::<StatisticsEntity>(row, &cols)
        })?;

        let weights = [0.5, 3.0, 2.0, 2.0, 3.0];

        for entry in rows {
            if let Ok(stat_entity) = entry {
                let (mut choice_count, mut choice_score) =
                    choice_map.get(&stat_entity.selection_id.unwrap()).unwrap();

                let factors_counted: i8 = 5;
                let mut entry_sum: f64 = 0.0;
                if let Some(entity_url) = stat_entity.url {
                    let factor_score = if url == entity_url { 1.0 * weights[0] } else { 0.0 };
                    entry_sum += factor_score;
                }

                if let Some(entity_src) = stat_entity.src {
                    let factor_score = if entity_src == src { 1.0 * weights[1] } else { 0.0 };
                    entry_sum += factor_score;
                }

                if let Some(entity_weekday) = stat_entity.weekday {
                    let factor_score = if (entity_weekday as i16) - (weekday as i16) == 0 {
                        1.0 * weights[2]
                    } else {
                        0.0
                    };
                    entry_sum += factor_score;
                }

                if let Some(entity_hour) = stat_entity.hour {
                    let factor_score = if (entity_hour as i16) - (hour as i16) == 0 {
                        1.0 * weights[3]
                    } else {
                        0.0
                    };
                    entry_sum += factor_score;

                }

                if let Some(entity_tld) = stat_entity.tld {
                    let factor_score = if entity_tld == dns_tld { 1.0 * weights[4] } else { 0.0 };
                    entry_sum = entry_sum + factor_score;
                }

                
                let entry_score = entry_sum / factors_counted as f64;
                choice_score += entry_score;
                choice_count += 1;
                choice_map.insert(
                    stat_entity.selection_id.unwrap(),
                    (choice_count, choice_score),
                );
            }
        }

        let choices_scores = choice_map.into_iter().fold(
            BTreeMap::<i64, usize>::new(),
            |mut map, (choice_id, (choice_count, choice_score))| {
                let final_score = (choice_score * 10.0) + choice_count as f64;
                map.insert(final_score as i64 , choice_id);

                println!("Choice: {}, count: {}, score: {}, final score: {}", choice_id, choice_count, choice_score, final_score);
                map
            },
        );

        let mut choices_sorted = choices_scores
            .iter()
            .map(|(_, choice_id)| -> SelectionEntity {
                let choice = choices
                    .iter()
                    .find(|item| item.id.unwrap() == *choice_id)
                    .unwrap();
                
                choice.clone()
            })
            .collect::<Vec<SelectionEntity>>();
        choices_sorted.reverse();

        Ok(choices_sorted)
    }

    fn open_conn(&self, exiting_conn: Option<Connection>) -> Result<Connection, rusqlite::Error> {
        if exiting_conn.is_some() {
            return Ok(exiting_conn.unwrap());
        } else {
            Connection::open(&self.db_path.as_ref().expect("DB Path should be set"))
        }
    }

    fn find_tld_from_url(url: &str) -> BSResult<String> {
        // TODO: Return None instead of String::default() when hostname is not present
        // TODO: Potentially expensive computationally, optimize
        use url::Host::Domain;

        let url_res = Url::parse(url); // TODO: Don't panic on invalid URLs
        if let Err(err) = url_res {
            return Err(BSError::from(err.to_string().as_str()))
        }

        let url = url_res.unwrap();
        let hostname: &str = if let Some(Domain(host)) = url.host() {
            host
        } else {
            // TODO Handle no host separately
            return Ok(String::from(url.host_str().unwrap()));
        };
        let hostname_parts: Vec<&str> = hostname.split('.').collect();
        let mut part_idx = hostname_parts.len() - 1;
        let mut needle = String::from(hostname_parts[part_idx]);
        while let Some(needle_index) = TLD_SUFFIXES.find(&needle) {
            if needle_index > 0 {
                if TLD_SUFFIXES.chars().nth(needle_index - 1).unwrap() != '\n' {
                    break;
                }
            }
            part_idx -= 1;
            needle = format!("{}.{}", hostname_parts[part_idx], needle);
        }

        let tld_parts: Vec<&str> = hostname_parts
            .iter()
            .skip(part_idx)
            .map(|item| *item)
            .collect();
        Ok(tld_parts.join("."))
    }
}
