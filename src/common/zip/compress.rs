use crate::service::global::proj::proj_util::{get_proj_base_dir, get_proj_download_base_dir};
use log::{info, warn};
use rust_wheel::common::util::rd_file_util::{create_folder_not_exists, join_paths};
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::path::Path;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Instant, SystemTime};
use zip::{write::FileOptions, ZipWriter};

static PROJECT_ZIP_LOCKS: LazyLock<Mutex<HashMap<String, Arc<Mutex<()>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn project_zip_lock(project_id: &str) -> Arc<Mutex<()>> {
    let mut locks = PROJECT_ZIP_LOCKS.lock().unwrap();
    locks
        .entry(project_id.to_string())
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}

fn archive_file_path(proj_id: &String) -> String {
    let download_dir = get_proj_download_base_dir(proj_id);
    create_folder_not_exists(&download_dir);
    join_paths(&[download_dir, "archive.zip".to_string()])
}

fn file_mtime(path: &Path) -> io::Result<SystemTime> {
    Ok(std::fs::metadata(path)?.modified()?)
}

fn dir_max_mtime(path: &Path) -> io::Result<SystemTime> {
    let mut max_mtime = file_mtime(path)?;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let child_mtime = dir_max_mtime(&entry.path())?;
            if child_mtime > max_mtime {
                max_mtime = child_mtime;
            }
        }
    }
    Ok(max_mtime)
}

fn is_archive_cache_valid(archive_path: &str, project_dir: &Path) -> bool {
    let archive = Path::new(archive_path);
    if !archive.is_file() {
        return false;
    }
    match (file_mtime(archive), dir_max_mtime(project_dir)) {
        (Ok(archive_mtime), Ok(project_mtime)) => archive_mtime >= project_mtime,
        (Err(e), _) | (_, Err(e)) => {
            warn!(
                "zip cache check failed: archive={}, project_dir={}, err={}",
                archive_path,
                project_dir.display(),
                e
            );
            false
        }
    }
}

pub fn gen_zip(proj_id: &String) -> String {
    let start = Instant::now();
    let project_id = proj_id.clone();
    let lock = project_zip_lock(&project_id);
    let _guard = lock.lock().unwrap();

    let proj_base_dir = get_proj_base_dir(proj_id);
    let folder_path = Path::new(proj_base_dir.as_str());
    let archive_path = archive_file_path(proj_id);

    if is_archive_cache_valid(&archive_path, folder_path) {
        info!(
            "gen_zip cache hit: project_id={}, archive={}, elapsed={:?}",
            project_id, archive_path, start.elapsed()
        );
        return archive_path;
    }

    info!(
        "gen_zip cache miss: project_id={}, rebuilding archive={}",
        project_id, archive_path
    );
    let compress_start = Instant::now();
    build_zip(folder_path, &archive_path);
    info!(
        "gen_zip complete: project_id={}, archive={}, compress_elapsed={:?}, total_elapsed={:?}",
        project_id,
        archive_path,
        compress_start.elapsed(),
        start.elapsed()
    );
    archive_path
}

fn build_zip(folder_path: &Path, archive_file_path: &str) {
    let file = File::create(archive_file_path).unwrap();
    let mut zip = ZipWriter::new(file);
    visit_folder(folder_path, &mut zip, "").unwrap();
    zip.finish().unwrap();
}

fn visit_folder(path: &Path, zip: &mut ZipWriter<File>, parent: &str) -> std::io::Result<()> {
    for entry in path.read_dir()? {
        let entry = entry?;
        let file_path = entry.path();
        let file_name = file_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        if file_path.is_dir() {
            let zip_path = join_paths(&[parent, &path.file_name().unwrap().to_string_lossy()]);
            visit_folder(&file_path, zip, &zip_path)?;
        } else {
            let zip_path = join_paths(&[
                parent,
                &path.file_name().unwrap().to_string_lossy(),
                file_name.as_str(),
            ]);
            let mut file = File::open(&file_path)?;
            let options: FileOptions<'_, ()> = FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(0o755);
            zip.start_file(zip_path, options)?;
            std::io::copy(&mut file, zip)?;
        }
    }
    Ok(())
}
