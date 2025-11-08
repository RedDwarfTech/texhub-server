use std::{
    fs::{self, File},
    io,
    path::PathBuf,
};

use log::error;

pub fn exact_upload_zip(input_path: &str, output_path: &str) -> Result<(), io::Error> {
    let file = File::open(&input_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    /*
    check the decompress size before decompress the file
    this action is necessary because we need to avoid the compress bomb
    more information we can know from here: https://en.wikipedia.org/wiki/Zip_bomb
     */
    if archive.decompressed_size().unwrap_or_default() > 200 * 1024 * 1024 {
        return Err(io::Error::new(io::ErrorKind::Other, "too huge for exact"));
    }
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let out_path = format!("{}/{}", output_path, file.name());
        let outpath = PathBuf::from(out_path);
        let comment = file.comment();
        if !comment.is_empty() {}
        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).map_err(|e| {
                error!("Could not create dir,{}", e);
                e
            })?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p).map_err(|e| {
                        error!(
                            "create parent dir failed,err:{},path:{}",
                            e,
                            &p.to_string_lossy()
                        );
                        e
                    })?;
                }
            }
            let mut outfile = File::create(&outpath).map_err(|e| {
                error!("create out file failed,{}", e);
                e
            })?;
            io::copy(&mut file, &mut outfile).map_err(|e| {
                error!("copy file failed,{}", e);
                e
            })?;
        }
        // Set file permissions if running on a Unix-like system.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
            }
        }
    }
    Ok(())
}
