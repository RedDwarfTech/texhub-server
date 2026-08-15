use std::{
    ffi::{c_int, CStr, CString},
    path::{Path, PathBuf},
};

use crate::common::database::get_connection;
use crate::common::interop::synctex::{
    synctex_display_query, synctex_edit_query, synctex_node_box_visible_depth,
    synctex_node_box_visible_h, synctex_node_box_visible_height, synctex_node_box_visible_v,
    synctex_node_box_visible_width, synctex_node_column, synctex_node_line, synctex_node_p,
    synctex_node_page, synctex_node_tag, synctex_node_visible_h, synctex_node_visible_v,
    synctex_scanner_free, synctex_scanner_get_name, synctex_scanner_new_with_output_file,
    synctex_scanner_next_result,
};
use crate::diesel::ExpressionMethods;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::model::diesel::tex::custom_tex_models::TexFile;
use crate::model::diesel::tex::tex_schema::tex_file as tex_file_table;
use crate::model::request::project::query::{
    get_pdf_pos_params::GetPdfPosParams, get_src_pos_params::GetSrcPosParams,
};
use crate::model::response::project::{pdf_pos_resp::PdfPosResp, src_pos_resp::SrcPosResp};
use crate::service::global::proj::proj_util::{
    get_compile_output_dir_name, get_proj_base_dir, get_proj_compile_workspace_dir,
};
use log::{error, info};
use rust_wheel::common::util::rd_file_util::{get_filename_without_ext, join_paths};

pub fn get_pdf_pos(params: &GetPdfPosParams) -> Vec<PdfPosResp> {
    info!("get pdf pos params:{:?}", params);
    let proj_dir = get_proj_base_dir(&params.project_id);
    let compile_out_dir = join_paths(&[proj_dir.clone(), get_compile_output_dir_name()]);
    let pdf_file_name = format!("{}{}", get_filename_without_ext(&params.main_file), ".pdf");
    let full_pdf_file_path = join_paths(&[&compile_out_dir, &pdf_file_name]);
    unsafe {
        let c_pdf_full_file_path = CString::new(full_pdf_file_path.clone());
        info!("full pdf path:{}", full_pdf_file_path);
        if let Err(e) = c_pdf_full_file_path {
            error!("parse out path error,{},{}", e, full_pdf_file_path);
            return Vec::new();
        }
        let c_build_path = CString::new(compile_out_dir.clone());
        if let Err(e) = c_build_path {
            error!("parse build path error,{},{}", e, compile_out_dir);
            return Vec::new();
        }
        let cstring_pdf_full_file_path = c_pdf_full_file_path.unwrap();
        let cstring_build_path = c_build_path.unwrap();
        let scanner = synctex_scanner_new_with_output_file(
            cstring_pdf_full_file_path.as_ptr(),
            cstring_build_path.as_ptr(),
            1,
        );
        let tex_file_path = join_paths(&[proj_dir, params.path.clone(), params.file.clone()]);
        let demo_tex = CString::new(tex_file_path.clone());
        let mut position_list: Vec<PdfPosResp> = Vec::new();
        let cstring_demo_tex = demo_tex.unwrap();
        let node_number = synctex_display_query(
            scanner,
            cstring_demo_tex.as_ptr(),
            params.line as c_int,
            params.column as c_int,
            0,
        );
        if node_number > 0 {
            for _i in 0..node_number {
                let node: synctex_node_p = synctex_scanner_next_result(scanner);
                let page = synctex_node_page(node);
                let h = synctex_node_box_visible_h(node);
                let v = synctex_node_box_visible_v(node) + synctex_node_box_visible_depth(node);
                let x = synctex_node_visible_h(node);
                let y = synctex_node_visible_v(node);
                let width = synctex_node_box_visible_width(node).abs();
                let height = (synctex_node_box_visible_height(node)
                    + synctex_node_box_visible_depth(node))
                .max(1.0);
                let single_pos = PdfPosResp::from((page, h, v, width, height, x, y));
                position_list.push(single_pos);
            }
        }
        synctex_scanner_free(scanner);
        position_list
    }
}

