use crate::proto::*;
use std::fs;
use std::ops::Bound;
use std::path::Path;
use tantivy::query::{Occur, PhraseQuery, Query, RangeQuery};
use tantivy::schema::{Field, INDEXED, STORED, STRING, Schema, TEXT, Value};
use tantivy::{Index, IndexReader, TantivyDocument, Term};

pub const INDEX_PATH: &str = "./data/tantivy_index";

pub struct ImdbSearcher {
    pub index: Index,
    pub reader: IndexReader,
    pub title: Field,
    pub category: Field,
    pub year: Field,
    pub imdb_id: Field
}

fn get_text(doc: &TantivyDocument, field: Field) -> Option<String> {
    match doc.get_first(field)?.as_value().as_str() {
        Some(s) => Some(s.to_string()),
        None => None,
    }
}

fn get_i64(doc: &TantivyDocument, field: Field) -> Option<i32> {
    match doc.get_first(field)?.as_value().as_i64() {
        Some(v) => Some(v as i32),
        None => None,
    }
}

pub fn build_schema() -> Schema {
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("category", STRING | STORED);
    schema_builder.add_i64_field("year", INDEXED | STORED);
    schema_builder.add_text_field("imdb_id", STRING | STORED);
    schema_builder.build()
}

impl ImdbSearcher {
    pub fn new() -> tantivy::Result<Self> {
        let index_path = Path::new(INDEX_PATH);
        if !index_path.exists() {
            fs::create_dir_all(index_path)?;
            tracing::info!("Creating new Tantivy index at {}", index_path.display());

            let schema = build_schema();
            let _ = Index::create_in_dir(index_path, schema)?;
            tracing::info!("Initialized empty Tantivy index");
        }

        let index = Index::open_in_dir(index_path)?;
        let schema = index.schema(); // <-- load the stored schema, not a new one

        let imdb_id = schema.get_field("imdb_id")?;
        let title = schema.get_field("title")?;
        let category = schema.get_field("category")?;
        let year = schema.get_field("year")?;

        let reader = index.reader()?;

        Ok(Self {
            index,
            reader,
            title,
            category,
            year,
            imdb_id
        })
    }

    pub fn drop_and_initialise_index(&mut self) -> tantivy::Result<()> {
        tracing::info!("Dropping and re-initialising Tantivy index");
        let index_path = Path::new(INDEX_PATH);
        if index_path.exists() {
            fs::remove_dir_all(index_path)?;
        }
        fs::create_dir_all(index_path)?;
        let schema = build_schema();
        self.index = Index::create_in_dir(index_path, schema)?;
        self.reader = self.index.reader()?;
        self.imdb_id = self.index.schema().get_field("imdb_id")?;
        self.title = self.index.schema().get_field("title")?;
        self.category = self.index.schema().get_field("category")?;
        self.year = self.index.schema().get_field("year")?;
        Ok(())
    }

    pub fn search(&self, title: &str, category: &str, year: i32) -> Vec<Match> {
        let searcher = self.reader.searcher();

        let mut clauses = vec![];

        let title_terms: Vec<Term> = title
            .split_whitespace()
            .map(|word| Term::from_field_text(self.title, word))
            .collect();

        if title_terms.len() >= 2 {
            clauses.push((
                Occur::Must,
                Box::new(PhraseQuery::new(title_terms)) as Box<dyn Query>,
            ));
        } else if title_terms.len() == 1 {
            clauses.push((
                Occur::Must,
                Box::new(tantivy::query::TermQuery::new(
                    title_terms[0].clone(),
                    tantivy::schema::IndexRecordOption::Basic,
                )) as Box<dyn Query>,
            ));
        } else {
            tracing::warn!("Empty title query received; skipping title clause.");
        }

        clauses.push((
            Occur::Must,
            Box::new(tantivy::query::TermQuery::new(
                Term::from_field_text(self.category, category),
                tantivy::schema::IndexRecordOption::Basic,
            )) as Box<dyn Query>,
        ));

        if year > 0 {
            clauses.push((
                Occur::Should,
                Box::new(RangeQuery::new(
                    Bound::Included(Term::from_field_i64(self.year, year as i64 - 1)),
                    Bound::Included(Term::from_field_i64(self.year, year as i64 + 1)),
                )) as Box<dyn Query>,
            ));
        }

        let query = tantivy::query::BooleanQuery::new(clauses);

        let top_docs = searcher
            .search(&query, &tantivy::collector::TopDocs::with_limit(5))
            .unwrap_or_default();

        top_docs
            .into_iter()
            .filter_map(|(_, addr)| {
                let retrieved: TantivyDocument = searcher.doc(addr).ok()?;
                Some(Match {
                    imdb_id: get_text(&retrieved, self.imdb_id)?,
                    title: get_text(&retrieved, self.title)?,
                    year: get_i64(&retrieved, self.year).unwrap_or(0),
                    score: 1.0,
                })
            })
            .collect()
    }
}
