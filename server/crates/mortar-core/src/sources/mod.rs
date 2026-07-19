//! Ingestion. Each submodule reads one upstream (the Bluesky AppView,
//! plc.directory, standard.site repos, Streamplace) and maps it into bricks;
//! `fetch` is the seam the rest of the crate consumes, one fetch-and-cache
//! function per source. The core cache and persistence structs take the yield
//! types re-exported here, never a submodule directly, so swapping an
//! ingestion backend stays inside this directory.

pub mod bluesky;
pub mod fetch;
pub mod pds;
pub mod standardsite;
pub mod streamplace;
pub mod util;

pub use bluesky::{AuthorYield, Follow};
pub use standardsite::StdDocs;
pub use streamplace::LiveStream;