pub fn get_src_pos(params: &GetSrcPosParams) -> Vec<SrcPosResp> {
    let proj_dir = get_proj_base_dir(&params.project_id);
    // Compilation runs in a dedicated pod (tex-render) whose working base differs from the
    // storage base of this server. The synctex file records source paths rooted at this
    // compile-side directory, so compute it once here for path resolution below.
    let compile_workspace_dir = get_proj_compile_workspace_dir(&params.project_id);
    let compile_out_dir = join_paths(&[proj_dir.clone(), get_compile_output_dir_name()]);
    let pdf_file_name = format!("{}{}", get_filename_without_ext(&params.main_file), ".pdf");
    let file_path = join_paths(&[&compile_out_dir, &pdf_file_name]);
    unsafe {
        let c_file_path = CString::new(file_path.clone());
        if let Err(e) = c_file_path {
            error!("parse out path error,{},{}", e, file_path);
            return Vec::new();
        }
        let c_build_path = CString::new(compile_out_dir.clone());
        if let Err(e) = c_build_path {
            error!("parse build path error,{},{}", e, compile_out_dir);
            return Vec::new();
        }
        let cstring_file_path = c_file_path.unwrap();
        let cstring_build_path = c_build_path.unwrap();
        let scanner = synctex_scanner_new_with_output_file(
            cstring_file_path.as_ptr(),
            cstring_build_path.as_ptr(),
            1,
        );
        let mut position_list: Vec<SrcPosResp> = Vec::new();
        let node_number = synctex_edit_query(scanner, params.page as c_int, params.h, params.v);
        if node_number > 0 {
            for _i in 0..node_number {
                let node: synctex_node_p = synctex_scanner_next_result(scanner);
                let file = synctex_scanner_get_name(scanner, synctex_node_tag(node));
                let line = synctex_node_line(node);
                let column = synctex_node_column(node);
                let c_str = CStr::from_ptr(file);
                let file_name: String = c_str.to_string_lossy().into_owned();
                // The synctex file records the source paths used at compile time, which are
                // rooted at the compile-side working directory (tex-render pod, e.g.
                // /tmp/texhub-compile/...), NOT at the storage directory of this server
                // (/opt/data/project/...). Because compute and storage are separated, we
                // must pass both roots so that the project-relative path can be extracted
                // regardless of which side generated the synctex file.
                let src_relative_path = get_file_relative_path(
                    file_name,
                    proj_dir.clone(),
                    &compile_workspace_dir,
                );
                // SyncTeX 可能只返回文件名（如 "skills.tex"）而文件位于子目录，
                // 补全为项目文件树中的完整相对路径（如 "engineering/intro/base/skills.tex"），
                // 供前端文件树定位；查库失败时回退为原始路径。
                let resolved_path =
                    resolve_src_file_path(&src_relative_path, &params.project_id);
                let single_pos = SrcPosResp::from((resolved_path, line, column));
                position_list.push(single_pos);
            }
        }
        synctex_scanner_free(scanner);
        position_list
    }
}

