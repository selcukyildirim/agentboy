pub mod model;
pub mod parser;

pub use model::{DocumentMetadata, DocumentType, ParsedDocument, Section, Table};
pub use parser::{extract_tables_as_text, extract_text};
pub use parser::{
    AutoParser, CsvParser, DocxParser, JsonParser, PdfParser, PlainTextParser, XlsxParser,
    XmlParser,
};
