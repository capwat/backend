use super::*;
use capwat_vfs::{backend::InMemoryFs, VfsSnapshot};

#[test]
fn locate() {
    let imfs = InMemoryFs::new();
    let snapshot = VfsSnapshot::build_dir().directory(
        "app",
        VfsSnapshot::build_dir()
            .directory("server", VfsSnapshot::build_dir().file("capwat.toml", b""))
            .directory("no-config", VfsSnapshot::build_dir())
            .file("capwat.toml", b""),
    );

    let imfs = imfs.apply_snapshot("/", snapshot.build()).unwrap();
    let vfs = Vfs::new(imfs);

    // it should go with the current directory and return None
    assert_eq!(Server::locate(&vfs, None), None);

    // it should go from /app/server/ and take /app/server/capwat.toml
    assert_eq!(
        Server::locate(&vfs, Some(Path::new("/app/server"))),
        Some(PathBuf::from("/app/server/capwat.toml"))
    );

    // it should with /app/capwat.toml if currenty directory is in with no capwat.toml file
    assert_eq!(
        Server::locate(&vfs, Some(Path::new("/app/no-config"))),
        Some(PathBuf::from("/app/capwat.toml"))
    );

    // it should return None if the directory does not exists
    assert_eq!(
        Server::locate(&vfs, Some(Path::new("/app/error/error"))),
        None
    );
}
