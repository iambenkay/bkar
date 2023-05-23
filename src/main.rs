use bkar::archiver;

fn main() {
    archive();

    // de_archive();
}

fn archive() {
    let dir = String::from("/Users/benjamin/Desktop/test-archive");
    let output = String::from("/Users/benjamin/Desktop/test-archive.bkar");
    archiver::create_archive_from_dir(&dir, &output);
}

fn de_archive() {
    let archive = String::from("/Users/benjamin/Desktop/test-archive.bkar");
    let output = String::from("/Users/benjamin/Desktop/output-archive");
    archiver::create_dir_from_archive(&archive, &output);
}
