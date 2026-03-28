fn main() {
    //let _ = steamworks::init("./libs/win64/steam_api64.dll".into()).unwrap();
    let _ = steamworks::init("./libs/linux64/libsteam_api.so".into()).unwrap();
}