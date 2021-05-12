use ureq::OrAnyStatus;

#[cfg(target_os = "windows")]
use ::{
    std::path::PathBuf,
    winreg::{enums::HKEY_CURRENT_USER, RegKey},
};

fn epoch() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn rbx_flags(ticket: &str, place: u32) -> String {
    format!(
        "--play -a \"https://auth.roblox.com/v1/authentication-ticket/redeem\" -t \"{game_info}\" -j \"https://assetgame.roblox.com/game/PlaceLauncher.ashx?request=RequestGame&browserTrackerId=0&placeId={place}&isPlayTogetherGame=false\" -b 0 --launchtime={now} --rloc en_us --gloc en_us",
        game_info = ticket, place = place, now = epoch())
}

pub fn xcsrf(cookie: &str) -> Option<String> {
    ureq::post("https://auth.roblox.com/v2/logout")
        .set("Cookie", &format!(".ROBLOSECURITY={}", cookie))
        .set("Content-Length", "0")
        .call()
        .or_any_status()
        .ok()?
        .header("x-csrf-token")
        .map(String::from)
}

pub fn ticket(cookie: &str, place: u32) -> Option<String> {
    ureq::post("https://auth.roblox.com/v1/authentication-ticket")
        .set("Cookie", &format!(".ROBLOSECURITY={}", cookie))
        .set("RBX-For-Gameauth", "true")
        .set(
            "Referer",
            &format!("https://www.roblox.com/games/{}/1", place),
        )
        .set("X-CSRF-TOKEN", &xcsrf(cookie).unwrap())
        .set("Content-Length", "0")
        .call()
        .ok()?
        .header("rbx-authentication-ticket")
        .map(String::from)
}

#[cfg(target_os = "windows")]
pub fn get_cookie() -> Option<String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let cookies = hkcu
        .open_subkey("Software\\Roblox\\RobloxStudioBrowser\\roblox.com")
        .ok()?;
    let entry: String = cookies.get_value(".ROBLOSECURITY").ok()?;
    let mut cookie = None;
    for key_val in entry.split(",") {
        let mut pieces = key_val.split("::");
        if let Some("COOK") = pieces.next() {
            let value = pieces.next()?;
            if value.starts_with('<') && value.ends_with('>') {
                cookie = Some(&value[1..value.len() - 1]);
            }
        }
    }
    Some(cookie?.to_owned())
}

#[cfg(target_os = "windows")]
pub fn locate_roblox() -> Option<PathBuf> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key: String = hkcu
        .open_subkey("Software\\ROBLOX Corporation\\Environments\\roblox-player")
        .ok()?
        .get_value("")
        .ok()?;
    let path = PathBuf::from(&key)
        .parent()?
        .to_path_buf()
        .join("RobloxPlayerBeta.exe");
    Some(path)
}

#[cfg(target_os = "macos")]
pub fn locate_roblox() -> Option<String> {
    None
}

#[cfg(target_os = "macos")]
pub fn get_cookie() -> Option<String> {
    None
}
