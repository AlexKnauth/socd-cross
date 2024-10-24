set -ex

main() {
    local src=$(pwd) \
          stage=

    if [[ $OS_NAME =~ ^macos\-.*$ ]]; then
        stage=$(mktemp -d -t tmp)
    else
        stage=$(mktemp -d)
    fi

    cp "src-tauri/target/release/socd-cross.exe" "$stage/socd-cross.exe" 2>/dev/null || :
    cp "src-tauri/target/release/socd-cross" "$stage/socd-cross" 2>/dev/null || :
    cp -R "src-tauri/target/release/bundle/macos/socd-cross.app" "$stage/socd-cross.app" 2>/dev/null || :

    cd $stage
    if [ "$OS_NAME" = "windows-latest" ]; then
        7z a $src/socd-cross-$TARGET.zip *
    else
        tar czf $src/socd-cross-$TARGET.tar.gz *
    fi
    cd $src

    rm -rf $stage
}

main
