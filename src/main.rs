use std::env;
use std::path::Path;
use std::io::{self, Error, ErrorKind};
use std::fs::File;
use tar::Builder;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> io::Result<()> {
    // Get environmental variables
    let _source_dir = env::var("SOURCE_DIR").expect("SOURCE_DIR not set");
    let _dest_dir = env::var("DEST_DIR").expect("DEST_DIR not set");
    let _retention_period_hours = env::var("RETENTION_PERIOD_HOURS").expect("RETENTION_PERIOD_HOURS not set")
        .parse::<u64>().expect("Invalid RETENTION_PERIOD_HOURS");

    // Create filename with date and time
    //let date_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    let _date_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
    //let filename = format!("backup_{}.tar.gz", date_time.as_secs());

    let filename = format!(
        "backup-{}.tar.gz",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    // Create tarball of source directory
    let _archive = create_archive(filename);

    // Delete old tarballs
    let _deletions = delete_old_tars();

    Ok(())
}

fn create_archive(filename: String) -> Result<(), Error> {
    // Get the source and target directories from environment variables
    let src_dir = match env::var("SOURCE_DIR") {
        Ok(val) => val,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Other,
                "Unable to get source directory from environment variable",
            ))
        }
    };
    let target_dir = match env::var("DEST_DIR") {
        Ok(val) => val,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::Other,
                "Unable to get target directory from environment variable",
            ))
        }
    };

    // Check that the source directory exists
    let src_path = Path::new(&src_dir);
    if !src_path.exists() {
        return Err(Error::new(
            ErrorKind::Other,
            "Source directory does not exist",
        ));
    }

    // Create the archive file
    // let file_name = format!("{}.tar.gz", filename);
    let file_path = Path::new(&target_dir).join(filename);
    let file = File::create(file_path)?;
    let enc = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let mut tar = Builder::new(enc);

    // Add the files in the source directory to the archive
    tar.append_dir_all("", &src_path)?;

    Ok(())
}

fn delete_old_tars() -> Result<(), Error> {
    let dir = env::var("DEST_DIR").unwrap_or_else(|_| panic!("DEST_DIR is not set"));
    let retention_period_hours = env::var("RETENTION_PERIOD_HOURS")
        .unwrap_or_else(|_| panic!("RETENTION_PERIOD_HOURS is not set"))
        .parse::<u64>()
        .unwrap_or_else(|_| panic!("RETENTION_PERIOD_HOURS is not a valid number"));

    let retention_period_secs = retention_period_hours * 3600;
    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let entries = fs::read_dir(dir).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() && path.extension().unwrap_or_default() == "gz" {
            let metadata = fs::metadata(&path).unwrap();
            let modified_time = metadata.modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_secs();
            if current_time - modified_time > retention_period_secs {
                fs::remove_file(path).unwrap();
            }
        }
    }

    Ok(())
}