/// Convert a source path recorded in the synctex file into a project-relative path.
///
/// Because compute and storage are separated (compilation runs in the tex-render pod
/// under `texhub.compile_workspace_base_dir` such as `/tmp/texhub-compile`, while this
/// texhub-server pod stores projects under `texhub.compile_base_dir` such as
/// `/opt/data/project`), the synctex Input paths are rooted at the compile-side
/// directory, e.g. `/tmp/texhub-compile/2026/7/{proj_id}/./theory/./intro/intro.tex`.
///
/// We first try stripping the compile-side root so the full project-relative path such
/// as `theory/intro/intro.tex` is preserved (this is the ONLY way to distinguish
/// duplicate file names located in different directories). If the prefix does not match
/// (the file may have been compiled on the storage machine in some legacy/local setups),
/// we fall back to stripping the storage-side root. As a last resort we return only the
/// bare file name so callers can do a best-effort name lookup.
fn get_file_relative_path(
    file_full_path: String,
    proj_dir: String,
    compile_workspace_dir: &str,
) -> String {
    let abs_path = Path::new(file_full_path.as_str());
    // Try the compile-side root first: the synctex file is generated by the tex-render pod
    // whose working directory is `texhub.compile_workspace_base_dir`. Stripping this root
    // keeps the full project-relative path, which uniquely disambiguates files that share
    // the same name but live in different folders.
    let mut root = Path::new(compile_workspace_dir);
    match abs_path.strip_prefix(root) {
        Ok(relative) => {
            let relative_path = PathBuf::from(relative);
            let path_string = relative_path
                .as_os_str()
                .to_string_lossy()
                .to_string();
            return clean_synctex_name(path_string);
        }
        Err(_) => {}
    }
    // Some legacy or local setups compile directly under the storage directory, in which
    // case the recorded path is rooted at `texhub.compile_base_dir` (the storage side).
    root = Path::new(proj_dir.as_str());
    match abs_path.strip_prefix(root) {
        Ok(relative) => {
            let relative_path = PathBuf::from(relative);
            let path_string = relative_path
                .as_os_str()
                .to_string_lossy()
                .to_string();
            return clean_synctex_name(path_string);
        }
        Err(err) => {
            // synctex 中 Input: 记录的是编译时传入的相对路径（相对编译目录/项目根），
            // 例如 "main.tex" 或 "./chapters/intro.tex"，此时无法 strip 项目根，
            // 直接作为项目内相对路径清洗后返回。
            if !abs_path.is_absolute() {
                return clean_synctex_name(file_full_path);
            }
            // 绝对路径但无法匹配任何已知根时的兜底：取文件名。
            error!("Failed to get relative path, fallback to file name: {}", err);
            abs_path
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default()
        }
    }
}

fn clean_synctex_name(name: String) -> String {
    name.trim_start_matches("./").replace("./", "")
}

/// 将 SyncTeX 返回的文件名解析为项目文件树中的完整相对路径。
///
/// SyncTeX 只记录编译时的 Input 路径，可能只返回文件名（如 "skills.tex"），
/// 而文件实际位于子目录（tex_file.file_path 是父目录路径，不含文件名）。
/// 此时按 project_id + name 查询 tex_file 表补全路径，
/// 如 "engineering/intro/base/skills.tex"，供前端文件树定位；
/// 已是完整相对路径或查库失败时原样返回。
fn resolve_src_file_path(file_name: &str, project_id: &str) -> String {
    // 已含目录分隔符（完整相对路径）时无需查库
    if file_name.contains('/') || file_name.contains('\\') {
        return file_name.to_string();
    }
    let mut query = tex_file_table::table.into_boxed::<diesel::pg::Pg>();
    query = query
        .filter(tex_file_table::project_id.eq(project_id))
        .filter(tex_file_table::name.eq(file_name))
        .filter(tex_file_table::file_type.eq(1)); // 1 = 文件（0 = 文件夹）
    let cvs: Result<Vec<TexFile>, diesel::result::Error> =
        query.load::<TexFile>(&mut get_connection());
    match cvs {
        Ok(files) => {
            if let Some(f) = files.first() {
                let dir = f.file_path.trim_matches('/');
                if dir.is_empty() {
                    // 根目录文件（file_path = "/"）直接返回文件名
                    f.name.clone()
                } else {
                    format!("{}/{}", dir, f.name)
                }
            } else {
                file_name.to_string()
            }
        }
        Err(err) => {
            error!(
                "resolve src file path failed, query by name {} error: {}",
                file_name, err
            );
            file_name.to_string()
        }
    }
}
