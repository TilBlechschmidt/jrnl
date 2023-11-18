use tower_http::services::{ServeDir, ServeFile};

pub fn service() -> ServeDir<ServeFile> {
    ServeDir::new("./frontend/build")
        .append_index_html_on_directories(true)
        .fallback(ServeFile::new("./frontend/build/index.html"))
}
