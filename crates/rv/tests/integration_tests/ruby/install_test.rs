use current_platform::CURRENT_PLATFORM;

use crate::common::RvTest;
use std::fs;

fn arch() -> &'static str {
    match CURRENT_PLATFORM {
        "aarch64-apple-darwin" => "arm64_sonoma",
        "x86_64-apple-darwin" => "ventura",
        "x86_64-unknown-linux-gnu" => "x86_64_linux",
        "aarch64-unknown-linux-gnu" => "arm64_linux",
        other => panic!("Unsupported platform {other}"),
    }
}

#[test]
fn test_ruby_install_successful_download() {
    let mut test = RvTest::new();

    let tarball_content = create_mock_tarball();
    let download_suffix = make_dl_suffix("3.4.5");
    let _mock = test
        .mock_tarball_download(&download_suffix, &tarball_content)
        .create();

    test.env.remove("RV_NO_CACHE");
    let cache_dir = test.temp_dir.path().join("cache");
    test.env
        .insert("RV_CACHE_DIR".into(), cache_dir.as_str().into());

    let output = test.rv(&["ruby", "install", "3.4.5"]);

    output.assert_success();

    let cache_key = rv_cache::cache_digest(format!("{}/{}", test.server_url(), download_suffix));
    let tarball_path = cache_dir
        .join("ruby-v0")
        .join("tarballs")
        .join(format!("{}.tar.gz", cache_key));
    assert!(tarball_path.exists(), "Tarball should be cached");

    let temp_path = cache_dir
        .join("ruby-v0")
        .join("tarballs")
        .join(format!("{}.tar.gz.tmp", cache_key));
    assert!(
        !temp_path.exists(),
        "Temp file should not exist after successful download"
    );

    let cached_content = fs::read(&tarball_path).expect("Should be able to read cached tarball");
    assert_eq!(
        cached_content, tarball_content,
        "Cached content should match downloaded content"
    );
}

#[test]
fn test_ruby_install_from_tarball() {
    let mut test = RvTest::new();

    let tarball_content = create_mock_tarball();
    let filename = make_tarball_file_name("3.4.5");
    let tarball_file = test.mock_tarball_on_disk(&filename, &tarball_content);

    let tarball_path = tarball_file.as_str();
    let output = test.rv(&["ruby", "install", "3.4.5", "--tarball-path", tarball_path]);

    output.assert_success();

    // TODO need to verify that this actually did what we expected?
}

#[test]
fn test_ruby_install_http_failure_no_empty_file() {
    let mut test = RvTest::new();

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    test.server
        .mock("GET", "/portable-ruby-3.4.5.arm64_sonoma.bottle.tar.gz")
        .with_status(404)
        .create();
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    test.server
        .mock("GET", "/portable-ruby-3.4.5.ventura.bottle.tar.gz")
        .with_status(404)
        .create();
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    test.server
        .mock("GET", "/portable-ruby-3.4.5.x86_64_linux.bottle.tar.gz")
        .with_status(404)
        .create();
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    test.server
        .mock("GET", "/portable-ruby-3.4.5.arm64_linux.bottle.tar.gz")
        .with_status(404)
        .create();

    test.env.remove("RV_NO_CACHE");
    let cache_dir = test.temp_dir.path().join("cache");
    test.env
        .insert("RV_CACHE_DIR".into(), cache_dir.as_str().into());

    let output = test.rv(&["ruby", "install", "3.4.5"]);

    output.assert_failure();

    let arch = arch();
    let cache_key = rv_cache::cache_digest(format!(
        "{}/portable-ruby-3.4.5.{arch}.bottle.tar.gz",
        test.server_url()
    ));
    let tarball_path = cache_dir
        .join("ruby-v0")
        .join("tarballs")
        .join(format!("{}.tar.gz", cache_key));
    let temp_path = cache_dir
        .join("ruby-v0")
        .join("tarballs")
        .join(format!("{}.tar.gz.tmp", cache_key));

    assert!(
        !tarball_path.exists(),
        "No tarball should be created on HTTP failure"
    );
    assert!(
        !temp_path.exists(),
        "No temp file should remain on HTTP failure"
    );
}

