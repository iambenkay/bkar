use std::cmp::min;
use std::fs;
use std::fs::{File, read_dir};
use std::io::{BufReader, BufWriter, Read, Write};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

const PATH_HEADER: &[u8; 3] = b"PTH";
const DATA_HEADER: &[u8; 3] = b"DAT";
const BKAR_ARCHIVE_IDENTIFIER: &[u8; 8] = b"<!BKAR!>";

pub fn create_archive_from_dir(path: &str, output: &str) {
    let mut writer = BufWriter::new(create_file(output));

    write_archive_identifier(&mut writer);

    let root_dir = Path::new(path).parent().unwrap().to_str().unwrap();
    write_dir_to_archive(path, root_dir, &mut writer);
}

pub fn create_dir_from_archive(path: &str, output_dir: &str) {
    if Path::new(output_dir).exists() {
        fs::remove_dir_all(output_dir).unwrap();
    }
    let archive_file = File::open(path).unwrap();
    let mut reader = BufReader::new(archive_file);

    check_archive_identifier(&mut reader);

    loop {
        if check_path_header(&mut reader).unwrap() == 1 {
            break;
        }

        let path = read_path(&mut reader);
        let file_path = Path::new(output_dir).join(path);

        check_data_header(&mut reader);
        persist_data_to_path(file_path, &mut reader);
    }
}

fn write_dir_to_archive(path: &str, root_dir: &str, writer: &mut BufWriter<File>) {
    let paths = read_dir(path).unwrap();

    for path in paths {
        let path = path.unwrap().path();

        let path_str = path.to_str().unwrap();

        if !path.is_dir() {
            write_file_to_archive(path_str, root_dir, writer);
        } else {
            write_dir_to_archive(path_str, root_dir, writer);
        }
    }
}

fn write_file_to_archive(path: &str, root_dir: &str, writer: &mut BufWriter<File>) {
    let in_archive_path = path.replace(&format!("{}/", root_dir), "");
    let file = File::open(path).unwrap();

    write_path_box(in_archive_path, writer);
    write_data_box(file, writer);
}

fn write_path_box(path: String, writer: &mut BufWriter<File>) {
    writer.write_all(PATH_HEADER).unwrap();

    let path_size = path.as_bytes().len();
    if path_size > 4096 {
        panic!("Path size is too large!");
    }

    let path_size = path_size as u16;
    writer.write_all(&path_size.to_be_bytes()).unwrap();

    writer.write_all(path.as_bytes()).unwrap();
}

fn write_data_box(file: File, writer: &mut BufWriter<File>) {
    writer.write_all(DATA_HEADER).unwrap();
    let data_size = file.metadata().unwrap().size();

    writer.write_all(&data_size.to_be_bytes()).unwrap();

    let mut reader = BufReader::new(file);

    let mut buffer = [0; 4096];

    loop {
        let n = reader.read(&mut buffer).unwrap();
        if n == 0 {
            break;
        }

        writer.write_all(&buffer[..n]).unwrap();
    }
    writer.flush().unwrap();
}

fn check_archive_identifier(reader: &mut BufReader<File>) {
    let mut buffer: [u8; 8] = [0; 8];

    reader.read(&mut buffer).unwrap();

    if &buffer != BKAR_ARCHIVE_IDENTIFIER {
        panic!("Invalid Archive");
    }
}

fn check_path_header(reader: &mut BufReader<File>) -> Result<u32, &str> {
    let mut buffer: [u8; 3] = [0; 3];

    let bytes_read = reader.read(&mut buffer).unwrap();

    if bytes_read == 0 {
        return Ok(1);
    }

    if &buffer != PATH_HEADER {
        return Err("Invalid Archive");
    }

    Ok(0)
}

fn read_path(reader: &mut BufReader<File>) -> String {
    let mut buffer: [u8; 2] = [0; 2];
    reader.read(&mut buffer).unwrap();

    let path_size = u16::from_be_bytes(buffer);

    let mut path_buffer = vec![0; path_size as usize];

    reader.read(&mut path_buffer).unwrap();

    let path = String::from_utf8(path_buffer).unwrap();

    path
}

fn check_data_header(reader: &mut BufReader<File>) {
    let mut buffer: [u8; 3] = [0; 3];

    reader.read(&mut buffer).unwrap();

    if &buffer != DATA_HEADER {
        panic!("Invalid Archive");
    }
}

fn persist_data_to_path(path: PathBuf, reader: &mut BufReader<File>) {
    let mut buffer: [u8; 8] = [0; 8];
    reader.read(&mut buffer).unwrap();

    let data_size = u64::from_be_bytes(buffer);

    let mut bytes_read: u64 = 0;

    let mut writer = BufWriter::new(create_file(path.to_str().unwrap()));

    while bytes_read < data_size {
        let mut buffer = vec![0; min(4096, (data_size - bytes_read) as usize)];
        let n = reader.read(&mut buffer).unwrap();

        writer.write_all(&buffer[..n]).unwrap();
        bytes_read += n as u64;
    }
    writer.flush().unwrap();
}

fn write_archive_identifier(writer: &mut BufWriter<File>) {
    writer.write_all(BKAR_ARCHIVE_IDENTIFIER).unwrap();
}

fn create_file(filename: &str) -> File {
    let path = Path::new(filename);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    File::create(filename).unwrap()
}