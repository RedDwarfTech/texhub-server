use crate::service::global::proj::proj_util::{get_proj_base_dir, get_proj_download_base_dir};
use rust_wheel::common::util::rd_file_util::{create_folder_not_exists, join_paths};
use std::{fs::File, path::Path};
use zip::{write::FileOptions, ZipWriter};

pub fn gen_zip(proj_id: &String) -> String {
    let proj_base_dir = get_proj_base_dir(proj_id);
    let folder_path = Path::new(proj_base_dir.as_str());
    let download_dir = get_proj_download_base_dir(proj_id);
    create_folder_not_exists(&download_dir);
    let archive_file_path = join_paths(&[download_dir, "archive.zip".to_string()]);
    let file = File::create(archive_file_path.clone()).unwrap();
    let mut zip = ZipWriter::new(file);
    visit_folder(folder_path, &mut zip, "").unwrap();
    zip.finish().unwrap();
    return archive_file_path;
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
            let options: FileOptions<'_, ()>  = FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(0o755);
            zip.start_file(zip_path, options)?;
            std::io::copy(&mut file, zip)?;
        }
    }
    Ok(())
}