#[test]
fn test_ruby_install_interrupted_download_cleanup() {
    let mut test = RvTest::new();

    let download_suffix = make_dl_suffix("3.4.5");
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    let _mock = test
        .server
        .mock("GET", "/latest/download/ruby-3.4.5.arm64_sonoma.tar.gz")
        .with_status(200)
        .with_header("content-type", "application/gzip")
        .with_body("partial")
        .create();
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    let _mock = test
        .server
        .mock("GET", "/latest/download/ruby-3.4.5.ventura.tar.gz")
        .with_status(200)
        .with_header("content-type", "application/gzip")
        .with_body("partial")
        .create();
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    let _mock = test
        .server
        .mock("GET", "/latest/download/ruby-3.4.5.x86_64_linux.tar.gz")
        .with_status(200)
        .with_header("content-type", "application/gzip")
        .with_body("partial")
        .create();
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    let _mock = test
        .server
        .mock("GET", "/latest/download/ruby-3.4.5.arm64_linux.tar.gz")
        .with_status(200)
        .with_header("content-type", "application/gzip")
        .with_body("partial")
        .create();

    test.env.remove("RV_NO_CACHE");
    let cache_dir = test.temp_dir.path().join("cache");
    test.env
        .insert("RV_CACHE_DIR".into(), cache_dir.as_str().into());

    let output = test.rv(&["ruby", "install", "3.4.5"]);

    output.assert_failure();

    let tarball_name = format!("{}/{}", test.server_url(), download_suffix);
    dbg!(&tarball_name);
    let cache_key = rv_cache::cache_digest(tarball_name);
    let tarball_path = cache_dir
        .join("ruby-v0")
        .join("tarballs")
        .join(format!("{}.tar.gz", cache_key));
    let temp_path = cache_dir
        .join("ruby-v0")
        .join("tarballs")
        .join(format!("{}.tar.gz.tmp", cache_key));

    assert!(
        tarball_path.exists(),
        "Tarball should exist at {} after successful download",
        tarball_path,
    );
    assert!(
        !temp_path.exists(),
        "No temp file should remain at {} after failure",
        temp_path,
    );
}

#[test]
fn test_ruby_install_cached_file_reused() {
    let mut test = RvTest::new();

    let tarball_content = create_mock_tarball();
    let download_suffix = make_dl_suffix("3.4.5");
    let mock = test
        .mock_tarball_download(&download_suffix, &tarball_content)
        .expect(1)
        .create();

    test.env.remove("RV_NO_CACHE");
    let cache_dir = test.temp_dir.path().join("cache");
    test.env
        .insert("RV_CACHE_DIR".into(), cache_dir.as_str().into());

    let output1 = test.rv(&["ruby", "install", "3.4.5"]);
    output1.assert_success();

    let output2 = test.rv(&["ruby", "install", "3.4.5"]);
    output2.assert_success();

    assert!(
        output2
            .stdout()
            .contains("already exists, skipping download")
    );

    mock.assert();
}

#[test]
fn test_ruby_install_invalid_url() {
    let mut test = RvTest::new();

    test.env.insert(
        "RV_RELEASES_URL".into(),
        "http://invalid-url-that-does-not-exist.com".into(),
    );

    test.env.remove("RV_NO_CACHE");
    let cache_dir = test.temp_dir.path().join("cache");
    test.env
        .insert("RV_CACHE_DIR".into(), cache_dir.as_str().into());

    let output = test.rv(&["ruby", "install", "3.4.5"]);

    output.assert_failure();

    let tarballs_dir = cache_dir.join("ruby-v0").join("tarballs");
    if tarballs_dir.exists() {
        let entries: Vec<_> = fs::read_dir(&tarballs_dir).unwrap().collect();
        assert!(
            entries.is_empty(),
            "No files should be created in tarballs directory"
        );
    }
}

fn make_dl_suffix(version: &str) -> String {
    let filename = make_tarball_file_name(version);
    format!("latest/download/{filename}")
}

fn make_tarball_file_name(version: &str) -> String {
    let suffix = make_platform_suffix();
    format!("ruby-{version}.{suffix}.tar.gz")
}

fn make_platform_suffix() -> String {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    let suffix = "arm64_sonoma";
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    let suffix = "ventura";
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    let suffix = "arm64_linux";
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    let suffix = "x86_64_linux";

    suffix.to_string()
}

