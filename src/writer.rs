use crate::args::FusionArgs;
use crate::fs_utils::{create_file_with_dirs, read_file};
use crate::metadata::ProjectMetadata;
use anyhow::Result;
use std::io::{BufWriter, Write, stdout};
use std::path::PathBuf;

pub fn write_output(
    args: &FusionArgs,
    files: Vec<(PathBuf, Vec<u8>)>,
    metadata: Option<ProjectMetadata>,
    toc: Option<Vec<u8>>,
) -> Result<()> {
    let mut writer = create_writer(args)?;

    write_header(&mut writer, args)?;
    write_metadata(&mut writer, metadata)?;
    write_toc(&mut writer, toc)?;
    write_files(&mut writer, &files, &args.separator)?;

    Ok(())
}

fn create_writer(args: &FusionArgs) -> Result<Box<dyn Write>> {
    if args.stdout {
        Ok(Box::new(BufWriter::new(stdout())))
    } else {
        Ok(Box::new(BufWriter::new(create_file_with_dirs(
            &args.output,
        )?)))
    }
}

// fn write_if_exists(
//     writer: &mut dyn Write,
//     data: Option<impl AsRef<[u8]>>,
//     label: &str,
// ) -> Result<()> {
//     if let Some(data) = data {
//         writer.write_all(data.as_ref())?;
//     }
//     Ok(())
// }

fn write_header(writer: &mut dyn Write, args: &FusionArgs) -> Result<()> {
    if let Some(head_path) = &args.head {
        writer.write_all(&read_file(head_path)?)?;
    }
    Ok(())
}

fn write_metadata(writer: &mut dyn Write, metadata: Option<ProjectMetadata>) -> Result<()> {
    if let Some(meta) = metadata {
        write!(writer, "{}", meta)?;
    }
    Ok(())
}

fn write_toc(writer: &mut dyn Write, toc: Option<Vec<u8>>) -> Result<()> {
    if let Some(toc_data) = toc {
        writer.write_all(&toc_data)?;
    }
    Ok(())
}

fn write_files(
    writer: &mut dyn Write,
    files: &[(PathBuf, Vec<u8>)],
    separator: &str,
) -> Result<()> {
    for (path, content) in files {
        writeln!(writer, "{} {}", separator, path.display())?;
        writer.write_all(content)?;
        writer.write_all(b"\n")?;
    }
    Ok(())
}
