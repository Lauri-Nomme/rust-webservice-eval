#[macro_use]
extern crate clap;
#[macro_use]
extern crate tower_web;

use std::{fs, io};
use std::fs::DirEntry;
use std::path::PathBuf;

use tokio::fs::File;
use tokio::prelude::Future;
use tower_web::ServiceBuilder;

#[derive(Clone, Debug)]
pub struct BlobResource {
    root_path: PathBuf,
}

impl BlobResource {
    fn new(root_path: PathBuf) -> BlobResource {
        BlobResource {
            root_path
        }
    }

    fn entry_result_to_blobdesc(entry_result: Result<DirEntry, std::io::Error>) -> Option<BlobDesc> {
        let entry = entry_result.ok()?;

        if !entry.file_type().ok()?.is_file() {
            None
        } else {
            Some(BlobDesc {
                name: entry.path().file_name()?.to_str()?.to_string(),
                size: entry.metadata().ok()?.len(),
            })
        }
    }

    fn list_blobs(&self) -> Result<impl Iterator<Item=BlobDesc>, std::io::Error> {
        fs::read_dir(&self.root_path)
            .map(|dir| dir.filter_map(Self::entry_result_to_blobdesc))
    }
}

#[derive(Extract, Debug)]
struct PathRequest {
    name: PathBuf,
}

#[derive(Serialize)]
struct BlobDesc {
    name: String,
    size: u64,
}

impl_web! {
    impl BlobResource {
        #[get("/list")]
        #[content_type("application/json")]
        fn list(&self) -> Result<Vec<BlobDesc>, std::io::Error> {
            self.list_blobs().map(|x| x.collect())
        }

        #[get("/file")]
        fn file(&self, query_string: PathRequest) -> impl Future<Item = File, Error = io::Error> {
            let mut path = PathBuf::from(&self.root_path);
            path.push(query_string.name);
            File::open(path)
        }
    }
}

pub fn main() {
    let args = clap_app!(myapp =>
        (@arg listen: -l --listen +takes_value "Endpoint to Listen on")
        (@arg path: -p --path +takes_value +required "Path to serve")
    ).get_matches();

    let path = args.value_of("path").unwrap();
    let listen_addr = args.value_of("listen").unwrap_or("127.0.0.1:8888").parse().expect("Invalid listen address");

    println!("Listening on http://{}, serving {}", listen_addr, path);

    ServiceBuilder::new()
        .resource(BlobResource::new(path.into()))
        .run(&listen_addr)
        .unwrap();
}