#[test]
fn test_ruby_install_atomic_rename_behavior() {
    let mut test = RvTest::new();

    let tarball_content = create_mock_tarball();
    let download_suffix = make_dl_suffix("3.4.5");
    let _mock = test
        .mock_tarball_download(&download_suffix, &tarball_content)
        .create();

    test.env.remove("RV_NO_CACHE");
    let cache_dir = test.temp_dir.path().join("cache");
    test.env
        .insert("RV_CACHE_DIR".into(), cache_dir.as_str().into());

    let output = test.rv(&["ruby", "install", "3.4.5"]);
    output.assert_success();

    let cache_key = rv_cache::cache_digest(format!("{}/{}", test.server_url(), download_suffix));
    let tarball_path = cache_dir
        .join("ruby-v0")
        .join("tarballs")
        .join(format!("{}.tar.gz", cache_key));

    assert!(tarball_path.exists(), "Final tarball should exist");

    let metadata = fs::metadata(&tarball_path).expect("Should be able to get file metadata");
    assert!(metadata.len() > 0, "Tarball should not be empty");

    let content = fs::read(&tarball_path).expect("Should be able to read tarball");
    assert_eq!(content, tarball_content, "Content should match exactly");
}

#[test]
fn test_ruby_install_temp_file_cleanup_on_extraction_failure() {
    let mut test = RvTest::new();

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    let _mock = test
        .server
        .mock(
            "GET",
            "/download/3.4.5/portable-ruby-3.4.5.arm64_sonoma.bottle.tar.gz",
        )
        .with_status(200)
        .with_header("content-type", "application/gzip")
        .with_body("invalid-tarball-content")
        .create();

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    let _mock = test
        .server
        .mock(
            "GET",
            "/download/3.4.5/portable-ruby-3.4.5.ventura.bottle.tar.gz",
        )
        .with_status(200)
        .with_header("content-type", "application/gzip")
        .with_body("invalid-tarball-content")
        .create();

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    let _mock = test
        .server
        .mock(
            "GET",
            "/download/3.4.5/portable-ruby-3.4.5.x86_64_linux.bottle.tar.gz",
        )
        .with_status(200)
        .with_header("content-type", "application/gzip")
        .with_body("invalid-tarball-content")
        .create();

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    let _mock = test
        .server
        .mock(
            "GET",
            "/download/3.4.5/portable-ruby-3.4.5.arm64_linux.bottle.tar.gz",
        )
        .with_status(200)
        .with_header("content-type", "application/gzip")
        .with_body("invalid-tarball-content")
        .create();

    test.env.remove("RV_NO_CACHE");
    let cache_dir = test.temp_dir.path().join("cache");
    test.env
        .insert("RV_CACHE_DIR".into(), cache_dir.as_str().into());

    let output = test.rv(&["ruby", "install", "3.4.5"]);

    output.assert_failure();

    let arch = arch();
    let cache_key = rv_cache::cache_digest(format!(
        "{}/download/3.4.5/portable-ruby-3.4.5.{arch}.bottle.tar.gz",
        test.server_url()
    ));
    let temp_path = cache_dir
        .join("ruby-v0")
        .join("tarballs")
        .join(format!("{}.tar.gz.tmp", cache_key));

    assert!(!temp_path.exists(), "Temp file should be cleaned up");
}

fn create_mock_tarball() -> Vec<u8> {
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;
    use tar::Builder;

    let mut archive_data = Vec::new();
    {
        let mut builder = Builder::new(&mut archive_data);

        let mut dir_header = tar::Header::new_gnu();
        dir_header.set_path("portable-ruby/").unwrap();
        dir_header.set_size(0);
        dir_header.set_mode(0o755);
        dir_header.set_entry_type(tar::EntryType::Directory);
        dir_header.set_cksum();
        builder.append(&dir_header, std::io::empty()).unwrap();

        let mut bin_dir_header = tar::Header::new_gnu();
        bin_dir_header.set_path("portable-ruby/bin/").unwrap();
        bin_dir_header.set_size(0);
        bin_dir_header.set_mode(0o755);
        bin_dir_header.set_entry_type(tar::EntryType::Directory);
        bin_dir_header.set_cksum();
        builder.append(&bin_dir_header, std::io::empty()).unwrap();

        let ruby_content = "#!/bin/bash\necho 'mock ruby'\n";
        let mut ruby_header = tar::Header::new_gnu();
        ruby_header.set_path("portable-ruby/bin/ruby").unwrap();
        ruby_header.set_size(ruby_content.len() as u64);
        ruby_header.set_mode(0o755);
        ruby_header.set_cksum();
        builder
            .append(&ruby_header, ruby_content.as_bytes())
            .unwrap();

        builder.finish().unwrap();
    }

    let mut gz_data = Vec::new();
    {
        let mut encoder = GzEncoder::new(&mut gz_data, Compression::default());
        encoder.write_all(&archive_data).unwrap();
        encoder.finish().unwrap();
    }

    gz_data
}
